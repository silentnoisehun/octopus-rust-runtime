use crate::orchestration::{ArmRecord, ArmStatus, RootRecord};
use crate::snapshot::{atomic_write, replace_event_log};
use crate::state_lock;
use crate::state_path::{sidecar_path, state_dir};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static BACKUP_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static RESTORE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Default)]
pub struct StateMaintenanceReport {
    pub repair: bool,
    pub roots_scanned: usize,
    pub orchestration_arms_scanned: usize,
    pub lifecycle_arms_scanned: usize,
    pub root_updates: usize,
    pub arm_updates: usize,
    pub stale_roots: usize,
    pub stale_arms: usize,
    pub invalid_snapshots: usize,
    pub orphan_arms: usize,
    pub valid_event_lines: usize,
    pub invalid_event_lines: usize,
    pub events_rewritten: bool,
    pub backup_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BackupEntry {
    relative_path: String,
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Clone)]
pub struct BackupVerificationReport {
    pub backup_id: String,
    pub sealed: bool,
    pub files: usize,
    pub bytes: u64,
    pub invalid_snapshots: usize,
    pub invalid_event_lines: usize,
}

#[derive(Debug, Clone)]
pub struct RestorePlanReport {
    pub backup_id: String,
    pub files: usize,
    pub bytes: u64,
    pub live_files: usize,
    pub live_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct RestoreReport {
    pub backup_id: String,
    pub pre_restore_backup_id: String,
    pub files: usize,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreRecoveryReport {
    None,
    RolledBack { transaction_id: String },
    Committed { transaction_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RestorePhase {
    Prepared,
    OldMoved,
    NewPublished,
}

#[derive(Debug, Clone)]
struct RestoreJournal {
    transaction_id: String,
    backup_id: String,
    pre_restore_backup_id: String,
    phase: RestorePhase,
}

impl BackupVerificationReport {
    pub fn render(&self) -> String {
        format!(
            "OCTOPUS STATE BACKUP VERIFY\nbackup: {}\nsealed: {}\nfiles: {}\nbytes: {}\ninvalid snapshots: {}\ninvalid events: {}\nintegrity: {}",
            self.backup_id,
            self.sealed,
            self.files,
            self.bytes,
            self.invalid_snapshots,
            self.invalid_event_lines,
            if self.sealed { "verified" } else { "legacy-unsealed" },
        )
    }
}

impl RestorePlanReport {
    pub fn render(&self) -> String {
        format!(
            "OCTOPUS STATE RESTORE PLAN\nbackup: {}\nsealed: true\nfiles: {}\nbytes: {}\nlive files: {}\nlive bytes: {}\npre-restore backup: required\njournal: required\nconfirmation: {}\nmutation: false",
            self.backup_id,
            self.files,
            self.bytes,
            self.live_files,
            self.live_bytes,
            self.backup_id,
        )
    }
}

impl RestoreReport {
    pub fn render(&self) -> String {
        format!(
            "OCTOPUS STATE RESTORE\nbackup: {}\npre-restore backup: {}\nfiles: {}\nbytes: {}\njournal: cleared\nintegrity: verified\nresult: restored",
            self.backup_id,
            self.pre_restore_backup_id,
            self.files,
            self.bytes,
        )
    }
}

impl RestoreRecoveryReport {
    pub fn render(&self) -> String {
        match self {
            Self::None => "OCTOPUS STATE RESTORE RECOVERY\npending: false\naction: none"
                .to_string(),
            Self::RolledBack { transaction_id } => format!(
                "OCTOPUS STATE RESTORE RECOVERY\npending: false\ntransaction: {transaction_id}\naction: rolled-back"
            ),
            Self::Committed { transaction_id } => format!(
                "OCTOPUS STATE RESTORE RECOVERY\npending: false\ntransaction: {transaction_id}\naction: committed"
            ),
        }
    }
}

impl StateMaintenanceReport {
    pub fn render(&self) -> String {
        let mode = if self.repair { "REPAIR" } else { "AUDIT" };
        format!(
            "OCTOPUS STATE {mode}\nroots: {} scanned, {} update{}\narms: {} orchestration + {} lifecycle, {} update{}\nstale: {} roots + {} arms\ninvalid snapshots: {}\norphan arms: {}\nevents: {} valid, {} invalid, rewritten={}\nbackup: {}",
            self.roots_scanned,
            self.root_updates,
            plural(self.root_updates),
            self.orchestration_arms_scanned,
            self.lifecycle_arms_scanned,
            self.arm_updates,
            plural(self.arm_updates),
            self.stale_roots,
            self.stale_arms,
            self.invalid_snapshots,
            self.orphan_arms,
            self.valid_event_lines,
            self.invalid_event_lines,
            self.events_rewritten,
            self.backup_dir
                .as_deref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "-".to_string()),
        )
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

pub fn audit(stale_after: Duration) -> Result<StateMaintenanceReport, String> {
    run_at(&state_dir(), &backup_base(), stale_after, false)
}

pub fn repair(stale_after: Duration) -> Result<StateMaintenanceReport, String> {
    run_at(&state_dir(), &backup_base(), stale_after, true)
}

pub fn create_backup() -> Result<PathBuf, String> {
    backup_state(&state_dir(), &backup_base(), now_millis())
}

pub fn verify_backup(backup_id: &str) -> Result<BackupVerificationReport, String> {
    let path = resolve_backup_id(&backup_base(), backup_id)?;
    verify_backup_at(&path)
}

pub fn plan_restore(backup_id: &str) -> Result<RestorePlanReport, String> {
    let state = state_dir();
    let backups = backup_base();
    ensure_restore_roots_disjoint(&state, &backups)?;
    let backup = resolve_backup_id(&backups, backup_id)?;
    let verification = verify_backup_at(&backup)?;
    require_restorable_backup(&verification)?;
    let live = inventory(&state)?;
    Ok(RestorePlanReport {
        backup_id: verification.backup_id,
        files: verification.files,
        bytes: verification.bytes,
        live_files: live.len(),
        live_bytes: live.iter().map(|entry| entry.bytes).sum(),
    })
}

pub fn restore_backup(backup_id: &str, confirmation: &str) -> Result<RestoreReport, String> {
    if confirmation != backup_id {
        return Err("restore confirmation must exactly match the backup id".to_string());
    }
    let state = state_dir();
    let backups = backup_base();
    let _exclusive = state_lock::acquire_exclusive(&state, state_lock::configured_timeout())?;
    let _ = recover_restore_locked(&state, &backups)?;
    restore_backup_locked(&state, &backups, backup_id)
}

pub fn recover_interrupted_restore() -> Result<RestoreRecoveryReport, String> {
    let state = state_dir();
    let journal = restore_journal_path(&state)?;
    if !journal.exists() {
        return Ok(RestoreRecoveryReport::None);
    }
    let _exclusive = state_lock::acquire_exclusive(&state, state_lock::configured_timeout())?;
    recover_restore_locked(&state, &backup_base())
}

fn backup_base() -> PathBuf {
    env::var_os("OCTOPUS_STATE_BACKUP_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"D:\codex\.octopus-rust-backups"))
}

fn require_restorable_backup(report: &BackupVerificationReport) -> Result<(), String> {
    if !report.sealed {
        return Err(format!(
            "backup {} is legacy-unsealed and cannot be restored",
            report.backup_id
        ));
    }
    if report.invalid_snapshots > 0 || report.invalid_event_lines > 0 {
        return Err(format!(
            "backup {} is not restore-safe: {} invalid snapshots, {} invalid events",
            report.backup_id, report.invalid_snapshots, report.invalid_event_lines
        ));
    }
    Ok(())
}

fn restore_backup_locked(
    state: &Path,
    backups: &Path,
    backup_id: &str,
) -> Result<RestoreReport, String> {
    ensure_plain_directory(state, "live state")?;
    ensure_restore_roots_disjoint(state, backups)?;
    let backup = resolve_backup_id(backups, backup_id)?;
    let verification = verify_backup_at(&backup)?;
    require_restorable_backup(&verification)?;

    let pre_restore_backup = backup_state(state, backups, now_millis())?;
    let pre_restore_backup_id = pre_restore_backup
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "pre-restore backup has no valid identifier".to_string())?
        .to_string();
    let transaction_id = format!(
        "{}-{}-{}",
        now_millis(),
        std::process::id(),
        RESTORE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let stage = restore_transaction_path(state, "stage", &transaction_id)?;
    let previous = restore_transaction_path(state, "previous", &transaction_id)?;
    if stage.exists() || previous.exists() {
        return Err("restore transaction paths already exist".to_string());
    }

    if let Err(error) = prepare_restore_stage(&backup, &stage) {
        let _ = safe_remove_restore_directory(state, &stage, "stage", &transaction_id);
        return Err(error);
    }

    let mut journal = RestoreJournal {
        transaction_id: transaction_id.clone(),
        backup_id: verification.backup_id.clone(),
        pre_restore_backup_id: pre_restore_backup_id.clone(),
        phase: RestorePhase::Prepared,
    };
    if let Err(error) = write_restore_journal(state, &journal) {
        let _ = safe_remove_restore_directory(state, &stage, "stage", &transaction_id);
        return Err(error);
    }

    let swap_result = (|| -> Result<(), String> {
        fs::rename(state, &previous).map_err(|error| {
            format!(
                "cannot move live state {} -> {}: {error}",
                state.display(),
                previous.display()
            )
        })?;
        journal.phase = RestorePhase::OldMoved;
        write_restore_journal(state, &journal)?;

        fs::rename(&stage, state).map_err(|error| {
            format!(
                "cannot publish restore {} -> {}: {error}",
                stage.display(),
                state.display()
            )
        })?;
        journal.phase = RestorePhase::NewPublished;
        write_restore_journal(state, &journal)?;

        if !state_matches_backup(state, &backup)? {
            return Err("published state does not match the selected backup".to_string());
        }
        safe_remove_restore_directory(state, &previous, "previous", &transaction_id)?;
        remove_restore_journal(state)?;
        Ok(())
    })();

    if let Err(error) = swap_result {
        return match recover_restore_locked(state, backups) {
            Ok(recovery) => Err(format!("{error}; recovery: {}", recovery.render())),
            Err(recovery_error) => Err(format!(
                "{error}; automatic recovery failed: {recovery_error}"
            )),
        };
    }

    Ok(RestoreReport {
        backup_id: verification.backup_id,
        pre_restore_backup_id,
        files: verification.files,
        bytes: verification.bytes,
    })
}

fn ensure_restore_roots_disjoint(state: &Path, backups: &Path) -> Result<(), String> {
    let state = state
        .canonicalize()
        .map_err(|error| format!("cannot resolve live state directory: {error}"))?;
    let backups = backups
        .canonicalize()
        .map_err(|error| format!("cannot resolve backup directory: {error}"))?;
    if backups.starts_with(&state) || state.starts_with(&backups) {
        return Err(
            "restore requires backup and live state directories to be disjoint".to_string(),
        );
    }
    Ok(())
}

fn recover_restore_locked(state: &Path, backups: &Path) -> Result<RestoreRecoveryReport, String> {
    let Some(journal) = read_restore_journal(state)? else {
        return Ok(RestoreRecoveryReport::None);
    };
    let stage = restore_transaction_path(state, "stage", &journal.transaction_id)?;
    let previous = restore_transaction_path(state, "previous", &journal.transaction_id)?;
    let state_exists = state.exists();
    let stage_exists = stage.exists();
    let previous_exists = previous.exists();

    if state_exists {
        ensure_plain_directory(state, "restore target")?;
    }
    if stage_exists {
        ensure_plain_directory(&stage, "restore stage")?;
    }
    if previous_exists {
        ensure_plain_directory(&previous, "previous state")?;
    }
    if state_exists && stage_exists && previous_exists {
        return Err("restore recovery found three competing state directories".to_string());
    }

    if !state_exists {
        if !previous_exists {
            return Err("restore recovery found no live or previous state".to_string());
        }
        validate_state_directory(&previous)?;
        fs::rename(&previous, state).map_err(|error| {
            format!(
                "cannot roll previous state {} back to {}: {error}",
                previous.display(),
                state.display()
            )
        })?;
        if stage_exists {
            safe_remove_restore_directory(state, &stage, "stage", &journal.transaction_id)?;
        }
        remove_restore_journal(state)?;
        return Ok(RestoreRecoveryReport::RolledBack {
            transaction_id: journal.transaction_id,
        });
    }

    if previous_exists {
        let selected = resolve_backup_id(backups, &journal.backup_id)
            .and_then(|path| {
                let report = verify_backup_at(&path)?;
                require_restorable_backup(&report)?;
                Ok(path)
            })
            .ok();
        let published_matches = if stage_exists {
            false
        } else if let Some(selected) = selected.as_deref() {
            state_matches_backup(state, selected).unwrap_or(false)
        } else {
            false
        };
        if published_matches {
            safe_remove_restore_directory(state, &previous, "previous", &journal.transaction_id)?;
            remove_restore_journal(state)?;
            return Ok(RestoreRecoveryReport::Committed {
                transaction_id: journal.transaction_id,
            });
        }

        if stage_exists {
            safe_remove_restore_directory(state, &stage, "stage", &journal.transaction_id)?;
        }
        let rejected = restore_transaction_path(state, "stage", &journal.transaction_id)?;
        fs::rename(state, &rejected).map_err(|error| {
            format!(
                "cannot quarantine rejected state {} -> {}: {error}",
                state.display(),
                rejected.display()
            )
        })?;
        fs::rename(&previous, state).map_err(|error| {
            format!(
                "cannot restore previous state {} -> {}: {error}",
                previous.display(),
                state.display()
            )
        })?;
        validate_state_directory(state)?;
        safe_remove_restore_directory(state, &rejected, "stage", &journal.transaction_id)?;
        remove_restore_journal(state)?;
        return Ok(RestoreRecoveryReport::RolledBack {
            transaction_id: journal.transaction_id,
        });
    }

    if stage_exists {
        safe_remove_restore_directory(state, &stage, "stage", &journal.transaction_id)?;
        remove_restore_journal(state)?;
        return Ok(RestoreRecoveryReport::RolledBack {
            transaction_id: journal.transaction_id,
        });
    }

    let selected = resolve_backup_id(backups, &journal.backup_id)?;
    let verification = verify_backup_at(&selected)?;
    require_restorable_backup(&verification)?;
    if state_matches_backup(state, &selected)? {
        remove_restore_journal(state)?;
        return Ok(RestoreRecoveryReport::Committed {
            transaction_id: journal.transaction_id,
        });
    }
    let pre_restore = resolve_backup_id(backups, &journal.pre_restore_backup_id)?;
    let pre_verification = verify_backup_at(&pre_restore)?;
    if pre_verification.sealed && state_matches_backup(state, &pre_restore)? {
        remove_restore_journal(state)?;
        return Ok(RestoreRecoveryReport::RolledBack {
            transaction_id: journal.transaction_id,
        });
    }
    Err(format!(
        "restore journal phase {} remains but live state matches neither selected nor pre-restore backup",
        journal.phase.as_str()
    ))
}

fn prepare_restore_stage(backup: &Path, stage: &Path) -> Result<(), String> {
    fs::create_dir_all(stage.join("roots"))
        .and_then(|_| fs::create_dir_all(stage.join("arms")))
        .map_err(|error| format!("cannot create restore stage {}: {error}", stage.display()))?;
    copy_directory(&backup.join("roots"), &stage.join("roots"))?;
    copy_directory(&backup.join("arms"), &stage.join("arms"))?;
    let events = backup.join("events.log");
    if events.is_file() {
        copy_file_synced(&events, &stage.join("events.log"))?;
    }
    if inventory(backup)? != inventory(stage)? {
        return Err("restore stage does not match backup inventory".to_string());
    }
    validate_state_directory(stage)?;
    Ok(())
}

fn state_matches_backup(state: &Path, backup: &Path) -> Result<bool, String> {
    let state_inventory = inventory(state)?;
    if state_inventory != inventory(backup)? {
        return Ok(false);
    }
    validate_state_directory(state)?;
    Ok(true)
}

fn validate_state_directory(path: &Path) -> Result<(), String> {
    let entries = inventory(path)?;
    let (invalid_snapshots, _) = validate_backup_payload(path, &entries)?;
    if invalid_snapshots > 0 {
        return Err(format!(
            "state directory contains {invalid_snapshots} invalid snapshots"
        ));
    }
    Ok(())
}

fn ensure_plain_directory(path: &Path, label: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {label} {}: {error}", path.display()))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(format!(
            "{label} is not a plain directory: {}",
            path.display()
        ));
    }
    Ok(())
}

fn restore_journal_path(state: &Path) -> Result<PathBuf, String> {
    sidecar_path(state, "restore-journal")
}

fn restore_transaction_path(
    state: &Path,
    kind: &str,
    transaction_id: &str,
) -> Result<PathBuf, String> {
    if !matches!(kind, "stage" | "previous")
        || transaction_id.is_empty()
        || !transaction_id
            .chars()
            .all(|character| character.is_ascii_digit() || character == '-')
    {
        return Err("invalid restore transaction path".to_string());
    }
    sidecar_path(state, &format!("restore-{kind}-{transaction_id}"))
}

fn write_restore_journal(state: &Path, journal: &RestoreJournal) -> Result<(), String> {
    let content = format!(
        "OCTOPUS STATE RESTORE JOURNAL\nschema: 1\ntransaction: {}\nbackup: {}\npre-restore-backup: {}\nphase: {}\nupdated: {}\n",
        journal.transaction_id,
        journal.backup_id,
        journal.pre_restore_backup_id,
        journal.phase.as_str(),
        now_millis(),
    );
    let path = restore_journal_path(state)?;
    atomic_write(&path, &content)
        .map_err(|error| format!("cannot write restore journal {}: {error}", path.display()))
}

fn read_restore_journal(state: &Path) -> Result<Option<RestoreJournal>, String> {
    let path = restore_journal_path(state)?;
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "cannot read restore journal {}: {error}",
                path.display()
            ));
        }
    };
    if !content.starts_with("OCTOPUS STATE RESTORE JOURNAL")
        || first_field(&content, "schema").as_deref() != Some("1")
    {
        return Err("restore journal header or schema is invalid".to_string());
    }
    let transaction_id = first_field(&content, "transaction")
        .ok_or_else(|| "restore journal is missing transaction id".to_string())?;
    let phase = first_field(&content, "phase")
        .and_then(|value| RestorePhase::parse(&value))
        .ok_or_else(|| "restore journal phase is invalid".to_string())?;
    let backup_id = first_field(&content, "backup")
        .ok_or_else(|| "restore journal is missing backup id".to_string())?;
    let pre_restore_backup_id = first_field(&content, "pre-restore-backup")
        .ok_or_else(|| "restore journal is missing pre-restore backup id".to_string())?;
    let _ = restore_transaction_path(state, "stage", &transaction_id)?;
    validate_backup_identifier(&backup_id)?;
    validate_backup_identifier(&pre_restore_backup_id)?;
    Ok(Some(RestoreJournal {
        transaction_id,
        backup_id,
        pre_restore_backup_id,
        phase,
    }))
}

