use crate::outcome::ExecutionOutcome;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const MAX_OUTPUT_BYTES: usize = 1024 * 1024;
const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const KILL_TIMEOUT_MS: u64 = 5_000;

#[derive(Debug, Clone)]
pub struct ProcessSpec {
    pub executable: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env_allowlist: Option<HashSet<String>>,
    pub env_overrides: HashMap<String, String>,
    pub timeout_ms: Option<u64>,
    pub max_output_bytes: Option<usize>,
    pub git_optional_locks: bool,
    pub no_color: bool,
}

#[allow(dead_code)]
impl ProcessSpec {
    pub fn new(executable: impl Into<String>) -> Self {
        Self {
            executable: executable.into(),
            args: Vec::new(),
            cwd: None,
            env_allowlist: None,
            env_overrides: HashMap::new(),
            timeout_ms: None,
            max_output_bytes: None,
            git_optional_locks: false,
            no_color: true,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn cwd(mut self, path: impl Into<PathBuf>) -> Self {
        self.cwd = Some(path.into());
        self
    }

    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    pub fn max_output_bytes(mut self, bytes: usize) -> Self {
        self.max_output_bytes = Some(bytes);
        self
    }

    pub fn git_optional_locks(mut self, enable: bool) -> Self {
        self.git_optional_locks = enable;
        self
    }

    pub fn no_color(mut self, enable: bool) -> Self {
        self.no_color = enable;
        self
    }

    pub fn env_allowlist(mut self, keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.env_allowlist = Some(keys.into_iter().map(Into::into).collect());
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_overrides.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProcessResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub timed_out: bool,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

pub fn run_process(spec: &ProcessSpec) -> Result<ProcessResult, ExecutionOutcome> {
    validate_executable(&spec.executable)?;

    if let Some(ref cwd) = spec.cwd {
        validate_cwd(cwd)?;
    }

    let mut command = Command::new(&spec.executable);
    command.args(&spec.args);

    if let Some(ref cwd) = spec.cwd {
        command.current_dir(cwd);
    }

    // Environment filtering: start clean, add only allowlisted vars
    command.env_clear();
    if spec.no_color {
        command.env("NO_COLOR", "1");
    }
    if spec.git_optional_locks {
        command.env("GIT_OPTIONAL_LOCKS", "0");
    }

    // Inherit PATH so executables can be found
    if let Ok(path) = env::var("PATH") {
        command.env("PATH", path);
    }

    // Inherit system root, temp dirs and read-only platform/toolchain roots. Native
    // Bio processes use these paths to discover installed MSVC components while
    // the child still starts from an otherwise empty environment.
    for key in &[
        "SystemRoot",
        "TEMP",
        "TMP",
        "USERPROFILE",
        "HOME",
        "ComSpec",
        "PATHEXT",
        "ProgramData",
        "ProgramFiles",
        "ProgramFiles(x86)",
        "ProgramW6432",
        "CommonProgramFiles",
        "CommonProgramFiles(x86)",
        "CommonProgramW6432",
        "LOCALAPPDATA",
        "APPDATA",
    ] {
        if let Ok(val) = env::var(key) {
            command.env(key, val);
        }
    }

    if let Some(ref allowlist) = spec.env_allowlist {
        for key in allowlist {
            if let Ok(val) = env::var(key) {
                command.env(key, val);
            }
        }
    }
    for (key, value) in &spec.env_overrides {
        command.env(key, value);
    }

    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let timeout = spec.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS);
    let max_output = spec.max_output_bytes.unwrap_or(MAX_OUTPUT_BYTES);

    let child = command.spawn().map_err(|error| {
        ExecutionOutcome::failed(
            "process_spawn_failed",
            format!("[process] cannot start {}: {}", spec.executable, error),
        )
    })?;

    let (tx, rx) = mpsc::channel();
    let child_id = child.id();

    thread::spawn(move || {
        let result = child.wait_with_output();
        let _ = tx.send(result);
    });

    let timeout_duration = Duration::from_millis(timeout);

    let raw = match rx.recv_timeout(timeout_duration) {
        Ok(result) => result.map_err(|error| {
            ExecutionOutcome::failed(
                "process_wait_failed",
                format!("[process] {} wait failed: {}", spec.executable, error),
            )
        })?,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Kill the child process
            #[cfg(windows)]
            {
                use std::process::Command as KillCmd;
                let _ = KillCmd::new("taskkill")
                    .args(["/F", "/T", "/PID", &child_id.to_string()])
                    .output();
            }
            #[cfg(not(windows))]
            {
                unsafe {
                    libc::kill(child_id as i32, libc::SIGKILL);
                }
            }

            // Wait briefly for cleanup
            let _ = rx.recv_timeout(Duration::from_millis(KILL_TIMEOUT_MS));

            return Err(ExecutionOutcome::failed(
                "process_timeout",
                format!(
                    "[process] {} timed out after {}ms",
                    spec.executable, timeout
                ),
            ));
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            return Err(ExecutionOutcome::failed(
                "process_channel_disconnected",
                format!("[process] {} channel disconnected", spec.executable),
            ));
        }
    };

    let mut stdout = raw.stdout;
    let mut stderr = raw.stderr;
    let mut stdout_truncated = false;
    let mut stderr_truncated = false;

    if stdout.len() > max_output {
        stdout.truncate(max_output);
        stdout_truncated = true;
    }
    if stderr.len() > max_output {
        stderr.truncate(max_output);
        stderr_truncated = true;
    }

    let exit_code = raw.status.code().unwrap_or(-1);

    Ok(ProcessResult {
        exit_code,
        stdout,
        stderr,
        timed_out: false,
        stdout_truncated,
        stderr_truncated,
    })
}

pub fn run_process_typed(spec: &ProcessSpec) -> ExecutionOutcome {
    match run_process(spec) {
        Ok(result) => {
            let stdout_text = String::from_utf8_lossy(&result.stdout);
            let stderr_text = String::from_utf8_lossy(&result.stderr);
            let mut output = String::new();

            if !stdout_text.is_empty() {
                output.push_str(&stdout_text);
            }
            if !stderr_text.is_empty() {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str("[stderr] ");
                output.push_str(&stderr_text);
            }

            if result.stdout_truncated || result.stderr_truncated {
                output.push_str("\n[warning] output was truncated");
            }

            if result.exit_code == 0 {
                ExecutionOutcome::completed(output)
            } else {
                ExecutionOutcome::failed(
                    "non_zero_exit",
                    format!(
                        "[process] exit code {}\n{}",
                        result.exit_code,
                        redact_secrets(&output)
                    ),
                )
            }
        }
        Err(outcome) => outcome,
    }
}

fn validate_executable(executable: &str) -> Result<(), ExecutionOutcome> {
    if executable.is_empty() {
        return Err(ExecutionOutcome::failed(
            "empty_executable",
            "[process] executable name is empty",
        ));
    }

    // Block shell interpreters
    let lower = executable.to_ascii_lowercase();
    let blocked = [
        "sh",
        "bash",
        "zsh",
        "cmd",
        "powershell",
        "pwsh",
        "cmd.exe",
        "powershell.exe",
        "pwsh.exe",
        "bash.exe",
        "sh.exe",
        "zsh.exe",
    ];
    if blocked.iter().any(|b| {
        lower == *b || lower.ends_with(&format!("/{b}")) || lower.ends_with(&format!("\\{b}"))
    }) {
        return Err(ExecutionOutcome::failed(
            "shell_blocked",
            format!("[process] shell execution is not allowed: {}", executable),
        ));
    }

    // Block injection patterns
    if executable.contains(';')
        || executable.contains('|')
        || executable.contains('&')
        || executable.contains('$')
        || executable.contains('`')
        || executable.contains('"')
        || executable.contains('\'')
    {
        return Err(ExecutionOutcome::failed(
            "argument_injection",
            format!(
                "[process] executable contains shell metacharacters: {}",
                executable
            ),
        ));
    }

    Ok(())
}

fn validate_cwd(cwd: &Path) -> Result<(), ExecutionOutcome> {
    if !cwd.is_dir() {
        return Err(ExecutionOutcome::failed(
            "invalid_cwd",
            format!(
                "[process] working directory does not exist: {}",
                cwd.display()
            ),
        ));
    }
    Ok(())
}

fn redact_secrets(text: &str) -> String {
    let mut result = text.to_string();
    let secret_patterns = ["password", "token", "secret", "key", "credential", "auth"];
    for pattern in &secret_patterns {
        if let Some(idx) = result.to_lowercase().find(pattern) {
            let after = &result[idx..];
            if let Some(eq_pos) = after.find('=') {
                let value_start = idx + eq_pos + 1;
                let value_end = after[eq_pos + 1..]
                    .find(|c: char| c.is_whitespace())
                    .map(|p| value_start + p)
                    .unwrap_or(result.len());
                if value_end > value_start {
                    let original = &result[value_start..value_end].to_string();
                    if original.len() > 4 {
                        let masked =
                            format!("{}***{}", &original[..2], &original[original.len() - 2..]);
                        result.replace_range(value_start..value_end, &masked);
                    }
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_where_produces_stdout() {
        let spec = ProcessSpec::new("where").arg("cargo");
        let result = run_process(&spec).unwrap();
        assert_eq!(result.exit_code, 0);
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("cargo"));
    }

    #[test]
    fn blocked_shell_returns_typed_failure() {
        let spec = ProcessSpec::new("sh").arg("-c").arg("echo hi");
        let outcome = run_process_typed(&spec);
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("shell_blocked"));
    }

    #[test]
    fn blocked_cmd_returns_typed_failure() {
        let spec = ProcessSpec::new("cmd").arg("/c").arg("echo hi");
        let outcome = run_process_typed(&spec);
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("shell_blocked"));
    }

    #[test]
    fn blocked_powershell_returns_typed_failure() {
        let spec = ProcessSpec::new("powershell").arg("-c").arg("Get-Date");
        let outcome = run_process_typed(&spec);
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("shell_blocked"));
    }

    #[test]
    fn nonexistent_executable_returns_typed_failure() {
        let spec = ProcessSpec::new("totally-fake-binary-xyz-12345");
        let outcome = run_process_typed(&spec);
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("process_spawn_failed"));
    }

    #[test]
    fn invalid_cwd_returns_typed_failure() {
        let spec = ProcessSpec::new("where")
            .arg("cargo")
            .cwd("Z:\\nonexistent\\path");
        let outcome = run_process_typed(&spec);
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("invalid_cwd"));
    }

