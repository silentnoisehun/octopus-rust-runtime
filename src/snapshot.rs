use crate::outcome::ExecutionOutcome;
use crate::state_path::state_dir;
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicU64 = AtomicU64::new(1);
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(1);
const EVENT_LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const STALE_EVENT_LOCK_AGE: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub enum SnapshotError {
    Io(std::io::Error),
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
        }
    }
}

impl From<std::io::Error> for SnapshotError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

struct EventLock {
    path: PathBuf,
}

impl Drop for EventLock {
    fn drop(&mut self) {
        for _ in 0..20 {
            match fs::remove_file(&self.path) {
                Ok(()) => return,
                Err(error) if error.kind() == ErrorKind::NotFound => return,
                Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                    thread::sleep(Duration::from_millis(2));
                }
                Err(_) => return,
            }
        }
    }
}

fn acquire_event_lock(root: &Path) -> Result<EventLock, SnapshotError> {
    fs::create_dir_all(root)?;
    let path = root.join("events.lock");
    let started = Instant::now();
    loop {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                writeln!(file, "pid={} created={}", std::process::id(), now_millis())?;
                file.sync_all()?;
                return Ok(EventLock { path });
            }
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::AlreadyExists | ErrorKind::PermissionDenied
                ) =>
            {
                let stale = fs::metadata(&path)
                    .and_then(|metadata| metadata.modified())
                    .and_then(|modified| modified.elapsed().map_err(std::io::Error::other))
                    .map(|age| age >= STALE_EVENT_LOCK_AGE)
                    .unwrap_or(false);
                if stale {
                    let _ = fs::remove_file(&path);
                    continue;
                }
                if started.elapsed() >= EVENT_LOCK_TIMEOUT {
                    return Err(std::io::Error::new(
                        ErrorKind::TimedOut,
                        "timed out waiting for events.log lock",
                    )
                    .into());
                }
                thread::sleep(Duration::from_millis(2));
            }
            Err(error) => return Err(error.into()),
        }
    }
}

pub(crate) fn atomic_write(path: &Path, content: &str) -> Result<(), SnapshotError> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(ErrorKind::InvalidInput, "snapshot path has no parent")
    })?;
    fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("snapshot");
    let temporary = parent.join(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));

    let result = (|| -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(SnapshotError::from)
}

pub(crate) fn replace_event_log(root: &Path, content: &str) -> Result<(), SnapshotError> {
    let _lock = acquire_event_lock(root)?;
    atomic_write(&root.join("events.log"), content)
}

pub struct ArmSnapshot {
    id: String,
    path: PathBuf,
    completed: bool,
}

impl ArmSnapshot {
    /// Start a new arm snapshot with fallible I/O. Returns Err if the state dir is unwritable.
    pub fn try_start(
        name: &str,
        prompt: &str,
        parent: Option<&str>,
    ) -> Result<Self, SnapshotError> {
        let root = state_dir();
        let arms = root.join("arms");
        fs::create_dir_all(&arms)?;

        let id = format!(
            "{}-{}-{}-{}",
            sanitize(name),
            now_millis(),
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let path = arms.join(format!("{id}.snap"));
        let content = format!(
            "OCTOPUS ARM SNAPSHOT\narm: {id}\nname: {}\nstatus: running\ncreated: {}\nparent: {}\nprompt-sha256: {}\n\n",
            clean(name),
            now_millis(),
            parent.unwrap_or("-"),
            digest(prompt)
        );
        atomic_write(&path, &content)?;
        // Non-fatal event log write failure: log and continue
        if let Err(e) = append_event(&root, &id, "running", name) {
            // Event log failure is advisory, not critical for snapshot correctness
            let _ = e;
        }

        Ok(Self {
            id,
            path,
            completed: false,
        })
    }

    /// Legacy compatibility wrapper. Prefer `try_start`.
    /// PANICS on I/O error — only use in contexts where unwritable state dir is impossible.
    #[allow(dead_code)]
    #[deprecated(
        since = "0.1.0",
        note = "use try_start instead; this panics on I/O error"
    )]
    pub fn start(name: &str, prompt: &str, parent: Option<&str>) -> Self {
        Self::try_start(name, prompt, parent).expect("snapshot start failed")
    }

    /// Return the snapshot ID
    #[allow(dead_code)]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Finish the snapshot. Returns Ok(()) on success, Err on I/O failure.
    pub fn try_finish(&mut self, outcome: &ExecutionOutcome) -> Result<(), SnapshotError> {
        let result = self.try_append_status(
            outcome.status.as_str(),
            outcome.code.as_deref(),
            &outcome.output,
        );
        if result.is_ok() {
            self.completed = true;
        }
        result
    }

    /// Legacy compatibility wrapper. Panics on I/O error.
    #[deprecated(
        since = "0.1.0",
        note = "use try_finish instead; this panics on I/O error"
    )]
    pub fn finish(&mut self, outcome: &ExecutionOutcome) {
        self.try_finish(outcome)
            .unwrap_or_else(|e| panic!("snapshot finish failed: {e}"));
    }

    fn try_append_status(
        &self,
        status: &str,
        code: Option<&str>,
        output: &str,
    ) -> Result<(), SnapshotError> {
        let mut content = fs::read_to_string(&self.path)?;
        content.push_str(&format!(
            "status: {status}\ncode: {}\nupdated: {}\noutput-sha256: {}\noutput-bytes: {}\n",
            code.unwrap_or("-"),
            now_millis(),
            digest(output),
            output.len()
        ));
        atomic_write(&self.path, &content)?;
        if let Some(root) = self.path.parent().and_then(Path::parent) {
            let _ = append_event(root, &self.id, status, "-");
        }
        Ok(())
    }
}

