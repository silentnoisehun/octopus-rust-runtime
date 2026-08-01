use crate::orchestration::{ArmRecord, ArmStatus, RootRecord};
use crate::state_path::{sidecar_path, state_dir};
use crate::{ExecutionOutcome, ExecutionStatus};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SCHEMA: u8 = 1;
const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const STALE_LOCK_AGE: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResonanceEntry {
    sequence: u64,
    timestamp: u128,
    root_id: String,
    status: String,
    arms: usize,
    completed: usize,
    failed: usize,
    other: usize,
    input_hash: String,
    output_hash: String,
    topology_hash: String,
    code: String,
    previous_hash: String,
    entry_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerificationReport {
    entries: Vec<ResonanceEntry>,
    head_hash: String,
}

struct ResonanceLock {
    path: PathBuf,
}

impl Drop for ResonanceLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn hash(value: impl AsRef<[u8]>) -> String {
    hex::encode(Sha256::digest(value.as_ref()))
}

fn configured_path() -> Result<PathBuf, String> {
    if let Some(configured) = env::var_os("OCTOPUS_RESONANCE_LOG") {
        let configured = PathBuf::from(configured);
        return if configured.is_absolute() {
            Ok(configured)
        } else {
            env::current_dir()
                .map(|directory| directory.join(configured))
                .map_err(|error| format!("cannot resolve resonance log path: {error}"))
        };
    }
    sidecar_path(&state_dir(), "resonance.log")
}

fn acquire_lock(path: &Path) -> Result<ResonanceLock, String> {
    let lock_path = path.with_extension("log.lock");
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create resonance lock directory: {error}"))?;
    }
    let started = Instant::now();
    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                writeln!(file, "pid={} created={}", std::process::id(), now_millis())
                    .map_err(|error| format!("cannot write resonance lock: {error}"))?;
                file.sync_all()
                    .map_err(|error| format!("cannot sync resonance lock: {error}"))?;
                return Ok(ResonanceLock { path: lock_path });
            }
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::AlreadyExists | ErrorKind::PermissionDenied
                ) =>
            {
                let stale = fs::metadata(&lock_path)
                    .and_then(|metadata| metadata.modified())
                    .and_then(|modified| modified.elapsed().map_err(std::io::Error::other))
                    .map(|age| age >= STALE_LOCK_AGE)
                    .unwrap_or(false);
                if stale {
                    let _ = fs::remove_file(&lock_path);
                    continue;
                }
                if started.elapsed() >= LOCK_TIMEOUT {
                    return Err("timed out waiting for resonance log lock".to_string());
                }
                thread::sleep(Duration::from_millis(2));
            }
            Err(error) => return Err(format!("cannot acquire resonance log lock: {error}")),
        }
    }
}

fn field<'a>(parts: &'a [&str], name: &str) -> Result<&'a str, String> {
    parts
        .iter()
        .find_map(|part| part.strip_prefix(&format!("{name}=")))
        .ok_or_else(|| format!("resonance entry is missing field '{name}'"))
}

fn parse_entry(line: &str) -> Result<ResonanceEntry, String> {
    let parts: Vec<_> = line.split('\t').collect();
    if parts.first() != Some(&"entry") {
        return Err("invalid resonance entry prefix".to_string());
    }
    let parse_number = |name: &str| -> Result<u64, String> {
        field(&parts, name)?
            .parse()
            .map_err(|_| format!("invalid resonance numeric field '{name}'"))
    };
    Ok(ResonanceEntry {
        sequence: parse_number("seq")?,
        timestamp: field(&parts, "ts")?
            .parse()
            .map_err(|_| "invalid resonance timestamp".to_string())?,
        root_id: field(&parts, "root")?.to_string(),
        status: field(&parts, "status")?.to_string(),
        arms: parse_number("arms")? as usize,
        completed: parse_number("completed")? as usize,
        failed: parse_number("failed")? as usize,
        other: parse_number("other")? as usize,
        input_hash: field(&parts, "input")?.to_string(),
        output_hash: field(&parts, "output")?.to_string(),
        topology_hash: field(&parts, "topology")?.to_string(),
        code: field(&parts, "code")?.to_string(),
        previous_hash: field(&parts, "previous")?.to_string(),
        entry_hash: field(&parts, "hash")?.to_string(),
    })
}

