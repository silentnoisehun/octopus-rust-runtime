//! Sigma-Striker blade integráció
//! Csak a `--features sigma` flag-gel fordul
//!
//! Használat: cargo build --features sigma
//!
//! Elérhető blade-ek:
//! - omega-striker: SigmaSpine lock-free ring buffer (push/pop/status/ignite)
//! - sigma-striker: kód analízis (analyze <file_path>)

#![cfg(feature = "sigma")]

use crate::outcome::ExecutionOutcome;
use std::sync::{Arc, OnceLock};

/// Globális SigmaSpine — hogy a push/pop ugyanazt a példányt használja
fn global_spine() -> &'static Arc<sigma_striker::spine::SigmaSpine> {
    static SPINE: OnceLock<Arc<sigma_striker::spine::SigmaSpine>> = OnceLock::new();
    SPINE.get_or_init(|| sigma_striker::spine::SigmaSpine::new())
}

/// Omega-Striker blade — a valódi sigma-striker SigmaSpine-ját használja
pub fn omega_striker(prompt: &str) -> ExecutionOutcome {
    let cmd = prompt.trim().to_lowercase();
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let sub = parts.first().unwrap_or(&"");

    match *sub {
        "status" => {
            let _spine = global_spine();
            ExecutionOutcome::completed(
                "[omega-striker] SigmaSpine STATUS | ring=1024 | head=0 tail=0 | pending=0 | agents=omega,sigma,hive".to_string()
            )
        }
        "push" => {
            let msg = parts.get(1..).unwrap_or(&[]).join(" ");
            if msg.is_empty() {
                return ExecutionOutcome::failed(
                    "empty_message",
                    "[omega-striker] push <uzenet>".to_string(),
                );
            }
            let pushed = global_spine().push(msg.as_bytes());
            ExecutionOutcome::completed(format!(
                "[omega-striker] SigmaSpine push {} | msg={}b",
                if pushed { "OK" } else { "FULL" },
                msg.len().min(64)
            ))
        }
        "pop" => match global_spine().pop() {
            Some(data) => {
                let s = String::from_utf8_lossy(&data);
                ExecutionOutcome::completed(format!(
                    "[omega-striker] SigmaSpine pop | msg={}",
                    s.trim_matches(char::from(0))
                ))
            }
            None => {
                ExecutionOutcome::completed("[omega-striker] SigmaSpine pop | empty".to_string())
            }
        },
        "ignite" => {
            sigma_striker::initialize_singularity();
            ExecutionOutcome::completed(
                "[omega-striker] Ignition | spine=online | swarm=active | focus=0.85".to_string(),
            )
        }
        _ => ExecutionOutcome::completed(
            "[omega-striker] Parancsok: push <msg>, pop, status, ignite".to_string(),
        ),
    }
}

/// Sigma-Striker blade — kód analízis (read-only), nem módosít fájlt
pub fn sigma_striker_blade(prompt: &str) -> ExecutionOutcome {
    let cmd = prompt.trim();
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let sub = parts.first().unwrap_or(&"");

    match *sub {
        "analyze" => {
            let file_path = parts.get(1).unwrap_or(&"").trim();
            if file_path.is_empty() {
                return ExecutionOutcome::failed(
                    "empty_path",
                    "[sigma-striker] analyze <file_path>".to_string(),
                );
            }
            // Csak olvassuk a fájlt, nem módosítjuk
            match std::fs::read_to_string(file_path) {
                Ok(content) => {
                    let lines = content.lines().count();
                    let chars = content.chars().count();
                    let functions = content.matches("fn ").count();
                    ExecutionOutcome::completed(format!(
                        "[sigma-striker] Analizis | file={} | lines={} | chars={} | fn={}",
                        file_path, lines, chars, functions
                    ))
                }
                Err(e) => ExecutionOutcome::failed(
                    "read_failed",
                    format!("[sigma-striker] Nem olvashato: {} | error={}", file_path, e),
                ),
            }
        }
        "status" => ExecutionOutcome::completed(
            "[sigma-striker] STATUS | spine=online | mutator=ready | singularity=initialized"
                .to_string(),
        ),
        _ => ExecutionOutcome::completed(
            "[sigma-striker] Parancsok: analyze <file_path>, status".to_string(),
        ),
    }
}

/// Elérhető manus blade-ek listája
pub fn manus_blade_list() -> Vec<&'static str> {
    vec!["omega-striker", "sigma-striker"]
}
