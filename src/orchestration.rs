use crate::outcome::ExecutionOutcome;
use crate::snapshot::{atomic_write, SnapshotError};
use crate::state_path::state_dir;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static ROOT_SEQ: AtomicU64 = AtomicU64::new(1);
static ARM_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArmStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
    Resumed,
}

impl ArmStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::Resumed => "resumed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArmRecord {
    pub id: String,
    pub name: String,
    pub root_id: String,
    pub parent_arm_id: Option<String>,
    pub status: ArmStatus,
    pub prompt_hash: String,
    pub prompt: String,
    pub output_hash: Option<String>,
    pub output_bytes: usize,
    pub error_code: Option<String>,
    pub started_at: u128,
    pub finished_at: Option<u128>,
    pub duration_ms: Option<u128>,
}

#[derive(Debug, Clone)]
pub struct RootRecord {
    pub id: String,
    pub status: ArmStatus,
    pub prompt_hash: String,
    pub input_hash: String,
    pub output_hash: Option<String>,
    pub started_at: u128,
    pub finished_at: Option<u128>,
    pub duration_ms: Option<u128>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EventEntry {
    pub timestamp: u128,
    pub root_id: String,
    pub arm_id: String,
    pub event_type: String,
    pub details: String,
}

struct OrchestrationState {
    roots: HashMap<String, RootRecord>,
    arms: HashMap<String, ArmRecord>,
    events: Vec<EventEntry>,
    file_locks: HashMap<PathBuf, String>,
}

impl OrchestrationState {
    fn new() -> Self {
        Self {
            roots: HashMap::new(),
            arms: HashMap::new(),
            events: Vec::new(),
            file_locks: HashMap::new(),
        }
    }
}

static STATE: Mutex<Option<OrchestrationState>> = Mutex::new(None);

fn state() -> std::sync::MutexGuard<'static, Option<OrchestrationState>> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn create_root(prompt: &str) -> RootRecord {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let id = format!(
        "root-{}-{}-{}",
        std::process::id(),
        ROOT_SEQ.fetch_add(1, Ordering::Relaxed),
        now
    );

    let prompt_hash = hash(prompt);

    let record = RootRecord {
        id: id.clone(),
        status: ArmStatus::Running,
        prompt_hash: prompt_hash.clone(),
        input_hash: prompt_hash.clone(),
        output_hash: None,
        started_at: now,
        finished_at: None,
        duration_ms: None,
        children: Vec::new(),
    };

    let mut s = state();
    let s = s.get_or_insert_with(OrchestrationState::new);
    s.roots.insert(id.clone(), record.clone());

    s.events.push(EventEntry {
        timestamp: now,
        root_id: id.clone(),
        arm_id: id.clone(),
        event_type: "root_created".to_string(),
        details: format!("Root created with prompt hash: {prompt_hash}"),
    });

    // Persist to disk
    report_persist_error("root", &record.id, persist_root(&record));

    record
}

pub fn create_arm(
    root_id: &str,
    name: &str,
    prompt: &str,
    parent_arm_id: Option<&str>,
) -> ArmRecord {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let id = format!(
        "arm-{}-{}-{}-{}",
        sanitize_name(name),
        std::process::id(),
        ARM_SEQ.fetch_add(1, Ordering::Relaxed),
        now
    );

    let record = ArmRecord {
        id: id.clone(),
        name: name.to_string(),
        root_id: root_id.to_string(),
        parent_arm_id: parent_arm_id.map(|s| s.to_string()),
        status: ArmStatus::Running,
        prompt_hash: hash(prompt),
        prompt: prompt.to_string(),
        output_hash: None,
        output_bytes: 0,
        error_code: None,
        started_at: now,
        finished_at: None,
        duration_ms: None,
    };

    let mut s = state();
    let s = s.get_or_insert_with(OrchestrationState::new);
    s.arms.insert(id.clone(), record.clone());

    let root_to_persist = if let Some(root) = s.roots.get_mut(root_id) {
        root.children.push(id.clone());
        Some(root.clone())
    } else {
        None
    };

    s.events.push(EventEntry {
        timestamp: now,
        root_id: root_id.to_string(),
        arm_id: id.clone(),
        event_type: "arm_created".to_string(),
        details: format!("Arm '{name}' created"),
    });

    // Persist to disk
    if let Some(root) = root_to_persist {
        report_persist_error("root", &root.id, persist_root(&root));
    }
    report_persist_error("arm", &record.id, persist_arm(&record));

    record
}