fn payload(entry: &ResonanceEntry) -> String {
    format!(
        "seq={}\tts={}\troot={}\tstatus={}\tarms={}\tcompleted={}\tfailed={}\tother={}\tinput={}\toutput={}\ttopology={}\tcode={}\tprevious={}",
        entry.sequence,
        entry.timestamp,
        entry.root_id,
        entry.status,
        entry.arms,
        entry.completed,
        entry.failed,
        entry.other,
        entry.input_hash,
        entry.output_hash,
        entry.topology_hash,
        entry.code,
        entry.previous_hash
    )
}

fn verify_content(content: &str) -> Result<VerificationReport, String> {
    let expected_header = header();
    let body = if content.is_empty() {
        ""
    } else {
        content
            .strip_prefix(&expected_header)
            .ok_or_else(|| "resonance header or schema mismatch".to_string())?
    };
    let mut entries = Vec::new();
    let mut previous = GENESIS_HASH.to_string();
    let mut roots = HashSet::new();
    for line in body.lines().filter(|line| !line.trim().is_empty()) {
        if !line.starts_with("entry\t") {
            return Err("unexpected content in append-only resonance log".to_string());
        }
        let entry = parse_entry(line)?;
        let expected_sequence = entries.len() as u64 + 1;
        if entry.sequence != expected_sequence {
            return Err(format!(
                "resonance sequence mismatch: expected {expected_sequence}, got {}",
                entry.sequence
            ));
        }
        if entry.previous_hash != previous {
            return Err(format!(
                "resonance previous hash mismatch at sequence {}",
                entry.sequence
            ));
        }
        if !roots.insert(entry.root_id.clone()) {
            return Err(format!("duplicate resonance root '{}'", entry.root_id));
        }
        for (name, value) in [
            ("input", entry.input_hash.as_str()),
            ("output", entry.output_hash.as_str()),
            ("topology", entry.topology_hash.as_str()),
            ("previous", entry.previous_hash.as_str()),
            ("hash", entry.entry_hash.as_str()),
        ] {
            if value.len() != 64 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
                return Err(format!(
                    "invalid resonance {name} hash at sequence {}",
                    entry.sequence
                ));
            }
        }
        if entry.arms != entry.completed + entry.failed + entry.other {
            return Err(format!(
                "resonance arm counts do not balance at sequence {}",
                entry.sequence
            ));
        }
        let expected_hash = hash(payload(&entry));
        if entry.entry_hash != expected_hash {
            return Err(format!(
                "resonance hash mismatch at sequence {}",
                entry.sequence
            ));
        }
        previous = entry.entry_hash.clone();
        entries.push(entry);
    }
    Ok(VerificationReport {
        entries,
        head_hash: previous,
    })
}

fn header() -> String {
    format!("OCTOPUS RESONANCE LOG\nschema: {SCHEMA}\nchain: sha256\nmode: append-only\n\n")
}

fn append_at(
    path: &Path,
    root: &RootRecord,
    arms: &[ArmRecord],
    outcome: &ExecutionOutcome,
) -> Result<ResonanceEntry, String> {
    let _lock = acquire_lock(path)?;
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("cannot read resonance log: {error}")),
    };
    let report = verify_content(&content)?;
    if let Some(existing) = report.entries.iter().find(|entry| entry.root_id == root.id) {
        return Ok(existing.clone());
    }

    let completed = arms
        .iter()
        .filter(|arm| arm.status == ArmStatus::Completed)
        .count();
    let failed = arms
        .iter()
        .filter(|arm| arm.status == ArmStatus::Failed)
        .count();
    let other = arms.len().saturating_sub(completed + failed);
    let mut topology: Vec<_> = arms.iter().map(|arm| arm.name.as_str()).collect();
    topology.sort_unstable();
    let mut entry = ResonanceEntry {
        sequence: report.entries.len() as u64 + 1,
        timestamp: root.finished_at.unwrap_or_else(now_millis),
        root_id: root.id.clone(),
        status: match outcome.status {
            ExecutionStatus::Completed => "completed".to_string(),
            ExecutionStatus::Failed => "failed".to_string(),
        },
        arms: arms.len(),
        completed,
        failed,
        other,
        input_hash: root.input_hash.clone(),
        output_hash: root
            .output_hash
            .clone()
            .unwrap_or_else(|| hash(&outcome.output)),
        topology_hash: hash(topology.join("||")),
        code: outcome
            .code
            .clone()
            .unwrap_or_else(|| "success".to_string()),
        previous_hash: report.head_hash,
        entry_hash: String::new(),
    };
    entry.entry_hash = hash(payload(&entry));

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create resonance directory: {error}"))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("cannot open resonance log: {error}"))?;
    if content.is_empty() {
        file.write_all(header().as_bytes())
            .map_err(|error| format!("cannot write resonance header: {error}"))?;
    }
    writeln!(
        file,
        "entry\t{}\thash={}",
        payload(&entry),
        entry.entry_hash
    )
    .map_err(|error| format!("cannot append resonance entry: {error}"))?;
    file.sync_data()
        .map_err(|error| format!("cannot sync resonance log: {error}"))?;
    Ok(entry)
}