    #[test]
    fn shell_metacharacters_in_executable_are_blocked() {
        let spec = ProcessSpec::new("echo;rm -rf /");
        let outcome = run_process_typed(&spec);
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("argument_injection"));
    }

    #[test]
    fn timeout_returns_typed_failure() {
        let spec = ProcessSpec::new("ping")
            .args(["-n", "60", "127.0.0.1"])
            .timeout_ms(200);
        let outcome = run_process_typed(&spec);
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("process_timeout"));
    }

    #[test]
    fn empty_executable_returns_typed_failure() {
        let spec = ProcessSpec::new("");
        let outcome = run_process_typed(&spec);
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("empty_executable"));
    }

    #[test]
    fn redact_secrets_masks_passwords() {
        let text = "password=supersecret123 token=abc123def456";
        let redacted = redact_secrets(text);
        assert!(!redacted.contains("supersecret123"));
        assert!(!redacted.contains("abc123def456"));
    }

    #[test]
    fn git_nexus_uses_no_color() {
        let spec = ProcessSpec::new("where").arg("cargo").no_color(true);
        assert!(spec.no_color);
    }

    #[test]
    fn spec_builder_sets_args() {
        let spec = ProcessSpec::new("git")
            .arg("status")
            .arg("--short")
            .args(["-C", "src/"]);
        assert_eq!(spec.args, vec!["status", "--short", "-C", "src/"]);
    }

    #[test]
    fn spec_builder_sets_cwd() {
        let spec = ProcessSpec::new("git").cwd("/tmp");
        assert_eq!(spec.cwd, Some(PathBuf::from("/tmp")));
    }

    #[test]
    fn spec_builder_sets_timeout() {
        let spec = ProcessSpec::new("git").timeout_ms(5000);
        assert_eq!(spec.timeout_ms, Some(5000));
    }

    #[test]
    fn git_available_on_windows() {
        let spec = ProcessSpec::new("git").args(["--version"]);
        let result = run_process(&spec).unwrap();
        assert_eq!(result.exit_code, 0);
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("git version"));
    }
}