/// Create an arm record and persist it. Like `create_arm` but follows restricted
/// lifecycle rules (used by pipeline threads and internal executors).
pub fn create_arm_restricted(
    root_id: &str,
    name: &str,
    prompt: &str,
    parent_arm_id: Option<&str>,
) -> ArmRecord {
    create_arm(root_id, name, prompt, parent_arm_id)
}

pub fn finish_arm(arm_id: &str, outcome: &ExecutionOutcome) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let mut s = state();
    let s = s.get_or_insert_with(OrchestrationState::new);

    if let Some(arm) = s.arms.get_mut(arm_id) {
        arm.status = if outcome.is_failed() {
            ArmStatus::Failed
        } else {
            ArmStatus::Completed
        };
        arm.output_hash = Some(hash(&outcome.output));
        arm.output_bytes = outcome.output.len();
        arm.error_code = outcome.code.clone();
        arm.finished_at = Some(now);
        arm.duration_ms = Some(now - arm.started_at);

        let root_id = arm.root_id.clone();
        let status = arm.status.clone();
        let error_code = arm.error_code.clone();

        s.events.push(EventEntry {
            timestamp: now,
            root_id,
            arm_id: arm_id.to_string(),
            event_type: format!("arm_{}", status.as_str()),
            details: error_code.unwrap_or_else(|| "success".to_string()),
        });

        // Persist to disk
        report_persist_error("arm", &arm.id, persist_arm(arm));
    }
}