fn remove_restore_journal(state: &Path) -> Result<(), String> {
    let path = restore_journal_path(state)?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "cannot remove restore journal {}: {error}",
            path.display()
        )),
    }
}

fn safe_remove_restore_directory(
    state: &Path,
    path: &Path,
    kind: &str,
    transaction_id: &str,
) -> Result<(), String> {
    let expected = restore_transaction_path(state, kind, transaction_id)?;
    if path != expected {
        return Err(format!(
            "refusing to remove unmanaged path: {}",
            path.display()
        ));
    }
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("cannot remove {}: {error}", path.display())),
    }
}

impl RestorePhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::OldMoved => "old-moved",
            Self::NewPublished => "new-published",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "prepared" => Some(Self::Prepared),
            "old-moved" => Some(Self::OldMoved),
            "new-published" => Some(Self::NewPublished),
            _ => None,
        }
    }
}

fn run_at(
    state: &Path,
    backups: &Path,
    stale_after: Duration,
    repair: bool,
) -> Result<StateMaintenanceReport, String> {
    let now = now_millis();
    let stale_ms = stale_after.as_millis();
    let root_files = snapshot_files(&state.join("roots"))?;
    let arm_files = snapshot_files(&state.join("arms"))?;
    let event_path = state.join("events.log");
    let event_text = match fs::read_to_string(&event_path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("cannot read {}: {error}", event_path.display())),
    };

    let mut report = StateMaintenanceReport {
        repair,
        ..StateMaintenanceReport::default()
    };
    let mut arm_updates = Vec::<(PathBuf, ArmRecord)>::new();
    let mut lifecycle_updates = Vec::<(PathBuf, String, String)>::new();
    let mut children_by_root = HashMap::<String, Vec<String>>::new();
    let mut arm_root_ids = Vec::<String>::new();

    for (path, content) in &arm_files {
        if content.starts_with("OCTOPUS ARM SNAPSHOT") {
            report.lifecycle_arms_scanned += 1;
            let Some(id) = first_field(content, "arm") else {
                report.invalid_snapshots += 1;
                continue;
            };
            let Some(status) = last_field(content, "status").and_then(|value| parse_status(&value))
            else {
                report.invalid_snapshots += 1;
                continue;
            };
            let stamp = last_field(content, "updated")
                .or_else(|| first_field(content, "created"))
                .and_then(|value| value.parse::<u128>().ok())
                .unwrap_or(0);
            if is_in_flight(&status) && is_stale(now, stamp, stale_ms) {
                report.stale_arms += 1;
                report.arm_updates += 1;
                let mut updated = content.trim_end_matches(['\r', '\n']).to_string();
                updated.push_str(&format!(
                    "\nstatus: timed_out\ncode: stale_recovered\nupdated: {now}\noutput-sha256: -\noutput-bytes: 0\n"
                ));
                lifecycle_updates.push((path.clone(), updated, id));
            }
            continue;
        }

        if !content.starts_with("OCTOPUS ARM") {
            report.invalid_snapshots += 1;
            continue;
        }
        report.orchestration_arms_scanned += 1;
        let Some(mut arm) = parse_orchestration_arm(content) else {
            report.invalid_snapshots += 1;
            continue;
        };
        if path.file_stem().and_then(|name| name.to_str()) != Some(arm.id.as_str()) {
            report.invalid_snapshots += 1;
            continue;
        }
        let schema_two = first_field(content, "schema").as_deref() == Some("2");
        let mut update = !schema_two;
        if arm.parent_arm_id.as_deref() == Some(arm.root_id.as_str()) {
            arm.parent_arm_id = None;
            update = true;
        }
        if is_in_flight(&arm.status) && is_stale(now, arm.started_at, stale_ms) {
            arm.status = ArmStatus::TimedOut;
            arm.finished_at = Some(now);
            arm.duration_ms = Some(now.saturating_sub(arm.started_at));
            arm.error_code = Some("stale_recovered".to_string());
            report.stale_arms += 1;
            update = true;
        }
        children_by_root
            .entry(arm.root_id.clone())
            .or_default()
            .push(arm.id.clone());
        arm_root_ids.push(arm.root_id.clone());
        if update {
            report.arm_updates += 1;
            arm_updates.push((path.clone(), arm));
        }
    }

    for children in children_by_root.values_mut() {
        children.sort();
        children.dedup();
    }

    let mut root_updates = Vec::<(PathBuf, RootRecord)>::new();
    let mut root_ids = HashSet::<String>::new();
    for (path, content) in &root_files {
        report.roots_scanned += 1;
        let Some(mut root) = parse_root(content) else {
            report.invalid_snapshots += 1;
            continue;
        };
        if path.file_stem().and_then(|name| name.to_str()) != Some(root.id.as_str()) {
            report.invalid_snapshots += 1;
            continue;
        }
        root_ids.insert(root.id.clone());
        let schema_two = first_field(content, "schema").as_deref() == Some("2");
        let mut update = !schema_two;
        if let Some(discovered) = children_by_root.get(&root.id) {
            let before = root.children.clone();
            root.children.extend(discovered.iter().cloned());
            root.children.sort();
            root.children.dedup();
            update |= root.children != before;
        }
        if is_in_flight(&root.status) && is_stale(now, root.started_at, stale_ms) {
            root.status = ArmStatus::TimedOut;
            root.finished_at = Some(now);
            root.duration_ms = Some(now.saturating_sub(root.started_at));
            report.stale_roots += 1;
            update = true;
        }
        if update {
            report.root_updates += 1;
            root_updates.push((path.clone(), root));
        }
    }
    report.orphan_arms = arm_root_ids
        .iter()
        .filter(|root_id| !root_ids.contains(root_id.as_str()))
        .count();

    let mut valid_events = Vec::new();
    for line in event_text.lines().filter(|line| !line.is_empty()) {
        if valid_event_line(line) {
            valid_events.push(line.to_string());
        } else {
            report.invalid_event_lines += 1;
        }
    }
    report.valid_event_lines = valid_events.len();
    for (_, _, id) in &lifecycle_updates {
        valid_events.push(format!("{now}\t{id}\tfailed\tstate-repair:timed_out"));
    }
    let event_rewrite_needed = report.invalid_event_lines > 0 || !lifecycle_updates.is_empty();
    let mutation_count = root_updates.len()
        + arm_updates.len()
        + lifecycle_updates.len()
        + usize::from(event_rewrite_needed);

    if repair && mutation_count > 0 {
        let backup = backup_state(state, backups, now)?;
        report.backup_dir = Some(backup);
        for (path, root) in root_updates {
            atomic_write(&path, &serialize_root(&root))
                .map_err(|error| format!("cannot update {}: {error}", path.display()))?;
        }
        for (path, arm) in arm_updates {
            atomic_write(&path, &serialize_arm(&arm))
                .map_err(|error| format!("cannot update {}: {error}", path.display()))?;
        }
        for (path, content, _) in lifecycle_updates {
            atomic_write(&path, &content)
                .map_err(|error| format!("cannot update {}: {error}", path.display()))?;
        }
        if event_rewrite_needed {
            let mut content = valid_events.join("\n");
            if !content.is_empty() {
                content.push('\n');
            }
            replace_event_log(state, &content)
                .map_err(|error| format!("cannot rewrite events.log: {error}"))?;
            report.events_rewritten = true;
        }
    }

    Ok(report)
}