impl Drop for ArmSnapshot {
    fn drop(&mut self) {
        if !self.completed {
            // Avoid panic in Drop — silently catch I/O error
            let _ = self.try_append_status(
                "failed",
                Some("runtime_drop"),
                "runtime exited before completion",
            );
        }
    }
}

fn append_event(root: &Path, id: &str, status: &str, name: &str) -> Result<(), SnapshotError> {
    let _lock = acquire_event_lock(root)?;
    let line = format!("{}\t{id}\t{status}\t{}\n", now_millis(), clean(name));
    let mut events = OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("events.log"))?;
    events.write_all(line.as_bytes())?;
    events.sync_data()?;
    Ok(())
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn digest(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}

fn sanitize(value: &str) -> String {
    let value: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect();
    let value = value.trim_matches('-').chars().take(48).collect::<String>();
    if value.is_empty() {
        "arm".to_string()
    } else {
        value
    }
}

fn clean(value: &str) -> String {
    value.replace(['\r', '\n', '\t'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "octopus-snapshot-{label}-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn atomic_write_replaces_a_complete_snapshot() {
        let dir = temp_dir("replace");
        let path = dir.join("arm.snap");
        atomic_write(&path, "status: running\n").unwrap();
        atomic_write(&path, "status: completed\n").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "status: completed\n");
        assert_eq!(
            fs::read_dir(&dir).unwrap().filter_map(Result::ok).count(),
            1
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn concurrent_event_records_remain_whole_lines() {
        let dir = temp_dir("events");
        fs::create_dir_all(&dir).unwrap();
        let mut threads = Vec::new();
        for worker in 0..8 {
            let root = dir.clone();
            threads.push(thread::spawn(move || {
                for item in 0..25 {
                    append_event(
                        &root,
                        &format!("arm-{worker}-{item}"),
                        "completed",
                        "diagnostics",
                    )
                    .unwrap();
                }
            }));
        }
        for handle in threads {
            handle.join().unwrap();
        }

        let content = fs::read_to_string(dir.join("events.log")).unwrap();
        let lines = content.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 200);
        assert!(lines.iter().all(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            fields.len() == 4
                && fields[0].parse::<u128>().is_ok()
                && fields[1].starts_with("arm-")
                && fields[2] == "completed"
                && fields[3] == "diagnostics"
        }));
        assert!(!dir.join("events.lock").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn try_finish_returns_error_on_io_failure() {
        let dir = temp_dir("try_finish_error");
        let mut snap = ArmSnapshot::try_start("test-blade", "test prompt", None).unwrap();

        // Remove the snapshot file to simulate I/O error on append
        std::fs::remove_file(&snap.path).unwrap();

        let outcome = crate::outcome::ExecutionOutcome::completed("ok");
        let result = snap.try_finish(&outcome);

        assert!(
            result.is_err(),
            "try_finish should return Err when snapshot file is missing"
        );
        match result {
            Err(SnapshotError::Io(e)) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("expected SnapshotError::Io(NotFound)"),
        }

        let _ = fs::remove_dir_all(dir);
    }
}