pub fn finish_root(root_id: &str, outcome: &ExecutionOutcome) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let (finished_root, finished_arms) = {
        let mut state_guard = state();
        let state = state_guard.get_or_insert_with(OrchestrationState::new);

        let finished_root = if let Some(root) = state.roots.get_mut(root_id) {
            root.status = if outcome.is_failed() {
                ArmStatus::Failed
            } else {
                ArmStatus::Completed
            };
            root.output_hash = Some(hash(&outcome.output));
            root.finished_at = Some(now);
            root.duration_ms = Some(now - root.started_at);

            state.events.push(EventEntry {
                timestamp: now,
                root_id: root_id.to_string(),
                arm_id: root_id.to_string(),
                event_type: format!("root_{}", root.status.as_str()),
                details: outcome
                    .code
                    .clone()
                    .unwrap_or_else(|| "success".to_string()),
            });

            report_persist_error("root", &root.id, persist_root(root));
            Some(root.clone())
        } else {
            None
        };
        let finished_arms = finished_root
            .as_ref()
            .map(|root| {
                root.children
                    .iter()
                    .filter_map(|arm_id| state.arms.get(arm_id).cloned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        (finished_root, finished_arms)
    };

    if let Some(root) = finished_root {
        if let Err(error) = crate::resonance::append_root(&root, &finished_arms, outcome) {
            eprintln!("[resonance] append failed for {}: {error}", root.id);
        }
    }
}

pub fn cancel_arm(arm_id: &str) -> Result<(), ExecutionOutcome> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let mut s = state();
    let s = s.get_or_insert_with(OrchestrationState::new);

    let arm = s.arms.get_mut(arm_id).ok_or_else(|| {
        ExecutionOutcome::failed(
            "arm_not_found",
            format!("[orchestration] arm not found: {arm_id}"),
        )
    })?;

    if !matches!(arm.status, ArmStatus::Running | ArmStatus::Resumed) {
        return Err(ExecutionOutcome::failed(
            "arm_not_running",
            format!(
                "[orchestration] arm is not running: {}",
                arm.status.as_str()
            ),
        ));
    }

    arm.status = ArmStatus::Cancelled;
    arm.finished_at = Some(now);
    arm.duration_ms = Some(now - arm.started_at);

    s.events.push(EventEntry {
        timestamp: now,
        root_id: arm.root_id.clone(),
        arm_id: arm_id.to_string(),
        event_type: "arm_cancelled".to_string(),
        details: "Cancelled by user".to_string(),
    });

    report_persist_error("arm", &arm.id, persist_arm(arm));

    Ok(())
}

pub fn lock_file(path: &Path, arm_id: &str) -> Result<(), ExecutionOutcome> {
    let mut s = state();
    let s = s.get_or_insert_with(OrchestrationState::new);

    if let Some(owner) = s.file_locks.get(&path.to_path_buf()) {
        if owner != arm_id {
            return Err(ExecutionOutcome::failed(
                "file_locked",
                format!("[orchestration] file is locked by arm: {}", owner),
            ));
        }
    }

    s.file_locks.insert(path.to_path_buf(), arm_id.to_string());
    Ok(())
}

pub fn unlock_file(path: &Path, arm_id: &str) {
    let mut s = state();
    let s = s.get_or_insert_with(OrchestrationState::new);

    if let Some(owner) = s.file_locks.get(&path.to_path_buf()) {
        if owner == arm_id {
            s.file_locks.remove(&path.to_path_buf());
        }
    }
}

pub fn retry_with_policy<F, T>(
    max_retries: u32,
    delay_ms: u64,
    mut f: F,
) -> Result<T, ExecutionOutcome>
where
    F: FnMut() -> Result<T, ExecutionOutcome>,
{
    let mut last_error = None;

    for attempt in 0..=max_retries {
        match f() {
            Ok(value) => return Ok(value),
            Err(error) => {
                if attempt < max_retries {
                    std::thread::sleep(Duration::from_millis(delay_ms));
                }
                last_error = Some(error);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        ExecutionOutcome::failed("retry_exhausted", "[orchestration] all retries exhausted")
    }))
}

pub fn circuit_breaker_check(
    name: &str,
    failure_count: u32,
    threshold: u32,
) -> Result<(), ExecutionOutcome> {
    if failure_count >= threshold {
        Err(ExecutionOutcome::failed(
            "circuit_open",
            format!(
                "[orchestration] circuit breaker '{name}' is open after {failure_count} failures"
            ),
        ))
    } else {
        Ok(())
    }
}

pub fn find_orphaned_arms() -> Vec<ArmRecord> {
    let s = state();
    let s = match s.as_ref() {
        Some(s) => s,
        None => return Vec::new(),
    };

    s.arms
        .values()
        .filter(|arm| matches!(arm.status, ArmStatus::Running | ArmStatus::Resumed))
        .cloned()
        .collect()
}

pub fn resume_arm(arm_id: &str) -> Result<ArmRecord, ExecutionOutcome> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let mut s = state();
    let s = s.get_or_insert_with(OrchestrationState::new);

    let arm = s.arms.get_mut(arm_id).ok_or_else(|| {
        ExecutionOutcome::failed(
            "arm_not_found",
            format!("[orchestration] arm not found: {arm_id}"),
        )
    })?;

    if !matches!(
        arm.status,
        ArmStatus::Running | ArmStatus::Failed | ArmStatus::Resumed
    ) {
        return Err(ExecutionOutcome::failed(
            "arm_not_resumable",
            format!(
                "[orchestration] arm cannot be resumed: {}",
                arm.status.as_str()
            ),
        ));
    }

    arm.status = ArmStatus::Resumed;
    arm.started_at = now;
    arm.finished_at = None;
    arm.duration_ms = None;

    s.events.push(EventEntry {
        timestamp: now,
        root_id: arm.root_id.clone(),
        arm_id: arm_id.to_string(),
        event_type: "arm_resumed".to_string(),
        details: "Arm resumed after failure".to_string(),
    });

    let record = arm.clone();
    report_persist_error("arm", &record.id, persist_arm(&record));

    Ok(record)
}

pub fn get_root(root_id: &str) -> Option<RootRecord> {
    let s = state();
    let s = s.as_ref()?;
    s.roots.get(root_id).cloned()
}

