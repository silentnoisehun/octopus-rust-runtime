//! Microscope commitment gate wired directly into the Octopus native runtime.
//!
//! The Octopus binary loads the *same persisted Microscope enforcement state
//! and audit chain* and calls `EnforcementEngine::can_execute()` before it may
//! invoke the native blade executor. This is a **fail-closed** boundary:
//!
//! - `OCTOPUS_ENFORCE=1` activates the gate (opt-in, so existing behaviour is
//!   preserved when unset);
//! - missing / unreadable / corrupt state, an unreadable audit, or an invalid
//!   chain MUST deny the blade (never a silent default);
//! - `Blocked` and `AttributionError` => the native executor is never called.
//! - every decision is persisted back to the shared audit chain.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use microscope_memory::enforcement::{load_engine_strict, save_audit, ActionEvent};

/// Runtime configuration for the gate.
#[derive(Debug, Clone)]
pub struct EnforcementConfig {
    pub state_dir: PathBuf,
    pub actor: String,
    pub scope: String,
    /// Optional documented justification for an override (the `guardian`
    /// path of the Microscope gate).
    pub justification: Option<String>,
}

impl EnforcementConfig {
    /// Build from process environment. Returns `None` when enforcement is off.
    ///
    /// Env:
    /// - `OCTOPUS_ENFORCE=1`
    /// - `OCTOPUS_ENFORCE_STATE_DIR=<dir>` (holds enforcement-state.bin /
    ///   enforcement-audit.bin)
    /// - `OCTOPUS_ENFORCE_ACTOR` (default `octopus`)
    /// - `OCTOPUS_ENFORCE_SCOPE` (default `octopus`)
    pub fn from_env() -> Option<Self> {
        let on = std::env::var("OCTOPUS_ENFORCE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if !on {
            return None;
        }
        let state_dir = PathBuf::from(std::env::var("OCTOPUS_ENFORCE_STATE_DIR").ok()?);
        let actor = std::env::var("OCTOPUS_ENFORCE_ACTOR").unwrap_or_else(|_| "operator".into());
        let scope = std::env::var("OCTOPUS_ENFORCE_SCOPE").unwrap_or_else(|_| "octopus".into());
        let justification = std::env::var("OCTOPUS_ENFORCE_JUSTIFICATION").ok();
        Some(Self {
            state_dir,
            actor,
            scope,
            justification,
        })
    }
}

/// Result of the gate consultation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    /// The action is inside A_t^valid; the native executor may run.
    Allow,
    /// Blocked (or faulty attribution); the native executor MUST NOT run.
    Deny(String),
}

impl Gate {
    pub fn allowed(&self) -> bool {
        matches!(self, Gate::Allow)
    }
}