fn snapshot_files(dir: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("cannot read {}: {error}", dir.display())),
    };
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("snap") {
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(content) => files.push((path, content)),
            Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
        }
    }
    Ok(files)
}

fn backup_state(state: &Path, backups: &Path, now: u128) -> Result<PathBuf, String> {
    fs::create_dir_all(backups)
        .map_err(|error| format!("cannot create backup root {}: {error}", backups.display()))?;
    let name = format!(
        "state-{now}-{}-{}",
        std::process::id(),
        BACKUP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let target = backups.join(&name);
    let partial = backups.join(format!(".partial-{name}"));
    if target.exists() {
        return Err(format!("backup already exists: {}", target.display()));
    }
    if partial.exists() {
        return Err(format!(
            "partial backup already exists: {}",
            partial.display()
        ));
    }

    let result = (|| -> Result<(), String> {
        fs::create_dir_all(partial.join("roots"))
            .and_then(|_| fs::create_dir_all(partial.join("arms")))
            .map_err(|error| format!("cannot create backup {}: {error}", partial.display()))?;
        copy_directory(&state.join("roots"), &partial.join("roots"))?;
        copy_directory(&state.join("arms"), &partial.join("arms"))?;
        let events = state.join("events.log");
        if events.is_file() {
            fs::copy(&events, partial.join("events.log"))
                .map_err(|error| format!("cannot back up {}: {error}", events.display()))?;
        }
        let entries = inventory(&partial)?;
        let manifest = render_manifest(now, state, &entries);
        atomic_write(&partial.join("manifest.txt"), &manifest)
            .map_err(|error| format!("cannot seal backup manifest: {error}"))?;
        let verification = verify_backup_at(&partial)?;
        if !verification.sealed {
            return Err("new backup did not verify as sealed".to_string());
        }
        fs::rename(&partial, &target).map_err(|error| {
            format!(
                "cannot publish backup {} -> {}: {error}",
                partial.display(),
                target.display()
            )
        })?;
        Ok(())
    })();

    if let Err(error) = result {
        let _ = fs::remove_dir_all(&partial);
        return Err(error);
    }
    Ok(target)
}

fn resolve_backup_id(backups: &Path, backup_id: &str) -> Result<PathBuf, String> {
    validate_backup_identifier(backup_id)?;
    let path = backups.join(backup_id);
    if !path.is_dir() {
        return Err(format!("backup not found: {backup_id}"));
    }
    let canonical_root = backups
        .canonicalize()
        .map_err(|error| format!("cannot resolve backup root: {error}"))?;
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("cannot resolve backup {backup_id}: {error}"))?;
    if canonical.parent() != Some(canonical_root.as_path()) {
        return Err("backup escaped the configured backup root".to_string());
    }
    Ok(canonical)
}