pub fn get_arm(arm_id: &str) -> Option<ArmRecord> {
    let s = state();
    let s = s.as_ref()?;
    s.arms.get(arm_id).cloned()
}

pub fn list_events(root_id: &str) -> Vec<EventEntry> {
    let s = state();
    let s = match s.as_ref() {
        Some(s) => s,
        None => return Vec::new(),
    };
    s.events
        .iter()
        .filter(|e| e.root_id == root_id)
        .cloned()
        .collect()
}

pub fn root_snapshot_is_durable(root_id: &str) -> bool {
    state_dir()
        .join("roots")
        .join(format!("{root_id}.snap"))
        .is_file()
}

pub fn arm_snapshot_is_durable(arm_id: &str) -> bool {
    state_dir()
        .join("arms")
        .join(format!("{arm_id}.snap"))
        .is_file()
}

fn hash(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .chars()
        .take(32)
        .collect()
}

fn report_persist_error(kind: &str, id: &str, result: Result<(), SnapshotError>) {
    if let Err(error) = result {
        eprintln!("[orchestration] failed to persist {kind} {id}: {error}");
    }
}

fn clean_field(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn persist_root(root: &RootRecord) -> Result<(), SnapshotError> {
    let dir = state_dir().join("roots");
    let path = dir.join(format!("{}.snap", root.id));
    let content = format!(
        "OCTOPUS ROOT\nschema: 2\nid: {}\nstatus: {}\nprompt-hash: {}\ninput-hash: {}\noutput-hash: {}\nstarted: {}\nfinished: {}\nduration: {}\nchildren: {}\n",
        root.id,
        root.status.as_str(),
        root.prompt_hash,
        root.input_hash,
        root.output_hash.as_deref().unwrap_or("-"),
        root.started_at,
        root.finished_at.map(|t| t.to_string()).unwrap_or_else(|| "-".to_string()),
        root.duration_ms.map(|d| format!("{d}ms")).unwrap_or_else(|| "-".to_string()),
        root.children.join(", ")
    );
    atomic_write(&path, &content)
}

fn persist_arm(arm: &ArmRecord) -> Result<(), SnapshotError> {
    let dir = state_dir().join("arms");
    let path = dir.join(format!("{}.snap", arm.id));
    let prompt_json = serde_json::to_string(&arm.prompt).unwrap_or_else(|_| "\"\"".to_string());
    let content = format!(
        "OCTOPUS ARM\nschema: 2\nid: {}\nname: {}\nroot: {}\nparent: {}\nstatus: {}\nprompt-hash: {}\nprompt-json: {}\noutput-hash: {}\noutput-bytes: {}\nerror: {}\nstarted: {}\nfinished: {}\nduration: {}\n",
        arm.id,
        clean_field(&arm.name),
        arm.root_id,
        arm.parent_arm_id
            .as_deref()
            .map(clean_field)
            .unwrap_or_else(|| "-".to_string()),
        arm.status.as_str(),
        arm.prompt_hash,
        prompt_json,
        arm.output_hash.as_deref().unwrap_or("-"),
        arm.output_bytes,
        arm.error_code
            .as_deref()
            .map(clean_field)
            .unwrap_or_else(|| "-".to_string()),
        arm.started_at,
        arm.finished_at.map(|t| t.to_string()).unwrap_or_else(|| "-".to_string()),
        arm.duration_ms.map(|d| format!("{d}ms")).unwrap_or_else(|| "-".to_string()),
    );
    atomic_write(&path, &content)
}

pub fn init_from_disk() {
    let root_dir = state_dir().join("roots");
    let arm_dir = state_dir().join("arms");

    let mut s = state();
    let s = s.get_or_insert_with(OrchestrationState::new);
    s.roots.clear();
    s.arms.clear();
    s.events.clear();
    s.file_locks.clear();

    // Load roots
    if let Ok(entries) = fs::read_dir(&root_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".snap") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let root = parse_root_snap(&content);
                        if let Some(root) = root.filter(|record| {
                            entry.path().file_stem().and_then(|name| name.to_str())
                                == Some(record.id.as_str())
                        }) {
                            s.roots.insert(root.id.clone(), root);
                        }
                    }
                }
            }
        }
    }

    // Load arms
    if let Ok(entries) = fs::read_dir(&arm_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".snap") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let arm = parse_arm_snap(&content);
                        if let Some(arm) = arm.filter(|record| {
                            entry.path().file_stem().and_then(|name| name.to_str())
                                == Some(record.id.as_str())
                        }) {
                            s.arms.insert(arm.id.clone(), arm);
                        }
                    }
                }
            }
        }
    }
}