pub(crate) fn append_root(
    root: &RootRecord,
    arms: &[ArmRecord],
    outcome: &ExecutionOutcome,
) -> Result<String, String> {
    let path = configured_path()?;
    let entry = append_at(&path, root, arms, outcome)?;
    Ok(format!(
        "resonance sequence={} root={} hash={}",
        entry.sequence, entry.root_id, entry.entry_hash
    ))
}

pub fn status_outcome(verify: bool, tail: usize) -> ExecutionOutcome {
    let path = match configured_path() {
        Ok(path) => path,
        Err(error) => return ExecutionOutcome::failed("resonance_path_invalid", error),
    };
    let _lock = match acquire_lock(&path) {
        Ok(lock) => lock,
        Err(error) => return ExecutionOutcome::failed("resonance_lock_failed", error),
    };
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return ExecutionOutcome::completed(format!(
                "RESONANCE LOG\npath: {}\nschema: {SCHEMA}\nintegrity: empty\nentries: 0\nhead: {GENESIS_HASH}",
                path.display()
            ));
        }
        Err(error) => {
            return ExecutionOutcome::failed(
                "resonance_read_failed",
                format!("cannot read resonance log: {error}"),
            );
        }
    };
    let report = match verify_content(&content) {
        Ok(report) => report,
        Err(error) => return ExecutionOutcome::failed("resonance_integrity_failed", error),
    };
    let mut output = format!(
        "RESONANCE LOG\npath: {}\nschema: {SCHEMA}\nintegrity: {}\nentries: {}\nhead: {}",
        path.display(),
        if verify { "verified" } else { "valid" },
        report.entries.len(),
        report.head_hash
    );
    let start = report.entries.len().saturating_sub(tail);
    for entry in &report.entries[start..] {
        output.push_str(&format!(
            "\n#{} ts={} root={} status={} arms={}/{}/{} code={} hash={}",
            entry.sequence,
            entry.timestamp,
            entry.root_id,
            entry.status,
            entry.completed,
            entry.failed,
            entry.other,
            entry.code,
            entry.entry_hash
        ));
    }
    ExecutionOutcome::completed(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQUENCE: AtomicU64 = AtomicU64::new(1);

    fn path(label: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "octopus-resonance-{label}-{}-{}.log",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn root(id: &str) -> RootRecord {
        RootRecord {
            id: id.to_string(),
            status: ArmStatus::Completed,
            prompt_hash: hash("input"),
            input_hash: hash("input"),
            output_hash: Some(hash("output")),
            started_at: 1,
            finished_at: Some(2),
            duration_ms: Some(1),
            children: Vec::new(),
        }
    }

    #[test]
    fn append_builds_a_verifiable_hash_chain() {
        let path = path("chain");
        append_at(
            &path,
            &root("root-1"),
            &[],
            &ExecutionOutcome::completed("one"),
        )
        .unwrap();
        append_at(
            &path,
            &root("root-2"),
            &[],
            &ExecutionOutcome::completed("two"),
        )
        .unwrap();
        let report = verify_content(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(report.entries.len(), 2);
        assert_eq!(
            report.entries[1].previous_hash,
            report.entries[0].entry_hash
        );
    }

    #[test]
    fn duplicate_root_append_is_idempotent() {
        let path = path("duplicate");
        let root = root("root-one");
        append_at(&path, &root, &[], &ExecutionOutcome::completed("one")).unwrap();
        append_at(&path, &root, &[], &ExecutionOutcome::completed("one")).unwrap();
        let report = verify_content(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(report.entries.len(), 1);
    }

    #[test]
    fn tampering_breaks_verification() {
        let path = path("tamper");
        append_at(
            &path,
            &root("root-1"),
            &[],
            &ExecutionOutcome::completed("one"),
        )
        .unwrap();
        let content = fs::read_to_string(path)
            .unwrap()
            .replace("status=completed", "status=failed");
        assert!(verify_content(&content).is_err());
    }
}
