use crate::outcome::ExecutionOutcome;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub struct ArmSnapshot {
    id: String,
    path: PathBuf,
    completed: bool,
}

impl ArmSnapshot {
    pub fn start(name: &str, prompt: &str, parent: Option<&str>) -> Self {
        let root = state_dir();
        let arms = root.join("arms");
        fs::create_dir_all(&arms).expect("cannot create Octopus snapshot directory");

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
        fs::write(&path, content).expect("cannot write Octopus arm snapshot");
        append_event(&root, &id, "running", name);

        Self {
            id,
            path,
            completed: false,
        }
    }

    pub fn finish(&mut self, outcome: &ExecutionOutcome) {
        self.append_status(
            outcome.status.as_str(),
            outcome.code.as_deref(),
            &outcome.output,
        );
        self.completed = true;
    }

    fn append_status(&self, status: &str, code: Option<&str>, output: &str) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.path)
            .expect("cannot update Octopus arm snapshot");
        writeln!(
            file,
            "status: {status}\ncode: {}\nupdated: {}\noutput-sha256: {}\noutput-bytes: {}\n",
            code.unwrap_or("-"),
            now_millis(),
            digest(output),
            output.len()
        )
        .expect("cannot append Octopus arm snapshot");
        if let Some(root) = self.path.parent().and_then(Path::parent) {
            append_event(root, &self.id, status, "-");
        }
    }
}

impl Drop for ArmSnapshot {
    fn drop(&mut self) {
        if !self.completed {
            self.append_status(
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

fn append_event(root: &Path, id: &str, status: &str, name: &str) {
    let mut events = OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("events.log"))
        .expect("cannot append Octopus event log");
    writeln!(events, "{}\t{id}\t{status}\t{}", now_millis(), clean(name))
        .expect("cannot write Octopus event log");
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