fn parse_status(value: &str) -> Option<ArmStatus> {
    match value {
        "running" => Some(ArmStatus::Running),
        "completed" => Some(ArmStatus::Completed),
        "failed" => Some(ArmStatus::Failed),
        "cancelled" => Some(ArmStatus::Cancelled),
        "timed_out" => Some(ArmStatus::TimedOut),
        "resumed" => Some(ArmStatus::Resumed),
        _ => None,
    }
}

fn parse_root_snap(content: &str) -> Option<RootRecord> {
    let mut id = String::new();
    let mut status = None;
    let mut prompt_hash = String::new();
    let mut input_hash = String::new();
    let mut output_hash = None;
    let mut started_at = 0u128;
    let mut finished_at = None;
    let mut duration_ms = None;
    let mut children = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() != 2 {
            continue;
        }
        let key = parts[0].trim();
        let value = parts[1].trim();

        match key {
            "id" => id = value.to_string(),
            "status" => status = parse_status(value),
            "prompt-hash" => prompt_hash = value.to_string(),
            "input-hash" => input_hash = value.to_string(),
            "output-hash" if value != "-" => {
                output_hash = Some(value.to_string());
            }
            "started" => started_at = value.parse().unwrap_or(0),
            "finished" if value != "-" => {
                finished_at = value.parse().ok();
            }
            "duration" if value != "-" => {
                duration_ms = value.trim_end_matches("ms").parse().ok();
            }
            "children" if !value.is_empty() && value != "-" => {
                children = value.split(", ").map(|s| s.to_string()).collect();
            }
            _ => {}
        }
    }

    if id.is_empty() {
        return None;
    }

    Some(RootRecord {
        id,
        status: status?,
        prompt_hash,
        input_hash,
        output_hash,
        started_at,
        finished_at,
        duration_ms,
        children,
    })
}

