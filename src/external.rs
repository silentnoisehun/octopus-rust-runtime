use crate::outcome::ExecutionOutcome;
use crate::process::{self, ProcessSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthState {
    Authenticated,
    Unauthenticated,
    Unknown,
}

impl AuthState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Authenticated => "authenticated",
            Self::Unauthenticated => "unauthenticated",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExternalProbe {
    pub available: bool,
    pub auth_state: AuthState,
    pub version: Option<String>,
    pub error: Option<String>,
}

pub fn probe_executable(name: &str) -> ExternalProbe {
    let spec = ProcessSpec::new(name).arg("--version").timeout_ms(5000);
    match process::run_process(&spec) {
        Ok(result) if result.exit_code == 0 => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let version = stdout.lines().next().map(|line| line.trim().to_string());
            ExternalProbe {
                available: true,
                auth_state: AuthState::Unknown,
                version,
                error: None,
            }
        }
        Ok(_) => ExternalProbe {
            available: false,
            auth_state: AuthState::Unknown,
            version: None,
            error: Some(format!("{name} returned non-zero exit code")),
        },
        Err(outcome) => ExternalProbe {
            available: false,
            auth_state: AuthState::Unknown,
            version: None,
            error: Some(outcome.output),
        },
    }
}

pub fn probe_gh_auth() -> ExternalProbe {
    let mut probe = probe_executable("gh");
    if !probe.available {
        return probe;
    }

    let spec = ProcessSpec::new("gh")
        .args(["auth", "status"])
        .timeout_ms(10000);
    match process::run_process(&spec) {
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            let stdout = String::from_utf8_lossy(&result.stdout);
            let combined = format!("{stdout}{stderr}");
            if combined.contains("Logged in") || combined.contains("account") {
                probe.auth_state = AuthState::Authenticated;
            } else if combined.contains("not logged in")
                || combined.contains("authentication")
                || result.exit_code != 0
            {
                probe.auth_state = AuthState::Unauthenticated;
            }
        }
        Err(_) => {
            probe.auth_state = AuthState::Unknown;
        }
    }
    probe
}

pub fn probe_curl() -> ExternalProbe {
    probe_executable("curl")
}

pub fn run_external_read(
    name: &str,
    executable: &str,
    args: &[&str],
    timeout_ms: u64,
) -> ExecutionOutcome {
    let probe = probe_executable(executable);
    if !probe.available {
        return ExecutionOutcome::failed(
            "tool_unavailable",
            format!("[{name}] required tool '{executable}' is not available on PATH"),
        );
    }

    let spec = ProcessSpec::new(executable)
        .args(args.iter().map(|s| s.to_string()))
        .timeout_ms(timeout_ms)
        .max_output_bytes(1024 * 1024);

    let result = process::run_process_typed(&spec);
    if result.is_failed() {
        return result;
    }

    // Check for rate limiting in output
    let output = &result.output;
    if output.contains("rate limit") || output.contains("403") || output.contains("429") {
        return ExecutionOutcome::failed(
            "rate_limited",
            format!("[{name}] rate limit detected in response"),
        );
    }

    // Redact any tokens from the output before returning
    let result = ExecutionOutcome {
        status: result.status,
        code: result.code,
        output: redact_tokens(&result.output),
    };
    result
}

pub fn require_auth(name: &str, probe: &ExternalProbe) -> Result<(), ExecutionOutcome> {
    match probe.auth_state {
        AuthState::Authenticated => Ok(()),
        AuthState::Unauthenticated => Err(ExecutionOutcome::failed(
            "auth_required",
            format!("[{name}] authentication required. Run 'gh auth login' or set credentials."),
        )),
        AuthState::Unknown => Err(ExecutionOutcome::failed(
            "auth_unknown",
            format!("[{name}] cannot determine authentication state"),
        )),
    }
}

pub fn redact_tokens(text: &str) -> String {
    let mut result = text.to_string();
    let token_patterns = [
        "ghp_", "gho_", "ghu_", "ghs_", "ghr_", "sk-", "Bearer ", "token=",
    ];
        for pattern in &token_patterns {
        let mut search_from = 0;
        while let Some(idx) = result[search_from..].find(pattern) {
            let idx = search_from + idx;
            let start = idx + pattern.len();
            let end = result[start..]
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                .map(|p| start + p)
                .unwrap_or(result.len());
            if end > start && (end - start) > 4 {
                let original = &result[start..end].to_string();
                let masked = format!("{}***{}", &original[..2], &original[original.len() - 2..]);
                result.replace_range(start..end, &masked);
                search_from = start + masked.len();
            } else {
                search_from = start;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_curl_works_on_windows() {
        let probe = probe_curl();
        assert!(probe.available);
    }

    #[test]
    fn probe_gh_detects_availability() {
        let probe = probe_executable("gh");
        // gh may or may not be installed
        if probe.available {
            assert!(probe.version.is_some());
        }
    }

    #[test]
    fn auth_state_as_str() {
        assert_eq!(AuthState::Authenticated.as_str(), "authenticated");
        assert_eq!(AuthState::Unauthenticated.as_str(), "unauthenticated");
        assert_eq!(AuthState::Unknown.as_str(), "unknown");
    }

    #[test]
    fn redact_tokens_masks_ghp() {
        let text = "Authorization: ghp_abc123def456ghi789";
        let redacted = redact_tokens(text);
        assert!(!redacted.contains("abc123def456ghi789"));
        assert!(redacted.contains("ghp_"));
    }

    #[test]
    fn redact_tokens_masks_bearer() {
        let text = "Bearer sk-abc123def456ghi789";
        let redacted = redact_tokens(text);
        assert!(!redacted.contains("abc123def456ghi789"));
    }

    #[test]
    fn require_auth_returns_ok_for_authenticated() {
        let probe = ExternalProbe {
            available: true,
            auth_state: AuthState::Authenticated,
            version: Some("2.0.0".to_string()),
            error: None,
        };
        assert!(require_auth("test", &probe).is_ok());
    }

    #[test]
    fn require_auth_returns_err_for_unauthenticated() {
        let probe = ExternalProbe {
            available: true,
            auth_state: AuthState::Unauthenticated,
            version: Some("2.0.0".to_string()),
            error: None,
        };
        let err = require_auth("test", &probe).unwrap_err();
        assert_eq!(err.code.as_deref(), Some("auth_required"));
    }

    #[test]
    fn redact_tokens_masks_multiple_occurrences_of_same_pattern() {
        let text = "ghp_aaa111 and ghp_bbb222";
        let redacted = redact_tokens(text);
        assert!(!redacted.contains("aaa111"), "first token should be masked: {redacted}");
        assert!(!redacted.contains("bbb222"), "second token should be masked: {redacted}");
        assert!(redacted.contains("ghp_"), "prefix should remain: {redacted}");
    }
}