fn validate_backup_identifier(backup_id: &str) -> Result<(), String> {
    if !backup_id.starts_with("state-")
        || backup_id.is_empty()
        || !backup_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err("backup id must be a direct state-* identifier".to_string());
    }
    Ok(())
}

fn render_manifest(created: u128, source: &Path, entries: &[BackupEntry]) -> String {
    let bytes = entries.iter().map(|entry| entry.bytes).sum::<u64>();
    let inventory_hash = inventory_hash(entries);
    let mut manifest = format!(
        "OCTOPUS STATE BACKUP\nschema: 2\ncomplete: true\ncreated: {created}\nsource: {}\nfiles: {}\nbytes: {bytes}\ninventory-sha256: {inventory_hash}\n",
        clean_field(&source.display().to_string()),
        entries.len(),
    );
    for entry in entries {
        manifest.push_str(&format!(
            "file: {}\t{}\t{}\n",
            entry.relative_path, entry.bytes, entry.sha256
        ));
    }
    manifest
}

fn inventory_hash(entries: &[BackupEntry]) -> String {
    let mut canonical = String::new();
    for entry in entries {
        canonical.push_str(&format!(
            "{}\t{}\t{}\n",
            entry.relative_path, entry.bytes, entry.sha256
        ));
    }
    hex::encode(Sha256::digest(canonical.as_bytes()))
}

