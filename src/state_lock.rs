use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use crate::state_path::sidecar_path;

const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(20);
const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LockMode {
    Shared,
    Exclusive,
}

pub(crate) struct StateLockGuard {
    file: File,
    #[allow(dead_code)]
    path: PathBuf,
}

impl Drop for StateLockGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

pub(crate) fn acquire_shared(root: &Path, timeout: Duration) -> Result<StateLockGuard, String> {
    acquire(root, LockMode::Shared, timeout)
}

pub(crate) fn acquire_exclusive(root: &Path, timeout: Duration) -> Result<StateLockGuard, String> {
    acquire(root, LockMode::Exclusive, timeout)
}

pub(crate) fn configured_timeout() -> Duration {
    std::env::var("OCTOPUS_STATE_LOCK_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|milliseconds| Duration::from_millis(milliseconds.clamp(50, 300_000)))
        .unwrap_or(DEFAULT_LOCK_TIMEOUT)
}

pub(crate) fn lock_path(root: &Path) -> Result<PathBuf, String> {
    let absolute = absolute_root(root)?;
    sidecar_path(&absolute, "state.lock")
}

fn absolute_root(root: &Path) -> Result<PathBuf, String> {
    if root.is_absolute() {
        Ok(root.to_path_buf())
    } else {
        std::env::current_dir()
            .map_err(|error| format!("cannot resolve current directory: {error}"))
            .map(|directory| directory.join(root))
    }
}

fn acquire(root: &Path, mode: LockMode, timeout: Duration) -> Result<StateLockGuard, String> {
    let root = absolute_root(root)?;
    match fs::symlink_metadata(&root) {
        Ok(metadata) if !metadata.is_dir() || metadata.file_type().is_symlink() => {
            return Err(format!(
                "state path is not a plain directory: {}",
                root.display()
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "cannot inspect state path {}: {error}",
                root.display()
            ));
        }
    }
    let path = lock_path(&root)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create state lock directory: {error}"))?;
    }
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|error| format!("cannot open state lock {}: {error}", path.display()))?;
    let started = Instant::now();
    loop {
        let result = match mode {
            LockMode::Shared => file.try_lock_shared(),
            LockMode::Exclusive => file.try_lock(),
        };
        match result {
            Ok(()) => return Ok(StateLockGuard { file, path }),
            Err(std::fs::TryLockError::WouldBlock) => {
                if started.elapsed() >= timeout {
                    return Err(format!(
                        "timed out waiting for {} state lock at {}",
                        match mode {
                            LockMode::Shared => "shared",
                            LockMode::Exclusive => "exclusive",
                        },
                        path.display()
                    ));
                }
                thread::sleep(LOCK_POLL_INTERVAL);
            }
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(format!("cannot lock state at {}: {error}", path.display()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    fn fixture() -> PathBuf {
        std::env::temp_dir().join(format!(
            "octopus-state-lock-{}-{}",
            std::process::id(),
            TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn lock_file_lives_outside_the_replaceable_state_directory() {
        let state = fixture().join("state");
        let lock = lock_path(&state).unwrap();
        assert_eq!(lock.parent(), state.parent());
        assert_ne!(lock, state.join("state.lock"));
    }

    #[test]
    fn exclusive_lock_waits_for_shared_guard_and_then_succeeds() {
        let state = fixture().join("state");
        let shared = acquire_shared(&state, Duration::from_secs(1)).unwrap();
        let error = acquire_exclusive(&state, Duration::from_millis(60))
            .err()
            .expect("exclusive lock must wait");
        assert!(error.contains("timed out"));
        drop(shared);
        let exclusive = acquire_exclusive(&state, Duration::from_secs(1)).unwrap();
        drop(exclusive);
        let _ = fs::remove_file(lock_path(&state).unwrap());
    }

    #[test]
    fn shared_lock_waits_for_exclusive_guard_and_then_succeeds() {
        let state = fixture().join("state");
        let exclusive = acquire_exclusive(&state, Duration::from_secs(1)).unwrap();
        let error = acquire_shared(&state, Duration::from_millis(60))
            .err()
            .expect("shared lock must wait");
        assert!(error.contains("timed out"));
        drop(exclusive);
        let shared = acquire_shared(&state, Duration::from_secs(1)).unwrap();
        drop(shared);
        let _ = fs::remove_file(lock_path(&state).unwrap());
    }

    #[test]
    fn invalid_file_state_path_is_rejected_without_creating_a_lock() {
        let base = fixture();
        fs::create_dir_all(&base).unwrap();
        let state = base.join("state-file");
        fs::write(&state, "not a directory").unwrap();
        let lock = lock_path(&state).unwrap();
        let error = acquire_shared(&state, Duration::from_millis(60))
            .err()
            .expect("file state path must fail");
        assert!(error.contains("not a plain directory"));
        assert!(!lock.exists());
        let _ = fs::remove_dir_all(base);
    }
}