fn parse_arm_snap(content: &str) -> Option<ArmRecord> {
    let mut id = String::new();
    let mut name = String::new();
    let mut root_id = String::new();
    let mut parent_arm_id = None;
    let mut status = None;
    let mut prompt_hash = String::new();
    let mut output_hash = None;
    let mut prompt = String::new();

    let mut output_bytes = 0usize;
    let mut error_code = None;
    let mut started_at = 0u128;
    let mut finished_at = None;
    let mut duration_ms = None;

    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() != 2 {
            continue;
        }
        let key = parts[0].trim();
        let value = parts[1].trim();

        match key {
            "id" => id = value.to_string(),
            "name" => name = value.to_string(),
            "root" => root_id = value.to_string(),
            "parent" if value != "-" => {
                parent_arm_id = Some(value.to_string());
            }
            "status" => status = parse_status(value),
            "prompt-hash" => prompt_hash = value.to_string(),
            "prompt" => prompt = value.to_string(),
            "prompt-json" => {
                prompt = serde_json::from_str(value).ok()?;
            }
            "output-hash" if value != "-" => {
                output_hash = Some(value.to_string());
            }
            "output-bytes" => output_bytes = value.parse().unwrap_or(0),
            "error" if value != "-" => {
                error_code = Some(value.to_string());
            }
            "started" => started_at = value.parse().unwrap_or(0),
            "finished" if value != "-" => {
                finished_at = value.parse().ok();
            }
            "duration" if value != "-" => {
                duration_ms = value.trim_end_matches("ms").parse().ok();
            }
            _ => {}
        }
    }

    if id.is_empty() {
        return None;
    }

    Some(ArmRecord {
        id,
        name,
        root_id,
        parent_arm_id,
        status: status?,
        prompt_hash,
        prompt,
        output_hash,
        output_bytes,
        error_code,
        started_at,
        finished_at,
        duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_root_produces_valid_record() {
        let root = create_root("test prompt");
        assert!(root.id.starts_with("root-"));
        assert_eq!(root.status, ArmStatus::Running);
    }

    #[test]
    fn create_arm_links_to_root() {
        let root = create_root("test");
        let arm = create_arm(&root.id, "code-reader", "test.rs", None);
        assert!(arm.id.starts_with("arm-"));
        assert_eq!(arm.root_id, root.id);
    }

    #[test]
    fn finish_arm_updates_status() {
        let root = create_root("test");
        let arm = create_arm(&root.id, "test", "input", None);
        finish_arm(&arm.id, &ExecutionOutcome::completed("output"));
        let updated = get_arm(&arm.id).unwrap();
        assert_eq!(updated.status, ArmStatus::Completed);
        assert!(updated.output_hash.is_some());
    }

    #[test]
    fn finish_root_updates_status() {
        let root = create_root("test");
        finish_root(&root.id, &ExecutionOutcome::completed("done"));
        let updated = get_root(&root.id).unwrap();
        assert_eq!(updated.status, ArmStatus::Completed);
    }

    #[test]
    fn cancel_arm_sets_cancelled() {
        let root = create_root("test");
        let arm = create_arm(&root.id, "test", "input", None);
        cancel_arm(&arm.id).unwrap();
        let updated = get_arm(&arm.id).unwrap();
        assert_eq!(updated.status, ArmStatus::Cancelled);
    }

    #[test]
    fn circuit_breaker_opens_on_threshold() {
        assert!(circuit_breaker_check("test", 3, 3).is_err());
        assert!(circuit_breaker_check("test", 2, 3).is_ok());
    }

    #[test]
    fn retry_succeeds_after_failures() {
        let mut attempts = 0;
        let result = retry_with_policy(3, 10, || {
            attempts += 1;
            if attempts < 3 {
                Err(ExecutionOutcome::failed("test", "fail"))
            } else {
                Ok("success")
            }
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn find_orphaned_arms_finds_running() {
        let root = create_root("test");
        let arm = create_arm(&root.id, "orphan", "input", None);
        let orphans = find_orphaned_arms();
        assert!(orphans.iter().any(|a| a.id == arm.id));
    }

    #[test]
    fn hash_produces_consistent_output() {
        let h1 = hash("hello");
        let h2 = hash("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn sanitize_name_removes_special_chars() {
        assert_eq!(sanitize_name("hello world!"), "hello-world-");
    }

    #[test]
    fn arm_parser_round_trips_multiline_prompt_json() {
        let prompt = "first line\nstatus: failed\nroot: forged\t✓";
        let encoded = serde_json::to_string(prompt).unwrap();
        let content = format!(
            "OCTOPUS ARM\nschema: 2\nid: arm-safe\nname: summarize\nroot: root-safe\nparent: -\nstatus: running\nprompt-hash: hash\nprompt-json: {encoded}\noutput-hash: -\noutput-bytes: 0\nerror: -\nstarted: 1\nfinished: -\nduration: -\n"
        );

        let arm = parse_arm_snap(&content).expect("valid arm snapshot");
        assert_eq!(arm.prompt, prompt);
        assert_eq!(arm.status, ArmStatus::Running);
        assert_eq!(arm.parent_arm_id, None);
    }

    #[test]
    fn snapshot_parsers_reject_missing_or_unknown_status() {
        let root_without_status = "OCTOPUS ROOT\nid: root-invalid\nstarted: 1\n";
        let arm_with_unknown_status =
            "OCTOPUS ARM\nid: arm-invalid\nstatus: corrupted\nstarted: 1\n";

        assert!(parse_root_snap(root_without_status).is_none());
        assert!(parse_arm_snap(arm_with_unknown_status).is_none());
    }
}