fn inventory(root: &Path) -> Result<Vec<BackupEntry>, String> {
    let mut entries = Vec::new();
    collect_inventory_directory(root, "roots", &mut entries)?;
    collect_inventory_directory(root, "arms", &mut entries)?;
    let events = root.join("events.log");
    if events.exists() {
        entries.push(inventory_entry(root, &events)?);
    }
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let mut names = HashSet::new();
    for entry in &entries {
        let folded = entry.relative_path.to_ascii_lowercase();
        if !names.insert(folded) {
            return Err(format!("duplicate backup path: {}", entry.relative_path));
        }
    }
    Ok(entries)
}

fn collect_inventory_directory(
    root: &Path,
    name: &str,
    output: &mut Vec<BackupEntry>,
) -> Result<(), String> {
    let directory = root.join(name);
    let metadata = fs::symlink_metadata(&directory)
        .map_err(|error| format!("missing backup directory {}: {error}", directory.display()))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(format!("unsafe backup directory: {}", directory.display()));
    }
    for item in fs::read_dir(&directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
    {
        let item = item.map_err(|error| format!("cannot read backup entry: {error}"))?;
        let path = item.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(format!("unexpected backup entry: {}", path.display()));
        }
        if path.extension().and_then(|value| value.to_str()) != Some("snap") {
            return Err(format!("unexpected backup payload: {}", path.display()));
        }
        output.push(inventory_entry(root, &path)?);
    }
    Ok(())
}

fn inventory_entry(root: &Path, path: &Path) -> Result<BackupEntry, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("cannot hash {}: {error}", path.display()))?;
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("backup path escaped root: {}", path.display()))?
        .to_string_lossy()
        .replace('\\', "/");
    if relative.starts_with('/')
        || relative
            .split('/')
            .any(|part| part == ".." || part.is_empty())
    {
        return Err(format!("unsafe backup path: {relative}"));
    }
    Ok(BackupEntry {
        relative_path: relative,
        bytes: bytes.len() as u64,
        sha256: hex::encode(Sha256::digest(&bytes)),
    })
}

fn verify_backup_at(path: &Path) -> Result<BackupVerificationReport, String> {
    let manifest_path = path.join("manifest.txt");
    let manifest = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
    if !manifest.starts_with("OCTOPUS STATE BACKUP") {
        return Err("backup manifest header is invalid".to_string());
    }
    let actual = inventory(path)?;
    let sealed = first_field(&manifest, "schema").as_deref() == Some("2")
        && first_field(&manifest, "complete").as_deref() == Some("true");
    if sealed {
        let expected = parse_manifest_entries(&manifest)?;
        let expected_files = first_field(&manifest, "files")
            .and_then(|value| value.parse::<usize>().ok())
            .ok_or_else(|| "sealed manifest has invalid file count".to_string())?;
        let expected_bytes = first_field(&manifest, "bytes")
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or_else(|| "sealed manifest has invalid byte count".to_string())?;
        let expected_inventory_hash = first_field(&manifest, "inventory-sha256")
            .ok_or_else(|| "sealed manifest is missing inventory hash".to_string())?;
        if expected_files != expected.len()
            || expected_bytes != expected.iter().map(|entry| entry.bytes).sum::<u64>()
            || expected_inventory_hash != inventory_hash(&expected)
            || expected != actual
        {
            return Err("backup manifest does not match payload".to_string());
        }
    }
    let (invalid_snapshots, invalid_event_lines) = validate_backup_payload(path, &actual)?;
    Ok(BackupVerificationReport {
        backup_id: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("backup")
            .to_string(),
        sealed,
        files: actual.len(),
        bytes: actual.iter().map(|entry| entry.bytes).sum(),
        invalid_snapshots,
        invalid_event_lines,
    })
}

fn parse_manifest_entries(manifest: &str) -> Result<Vec<BackupEntry>, String> {
    let mut entries = Vec::new();
    for line in manifest
        .lines()
        .filter_map(|line| line.strip_prefix("file: "))
    {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 3
            || (!matches!(fields[0].split('/').next(), Some("roots" | "arms"))
                && fields[0] != "events.log")
            || fields[0]
                .split('/')
                .any(|part| part == ".." || part.is_empty())
        {
            return Err(format!("invalid manifest entry: {line}"));
        }
        entries.push(BackupEntry {
            relative_path: fields[0].to_string(),
            bytes: fields[1]
                .parse()
                .map_err(|_| format!("invalid manifest byte count: {line}"))?,
            sha256: fields[2].to_string(),
        });
    }
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(entries)
}