/// Consult the gate. `Err` is a fail-closed condition: the caller must treat
/// it exactly like a denial and NOT invoke the native executor.
pub fn gate(spec: &str, prompt: &str, cfg: &EnforcementConfig) -> Result<Gate, String> {
    if !cfg.state_dir.is_dir() {
        return Err(format!(
            "enforcement state dir not found: {} (fail-closed)",
            cfg.state_dir.display()
        ));
    }
    if !cfg.state_dir.join("enforcement-state.bin").exists() {
        return Err(
            "enforcement state not provisioned (enforcement-state.bin missing), fail-closed"
                .to_string(),
        );
    }

    let mut engine = load_engine_strict(&cfg.state_dir)
        .map_err(|e| format!("enforcement load failed (fail-closed): {e}"))?;

    let event = ActionEvent {
        actor: cfg.actor.clone(),
        action: format!("run:{spec}"),
        content: prompt.chars().take(300).collect(),
        ts_ms: now_ms(),
        scope: cfg.scope.clone(),
        provenance: "octopus-runtime/execute_component".to_string(),
    };

    if let Some(reason) = event.attribution_error() {
        return Ok(Gate::Deny(format!("faulty attribution: {reason}")));
    }

    let allowed = engine.can_execute(&event, cfg.justification.as_deref());
    // Persist the decision and the audit chain (fail-closed on write errors).
    save_audit(&cfg.state_dir, engine.audit())
        .map_err(|e| format!("enforcement audit persist failed (fail-closed): {e}"))?;

    if allowed {
        Ok(Gate::Allow)
    } else {
        Ok(Gate::Deny(
            "blocked by Microscope commitment gate".to_string(),
        ))
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use microscope_memory::enforcement::{save_audit, save_engine, EnforcementEngine};

    use super::*;

    fn cfg(dir: &Path) -> EnforcementConfig {
        EnforcementConfig {
            state_dir: dir.to_path_buf(),
            actor: "octopus".to_string(),
            scope: "octopus".to_string(),
            justification: None,
        }
    }

    fn provision(dir: &Path, forbidden: &str) {
        let mut eng = EnforcementEngine::new();
        eng.add_commitment("*", forbidden, "octopus", "test policy", None);
        save_engine(dir, &eng).unwrap();
        save_audit(dir, eng.audit()).unwrap();
    }

    #[test]
    fn allowed_action_passes_the_gate() {
        let dir = tempfile::tempdir().unwrap();
        provision(dir.path(), "run:write");
        let c = cfg(dir.path());
        assert_eq!(gate("read", "read me", &c).unwrap(), Gate::Allow);
    }

    #[test]
    fn blocked_action_is_denied() {
        let dir = tempfile::tempdir().unwrap();
        provision(dir.path(), "run:write");
        let c = cfg(dir.path());
        let result = gate("write", "write me", &c).unwrap();
        assert_eq!(
            result,
            Gate::Deny("blocked by Microscope commitment gate".into())
        );
    }

    #[test]
    fn enforcement_off_is_inert() {
        // No env => no gate configured.
        assert!(EnforcementConfig::from_env().is_none());
    }

    #[test]
    fn missing_state_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        // No enforcement-state.bin.
        let c = cfg(dir.path());
        assert!(
            gate("run:x", "x", &c).is_err(),
            "unprovisioned state must deny"
        );

        // Missing dir entirely.
        let c2 = EnforcementConfig {
            state_dir: PathBuf::from(format!("{}/nope", dir.path().display())),
            actor: "octopus".into(),
            scope: "octopus".into(),
            justification: None,
        };
        let err = gate("run:x", "x", &c2).unwrap_err();
        assert!(err.contains("fail-closed"));
    }

    #[test]
    fn corrupt_state_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("enforcement-state.bin"), b"junk").unwrap();
        let c = cfg(dir.path());
        let err = gate("run:x", "x", &c).unwrap_err();
        assert!(err.contains("enforcement load failed"), "got: {err}");
    }

    #[test]
    fn corrupt_audit_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("enforcement-state.bin"), b"provisioned").unwrap();
        std::fs::write(dir.path().join("enforcement-audit.bin"), b"EAU1junk").unwrap();
        let c = cfg(dir.path());
        let err = gate("run:x", "x", &c).unwrap_err();
        assert!(err.contains("enforcement"), "got: {err}");
    }

    #[test]
    fn guardian_override_passes_the_gate() {
        let dir = tempfile::tempdir().unwrap();
        provision(dir.path(), "run:write");
        let c = EnforcementConfig {
            state_dir: dir.path().to_path_buf(),
            actor: "guardian".to_string(),
            scope: "octopus".to_string(),
            justification: Some("approved incident override".to_string()),
        };
        assert_eq!(gate("write", "write me", &c).unwrap(), Gate::Allow);
    }

    #[test]
    fn faulty_attribution_is_denied() {
        let dir = tempfile::tempdir().unwrap();
        provision(dir.path(), "run:write");
        let c = EnforcementConfig {
            state_dir: dir.path().to_path_buf(),
            actor: String::new(), // faulty: empty actor
            scope: "octopus".to_string(),
            justification: None,
        };
        let result = gate("write", "write me", &c).unwrap();
        assert!(
            matches!(&result, Gate::Deny(reason) if reason.contains("faulty attribution")),
            "got: {result:?}"
        );
    }
}
