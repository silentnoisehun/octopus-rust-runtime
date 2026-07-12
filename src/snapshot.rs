use crate::outcome::ExecutionOutcome;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicU64 = AtomicU64::new(1);

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
        fs::write(&path, &content)?;
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
        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        writeln!(
            file,
            "status: {status}\ncode: {}\nupdated: {}\noutput-sha256: {}\noutput-bytes: {}\n",
            code.unwrap_or("-"),
            now_millis(),
            digest(output),
            output.len()
        )?;
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

fn state_dir() -> PathBuf {
    env::var_os("OCTOPUS_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"D:\codex\.octopus-rust"))
}

fn append_event(root: &Path, id: &str, status: &str, name: &str) -> Result<(), SnapshotError> {
    let mut events = OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("events.log"))?;
    writeln!(events, "{}\t{id}\t{status}\t{}", now_millis(), clean(name))?;
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
    value.trim_matches('-').chars().take(48).collect::<String>()
}

fn clean(value: &str) -> String {
    value.replace(['\r', '\n', '\t'], " ")
}