fn validate_backup_payload(path: &Path, entries: &[BackupEntry]) -> Result<(usize, usize), String> {
    let mut invalid_snapshots = 0usize;
    let mut invalid_event_lines = 0usize;
    for entry in entries {
        let file = entry
            .relative_path
            .split('/')
            .fold(path.to_path_buf(), |current, component| {
                current.join(component)
            });
        if entry.relative_path == "events.log" {
            let content = fs::read_to_string(&file)
                .map_err(|error| format!("cannot read {}: {error}", file.display()))?;
            invalid_event_lines = content
                .lines()
                .filter(|line| !line.is_empty() && !valid_event_line(line))
                .count();
            continue;
        }
        let content = fs::read_to_string(&file)
            .map_err(|error| format!("cannot read {}: {error}", file.display()))?;
        let stem = file.file_stem().and_then(|value| value.to_str());
        let valid = if entry.relative_path.starts_with("roots/") {
            content.starts_with("OCTOPUS ROOT") && first_field(&content, "id").as_deref() == stem
        } else if content.starts_with("OCTOPUS ARM SNAPSHOT") {
            first_field(&content, "arm").as_deref() == stem
        } else {
            content.starts_with("OCTOPUS ARM") && first_field(&content, "id").as_deref() == stem
        };
        if !valid {
            invalid_snapshots += 1;
        }
    }
    if invalid_snapshots > 0 {
        return Err(format!(
            "backup contains {invalid_snapshots} invalid snapshots"
        ));
    }
    Ok((invalid_snapshots, invalid_event_lines))
}

fn copy_directory(source: &Path, target: &Path) -> Result<(), String> {
    let entries = match fs::read_dir(source) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("cannot read {}: {error}", source.display())),
    };
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "cannot read directory entry in {}: {error}",
                source.display()
            )
        })?;
        let path = entry.path();
        if path.is_file() {
            copy_file_synced(&path, &target.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn copy_file_synced(source: &Path, target: &Path) -> Result<(), String> {
    fs::copy(source, target)
        .map_err(|error| format!("cannot copy {}: {error}", source.display()))?;
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(target)
        .and_then(|file| file.sync_all())
        .map_err(|error| format!("cannot sync {}: {error}", target.display()))
}

fn parse_root(content: &str) -> Option<RootRecord> {
    let id = first_field(content, "id")?;
    let status = first_field(content, "status").and_then(|value| parse_status(&value))?;
    let children = first_field(content, "children")
        .filter(|value| value != "-")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Some(RootRecord {
        id,
        status,
        prompt_hash: first_field(content, "prompt-hash").unwrap_or_default(),
        input_hash: first_field(content, "input-hash").unwrap_or_default(),
        output_hash: first_field(content, "output-hash").filter(|value| value != "-"),
        started_at: first_field(content, "started")?.parse().ok()?,
        finished_at: first_field(content, "finished")
            .filter(|value| value != "-")
            .and_then(|value| value.parse().ok()),
        duration_ms: first_field(content, "duration")
            .filter(|value| value != "-")
            .and_then(|value| value.trim_end_matches("ms").parse().ok()),
        children,
    })
}

fn parse_orchestration_arm(content: &str) -> Option<ArmRecord> {
    let id = first_field(content, "id")?;
    let root_id = first_field(content, "root")?;
    let status = first_field(content, "status").and_then(|value| parse_status(&value))?;
    let prompt = if let Some(encoded) = first_field(content, "prompt-json") {
        serde_json::from_str(&encoded).ok()?
    } else {
        first_field(content, "prompt").unwrap_or_default()
    };
    Some(ArmRecord {
        id,
        name: first_field(content, "name")?,
        root_id,
        parent_arm_id: first_field(content, "parent").filter(|value| value != "-"),
        status,
        prompt_hash: first_field(content, "prompt-hash").unwrap_or_default(),
        prompt,
        output_hash: last_field(content, "output-hash").filter(|value| value != "-"),
        output_bytes: last_field(content, "output-bytes")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0),
        error_code: last_field(content, "error").filter(|value| value != "-"),
        started_at: last_field(content, "started")?.parse().ok()?,
        finished_at: last_field(content, "finished")
            .filter(|value| value != "-")
            .and_then(|value| value.parse().ok()),
        duration_ms: last_field(content, "duration")
            .filter(|value| value != "-")
            .and_then(|value| value.trim_end_matches("ms").parse().ok()),
    })
}

fn serialize_root(root: &RootRecord) -> String {
    format!(
        "OCTOPUS ROOT\nschema: 2\nid: {}\nstatus: {}\nprompt-hash: {}\ninput-hash: {}\noutput-hash: {}\nstarted: {}\nfinished: {}\nduration: {}\nchildren: {}\n",
        root.id,
        root.status.as_str(),
        root.prompt_hash,
        root.input_hash,
        root.output_hash.as_deref().unwrap_or("-"),
        root.started_at,
        root.finished_at.map(|value| value.to_string()).unwrap_or_else(|| "-".to_string()),
        root.duration_ms.map(|value| format!("{value}ms")).unwrap_or_else(|| "-".to_string()),
        if root.children.is_empty() { "-".to_string() } else { root.children.join(", ") },
    )
}

fn serialize_arm(arm: &ArmRecord) -> String {
    let prompt_json = serde_json::to_string(&arm.prompt).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        "OCTOPUS ARM\nschema: 2\nid: {}\nname: {}\nroot: {}\nparent: {}\nstatus: {}\nprompt-hash: {}\nprompt-json: {}\noutput-hash: {}\noutput-bytes: {}\nerror: {}\nstarted: {}\nfinished: {}\nduration: {}\n",
        arm.id,
        clean_field(&arm.name),
        arm.root_id,
        arm.parent_arm_id.as_deref().map(clean_field).unwrap_or_else(|| "-".to_string()),
        arm.status.as_str(),
        arm.prompt_hash,
        prompt_json,
        arm.output_hash.as_deref().unwrap_or("-"),
        arm.output_bytes,
        arm.error_code.as_deref().map(clean_field).unwrap_or_else(|| "-".to_string()),
        arm.started_at,
        arm.finished_at.map(|value| value.to_string()).unwrap_or_else(|| "-".to_string()),
        arm.duration_ms.map(|value| format!("{value}ms")).unwrap_or_else(|| "-".to_string()),
    )
}

fn first_field(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}: ");
    content.lines().find_map(|line| {
        line.strip_prefix(&prefix)
            .map(str::trim)
            .map(str::to_string)
    })
}

fn last_field(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}: ");
    content
        .lines()
        .filter_map(|line| {
            line.strip_prefix(&prefix)
                .map(str::trim)
                .map(str::to_string)
        })
        .next_back()
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

fn is_in_flight(status: &ArmStatus) -> bool {
    matches!(status, ArmStatus::Running | ArmStatus::Resumed)
}

fn is_stale(now: u128, stamp: u128, threshold: u128) -> bool {
    stamp == 0 || now.saturating_sub(stamp) >= threshold
}

