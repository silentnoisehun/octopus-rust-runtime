use std::env;
use std::path::{Path, PathBuf};

#[cfg(test)]
use std::sync::OnceLock;
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
static UNIT_STATE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub(crate) fn state_dir() -> PathBuf {
    if let Some(configured) = env::var_os("OCTOPUS_STATE_DIR") {
        let configured = PathBuf::from(configured);
        return if configured.is_absolute() {
            configured
        } else {
            env::current_dir()
                .map(|directory| directory.join(&configured))
                .unwrap_or(configured)
        };
    }

    #[cfg(test)]
    {
        UNIT_STATE_DIR
            .get_or_init(|| {
                let stamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                env::temp_dir().join(format!(
                    "octopus-runtime-unit-{}-{stamp}",
                    std::process::id()
                ))
            })
            .clone()
    }

    #[cfg(not(test))]
    {
        default_state_dir()
    }
}

/// Portable production default: the OS data-local directory (Windows
/// `%LOCALAPPDATA%`, Linux `$XDG_DATA_HOME` / `~/.local/share`, macOS
/// `~/Library/Application Support`) joined with the runtime state root,
/// falling back to the current directory and finally `.`. Never embeds a
/// machine-specific absolute path.
fn default_state_dir() -> PathBuf {
    dirs::data_local_dir()
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("octopus-rust-runtime")
        .join(".octopus-rust")
}

pub(crate) fn sidecar_path(root: &Path, suffix: &str) -> Result<PathBuf, String> {
    if suffix.is_empty()
        || !suffix
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '.'))
    {
        return Err("invalid state sidecar suffix".to_string());
    }
    let parent = root
        .parent()
        .ok_or_else(|| "state directory has no parent".to_string())?;
    let name = root
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.trim_start_matches('.'))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "state directory has no valid name".to_string())?;
    Ok(parent.join(format!(".{name}.{suffix}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_unit_state_never_targets_the_live_directory() {
        if env::var_os("OCTOPUS_STATE_DIR").is_none() {
            assert_ne!(state_dir(), PathBuf::from(r"D:\codex\.octopus-rust"));
            assert!(state_dir().starts_with(env::temp_dir()));
        }
    }

    #[test]
    fn production_default_is_portable_and_never_machine_specific() {
        let default = default_state_dir();
        assert_ne!(default, PathBuf::from(r"D:\codex\.octopus-rust"));
        assert!(
            !default.to_string_lossy().contains(r"D:\codex"),
            "default must not embed a machine-specific path: {}",
            default.display()
        );
        assert!(
            default.is_absolute(),
            "default must resolve to an absolute OS-conventional location: {}",
            default.display()
        );
        assert!(
            default.ends_with(Path::new("octopus-rust-runtime").join(".octopus-rust")),
            "default must target the portable runtime state root: {}",
            default.display()
        );
    }

    #[test]
    fn sidecar_path_normalizes_a_dot_prefixed_state_name() {
        let state = PathBuf::from(r"D:\codex\.octopus-rust");
        assert_eq!(
            sidecar_path(&state, "state.lock").unwrap(),
            PathBuf::from(r"D:\codex\.octopus-rust.state.lock")
        );
    }
}