fn valid_event_line(line: &str) -> bool {
    let fields = line.split('\t').collect::<Vec<_>>();
    fields.len() == 4
        && fields[0].parse::<u128>().is_ok()
        && !fields[1].is_empty()
        && matches!(fields[2], "running" | "completed" | "failed" | "cancelled")
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

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    fn fixture() -> (PathBuf, PathBuf) {
        let base = env::temp_dir().join(format!(
            "octopus-maintenance-{}-{}",
            std::process::id(),
            TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let state = base.join("state");
        let backup = base.join("backups");
        fs::create_dir_all(state.join("roots")).unwrap();
        fs::create_dir_all(state.join("arms")).unwrap();
        let old = now_millis() - Duration::from_secs(48 * 60 * 60).as_millis();
        fs::write(
            state.join("roots/root-old.snap"),
            format!("OCTOPUS ROOT\nid: root-old\nstatus: running\nprompt-hash: h\ninput-hash: h\noutput-hash: -\nstarted: {old}\nfinished: -\nduration: -\nchildren: -\n"),
        )
        .unwrap();
        fs::write(
            state.join("arms/arm-old.snap"),
            format!("OCTOPUS ARM\nid: arm-old\nname: summarize\nroot: root-old\nparent: root-old\nstatus: running\nprompt-hash: h\nprompt: first line\nstatus: failed\nroot: forged\noutput-hash: -\noutput-bytes: 0\nerror: -\nstarted: {old}\nfinished: -\nduration: -\n"),
        )
        .unwrap();
        fs::write(
            state.join("arms/lifecycle-old.snap"),
            format!("OCTOPUS ARM SNAPSHOT\narm: lifecycle-old\nname: summarize\nstatus: running\ncreated: {old}\nparent: -\nprompt-sha256: h\n\n"),
        )
        .unwrap();
        fs::write(
            state.join("events.log"),
            format!("{old}\tlifecycle-old\trunning\tsummarize\nbroken event line\n"),
        )
        .unwrap();
        (state, backup)
    }

    fn clean_fixture() -> (PathBuf, PathBuf) {
        let (state, backups) = fixture();
        let valid = fs::read_to_string(state.join("events.log"))
            .unwrap()
            .lines()
            .find(|line| valid_event_line(line))
            .unwrap()
            .to_string();
        fs::write(state.join("events.log"), format!("{valid}\n")).unwrap();
        (state, backups)
    }

    fn backup_id(path: &Path) -> String {
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap()
            .to_string()
    }

    #[test]
    fn audit_is_read_only_and_reports_planned_repairs() {
        let (state, backups) = fixture();
        let before = fs::read_to_string(state.join("events.log")).unwrap();
        let report = run_at(&state, &backups, Duration::from_secs(24 * 60 * 60), false).unwrap();
        assert_eq!(report.stale_roots, 1);
        assert_eq!(report.stale_arms, 2);
        assert_eq!(report.invalid_event_lines, 1);
        assert!(report.backup_dir.is_none());
        assert_eq!(
            fs::read_to_string(state.join("events.log")).unwrap(),
            before
        );
        let _ = fs::remove_dir_all(state.parent().unwrap());
    }

    #[test]
    fn repair_backs_up_and_normalizes_legacy_state() {
        let (state, backups) = fixture();
        let report = run_at(&state, &backups, Duration::from_secs(24 * 60 * 60), true).unwrap();
        let backup_dir = report.backup_dir.as_ref().expect("repair backup");
        assert!(backup_dir.is_dir());
        let verification = verify_backup_at(backup_dir).unwrap();
        assert!(verification.sealed);
        assert_eq!(verification.invalid_snapshots, 0);
        let manifest = fs::read_to_string(backup_dir.join("manifest.txt")).unwrap();
        assert!(manifest.contains("schema: 2\n"));
        assert!(manifest.contains("complete: true\n"));
        assert!(manifest.contains("inventory-sha256: "));
        assert!(report.events_rewritten);
        let root = fs::read_to_string(state.join("roots/root-old.snap")).unwrap();
        let arm = fs::read_to_string(state.join("arms/arm-old.snap")).unwrap();
        let lifecycle = fs::read_to_string(state.join("arms/lifecycle-old.snap")).unwrap();
        let events = fs::read_to_string(state.join("events.log")).unwrap();
        assert!(root.contains("schema: 2\n"));
        assert!(root.contains("status: timed_out\n"));
        assert!(root.contains("children: arm-old\n"));
        assert!(arm.contains("schema: 2\n"));
        assert!(arm.contains("parent: -\n"));
        assert!(arm.contains("prompt-json: \"first line\"\n"));
        assert!(arm.contains("status: timed_out\n"));
        assert!(lifecycle.contains("status: timed_out\n"));
        assert!(!events.contains("broken event line"));
        assert!(events.contains("state-repair:timed_out"));
        let _ = fs::remove_dir_all(state.parent().unwrap());
    }

    #[test]
    fn sealed_backup_detects_payload_corruption() {
        let (state, backups) = fixture();
        let backup = backup_state(&state, &backups, now_millis()).unwrap();
        let verification = verify_backup_at(&backup).unwrap();
        assert!(verification.sealed);

        fs::write(
            backup.join("roots/root-old.snap"),
            "OCTOPUS ROOT\nid: root-old\nstatus: completed\n",
        )
        .unwrap();
        let error = verify_backup_at(&backup).unwrap_err();
        assert!(error.contains("manifest does not match payload"));
        let _ = fs::remove_dir_all(state.parent().unwrap());
    }

    #[test]
    fn legacy_backup_is_readable_but_reported_unsealed() {
        let (state, backups) = fixture();
        let backup = backups.join("state-legacy-1");
        fs::create_dir_all(backup.join("roots")).unwrap();
        fs::create_dir_all(backup.join("arms")).unwrap();
        copy_directory(&state.join("roots"), &backup.join("roots")).unwrap();
        copy_directory(&state.join("arms"), &backup.join("arms")).unwrap();
        fs::copy(state.join("events.log"), backup.join("events.log")).unwrap();
        fs::write(
            backup.join("manifest.txt"),
            "OCTOPUS STATE BACKUP\ncreated: 1\nsource: legacy\n",
        )
        .unwrap();

        let verification = verify_backup_at(&backup).unwrap();
        assert!(!verification.sealed);
        assert_eq!(verification.invalid_snapshots, 0);
        assert_eq!(verification.invalid_event_lines, 1);
        assert_eq!(
            verification.render().lines().last(),
            Some("integrity: legacy-unsealed")
        );
        let _ = fs::remove_dir_all(state.parent().unwrap());
    }

    #[test]
    fn restore_replaces_state_and_preserves_a_sealed_pre_restore_backup() {
        let (state, backups) = clean_fixture();
        let selected = backup_state(&state, &backups, now_millis()).unwrap();
        let selected_id = backup_id(&selected);
        fs::write(
            state.join("roots/root-extra.snap"),
            "OCTOPUS ROOT\nid: root-extra\nstatus: completed\nprompt-hash: h\ninput-hash: h\noutput-hash: h\nstarted: 1\nfinished: 2\nduration: 1ms\nchildren: -\n",
        )
        .unwrap();
        assert_ne!(inventory(&state).unwrap(), inventory(&selected).unwrap());

        let report = restore_backup_locked(&state, &backups, &selected_id).unwrap();
        assert_eq!(report.backup_id, selected_id);
        assert!(state_matches_backup(&state, &selected).unwrap());
        let pre_restore = resolve_backup_id(&backups, &report.pre_restore_backup_id).unwrap();
        let pre_verification = verify_backup_at(&pre_restore).unwrap();
        assert!(pre_verification.sealed);
        assert!(pre_restore.join("roots/root-extra.snap").is_file());
        assert!(!restore_journal_path(&state).unwrap().exists());
        assert!(
            !fs::read_dir(state.parent().unwrap()).unwrap().any(|entry| {
                entry
                    .ok()
                    .and_then(|item| item.file_name().to_str().map(str::to_string))
                    .is_some_and(|name| {
                        name.contains(".restore-stage-") || name.contains(".restore-previous-")
                    })
            })
        );
        let _ = fs::remove_dir_all(state.parent().unwrap());
    }

    #[test]
    fn recovery_rolls_back_when_live_state_was_moved_but_not_published() {
        let (state, backups) = clean_fixture();
        let selected = backup_state(&state, &backups, now_millis()).unwrap();
        let transaction_id = "100-200-300";
        let stage = restore_transaction_path(&state, "stage", transaction_id).unwrap();
        let previous = restore_transaction_path(&state, "previous", transaction_id).unwrap();
        prepare_restore_stage(&selected, &stage).unwrap();
        write_restore_journal(
            &state,
            &RestoreJournal {
                transaction_id: transaction_id.to_string(),
                backup_id: backup_id(&selected),
                pre_restore_backup_id: backup_id(&selected),
                phase: RestorePhase::Prepared,
            },
        )
        .unwrap();
        fs::rename(&state, &previous).unwrap();

        let recovery = recover_restore_locked(&state, &backups).unwrap();
        assert_eq!(
            recovery,
            RestoreRecoveryReport::RolledBack {
                transaction_id: transaction_id.to_string()
            }
        );
        assert!(state.is_dir());
        assert!(!stage.exists());
        assert!(!previous.exists());
        assert!(!restore_journal_path(&state).unwrap().exists());
        let _ = fs::remove_dir_all(state.parent().unwrap());
    }

    #[test]
    fn recovery_commits_a_fully_published_verified_candidate() {
        let (state, backups) = clean_fixture();
        let selected = backup_state(&state, &backups, now_millis()).unwrap();
        fs::write(
            state.join("roots/root-extra.snap"),
            "OCTOPUS ROOT\nid: root-extra\nstatus: completed\nprompt-hash: h\ninput-hash: h\noutput-hash: h\nstarted: 1\nfinished: 2\nduration: 1ms\nchildren: -\n",
        )
        .unwrap();
        let pre_restore = backup_state(&state, &backups, now_millis()).unwrap();
        let transaction_id = "400-500-600";
        let stage = restore_transaction_path(&state, "stage", transaction_id).unwrap();
        let previous = restore_transaction_path(&state, "previous", transaction_id).unwrap();
        prepare_restore_stage(&selected, &stage).unwrap();
        fs::rename(&state, &previous).unwrap();
        fs::rename(&stage, &state).unwrap();
        write_restore_journal(
            &state,
            &RestoreJournal {
                transaction_id: transaction_id.to_string(),
                backup_id: backup_id(&selected),
                pre_restore_backup_id: backup_id(&pre_restore),
                phase: RestorePhase::OldMoved,
            },
        )
        .unwrap();

        let recovery = recover_restore_locked(&state, &backups).unwrap();
        assert_eq!(
            recovery,
            RestoreRecoveryReport::Committed {
                transaction_id: transaction_id.to_string()
            }
        );
        assert!(state_matches_backup(&state, &selected).unwrap());
        assert!(!previous.exists());
        assert!(!restore_journal_path(&state).unwrap().exists());
        let _ = fs::remove_dir_all(state.parent().unwrap());
    }

    #[test]
    fn restore_rejects_legacy_unsealed_backup_reports() {
        let report = BackupVerificationReport {
            backup_id: "state-legacy-1".to_string(),
            sealed: false,
            files: 1,
            bytes: 1,
            invalid_snapshots: 0,
            invalid_event_lines: 0,
        };
        let error = require_restorable_backup(&report).unwrap_err();
        assert!(error.contains("legacy-unsealed"));
    }

    #[test]
    fn recovery_rolls_back_a_corrupt_published_candidate() {
        let (state, backups) = clean_fixture();
        let selected = backup_state(&state, &backups, now_millis()).unwrap();
        fs::write(
            state.join("roots/root-extra.snap"),
            "OCTOPUS ROOT\nid: root-extra\nstatus: completed\nprompt-hash: h\ninput-hash: h\noutput-hash: h\nstarted: 1\nfinished: 2\nduration: 1ms\nchildren: -\n",
        )
        .unwrap();
        let pre_restore = backup_state(&state, &backups, now_millis()).unwrap();
        let transaction_id = "700-800-900";
        let stage = restore_transaction_path(&state, "stage", transaction_id).unwrap();
        let previous = restore_transaction_path(&state, "previous", transaction_id).unwrap();
        prepare_restore_stage(&selected, &stage).unwrap();
        fs::rename(&state, &previous).unwrap();
        fs::rename(&stage, &state).unwrap();
        fs::write(state.join("roots/root-old.snap"), "corrupt\n").unwrap();
        write_restore_journal(
            &state,
            &RestoreJournal {
                transaction_id: transaction_id.to_string(),
                backup_id: backup_id(&selected),
                pre_restore_backup_id: backup_id(&pre_restore),
                phase: RestorePhase::NewPublished,
            },
        )
        .unwrap();

        let recovery = recover_restore_locked(&state, &backups).unwrap();
        assert_eq!(
            recovery,
            RestoreRecoveryReport::RolledBack {
                transaction_id: transaction_id.to_string()
            }
        );
        assert!(state.join("roots/root-extra.snap").is_file());
        assert!(!previous.exists());
        assert!(!restore_journal_path(&state).unwrap().exists());
        let _ = fs::remove_dir_all(state.parent().unwrap());
    }

    #[test]
    fn restore_rejects_backup_directory_nested_inside_live_state() {
        let (state, _) = clean_fixture();
        let nested = state.join("backups");
        fs::create_dir_all(&nested).unwrap();
        let error = ensure_restore_roots_disjoint(&state, &nested).unwrap_err();
        assert!(error.contains("disjoint"));
        let _ = fs::remove_dir_all(state.parent().unwrap());
    }

    #[test]
    fn recovery_proves_completed_rollback_from_pre_restore_backup() {
        let (state, backups) = clean_fixture();
        let selected = backup_state(&state, &backups, now_millis()).unwrap();
        fs::write(
            state.join("roots/root-extra.snap"),
            "OCTOPUS ROOT\nid: root-extra\nstatus: completed\nprompt-hash: h\ninput-hash: h\noutput-hash: h\nstarted: 1\nfinished: 2\nduration: 1ms\nchildren: -\n",
        )
        .unwrap();
        let pre_restore = backup_state(&state, &backups, now_millis()).unwrap();
        let transaction_id = "1000-1100-1200";
        write_restore_journal(
            &state,
            &RestoreJournal {
                transaction_id: transaction_id.to_string(),
                backup_id: backup_id(&selected),
                pre_restore_backup_id: backup_id(&pre_restore),
                phase: RestorePhase::OldMoved,
            },
        )
        .unwrap();

        let recovery = recover_restore_locked(&state, &backups).unwrap();
        assert_eq!(
            recovery,
            RestoreRecoveryReport::RolledBack {
                transaction_id: transaction_id.to_string()
            }
        );
        assert!(state_matches_backup(&state, &pre_restore).unwrap());
        assert!(!restore_journal_path(&state).unwrap().exists());
        let _ = fs::remove_dir_all(state.parent().unwrap());
    }
}
