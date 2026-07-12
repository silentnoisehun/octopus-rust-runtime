pub mod approval;
#[allow(clippy::all)]
mod blade;
mod capability;
mod composite;
pub mod contract;
pub mod external;
mod mcp;
pub mod orchestration;
mod outcome;
mod process;
pub mod real_blades;
mod snapshot;

pub use capability::{CapabilityInfo, CapabilityMode, CapabilityStatus};
pub use contract::CapabilityContract;
pub use outcome::{ExecutionOutcome, ExecutionStatus};

use sha2::{Digest, Sha256};

pub fn run(blade_name: &str, prompt: &str) -> String {
    run_outcome(blade_name, prompt).output
}

pub fn run_outcome(blade_name: &str, prompt: &str) -> ExecutionOutcome {
    if matches!(blade_name, "pipeline-architect" | "rust-surgeon") {
        return run_arm_outcome(blade_name, prompt);
    }
    let mut snapshot = snapshot::ArmSnapshot::start(blade_name, prompt, None);
    let outcome = execute_component(blade_name, prompt);
    snapshot.finish(&outcome);
    outcome
}

pub fn run_arm(spec: &str, prompt: &str) -> String {
    run_arm_outcome(spec, prompt).output
}

pub fn run_arm_outcome(spec: &str, prompt: &str) -> ExecutionOutcome {
    let components: Vec<_> = spec
        .split('+')
        .map(str::trim)
        .filter(|component| !component.is_empty())
        .collect();
    if components.is_empty() {
        return ExecutionOutcome::failed("empty_arm", "Empty composite arm");
    }

    let mut snapshot = snapshot::ArmSnapshot::start(spec, prompt, Some("O"));
    let mut context = prompt.to_string();
    let mut outputs = Vec::new();
    let mut boundary = None;

    for component in &components {
        let outcome = match *component {
            "pipeline-architect" => match composite::BoundaryContract::from_prompt(prompt) {
                Ok(contract) => {
                    let description = contract.describe();
                    boundary = Some(contract);
                    ExecutionOutcome::completed(description)
                }
                Err(error) => ExecutionOutcome::failed(
                    "architect_refused",
                    format!("[pipeline-architect] REFUSED: {error}"),
                ),
            },
            "rust-surgeon" => match boundary.as_ref() {
                Some(contract) => match contract.apply() {
                    Ok(output) => ExecutionOutcome::completed(output),
                    Err(error) => ExecutionOutcome::failed(
                        "surgeon_refused",
                        format!("[rust-surgeon] REFUSED: {error}"),
                    ),
                },
                None => ExecutionOutcome::failed(
                    "surgeon_refused",
                    "[rust-surgeon] REFUSED: no pipeline boundary contract",
                ),
            },
            _ => execute_component(component, &context),
        };
        let failed = outcome.is_failed();
        context = format!(
            "Original task:\n{prompt}\n\nPrevious component: {component}\nPrevious result:\n{}",
            outcome.output
        );
        outputs.push((component.to_string(), outcome));
        if failed {
            break;
        }
    }

    let rendered = render_arm(spec, &outputs);
    let outcome = aggregate(rendered, outputs.iter().map(|(_, outcome)| outcome));
    snapshot.finish(&outcome);
    outcome
}

pub fn run_pipeline(spec: &str, prompt: &str) -> String {
    run_pipeline_outcome(spec, prompt).output
}

pub fn run_pipeline_outcome(spec: &str, prompt: &str) -> ExecutionOutcome {
    let arms: Vec<_> = spec
        .split("||")
        .map(str::trim)
        .filter(|arm| !arm.is_empty())
        .collect();
    if arms.is_empty() {
        return ExecutionOutcome::failed("empty_pipeline", "Empty Octopus pipeline");
    }
    if arms.len() == 1 {
        return run_arm_outcome(arms[0], prompt);
    }

    let mut handles = Vec::new();
    for arm in &arms {
        let arm = arm.to_string();
        let prompt = prompt.to_string();
        handles.push(std::thread::spawn(move || {
            let outcome = run_arm_outcome(&arm, &prompt);
            (arm, outcome)
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.join().unwrap_or_else(|_| {
            (
                "failed-arm".to_string(),
                ExecutionOutcome::failed("arm_thread_panicked", "Composite arm thread panicked"),
            )
        }));
    }
    let rendered = render_pipeline(&results);
    aggregate(rendered, results.iter().map(|(_, outcome)| outcome))
}

pub fn list() -> Vec<&'static str> {
    let mut blades = blade::list();
    if !blades.contains(&"pipeline-architect") {
        blades.push("pipeline-architect");
    }
    blades
}

pub fn capabilities() -> Vec<CapabilityInfo> {
    capability::catalog(&list())
}

pub fn render_capabilities() -> String {
    capability::render(&list())
}

pub fn run_mcp() {
    mcp::run();
}

pub fn orch_init() {
    orchestration::init_from_disk();
}

pub fn orch_status(root_id: &str) -> ExecutionOutcome {
    match orchestration::get_root(root_id) {
        None => ExecutionOutcome::failed("root_not_found", format!("Root {root_id} not found")),
        Some(root) => {
            let arms = orchestration::list_events(root_id);
            let mut output = format!(
                "Root: {}  Status: {}  Prompt: {}  Input: {}",
                root.id,
                root.status.as_str(),
                &root.prompt_hash[..12],
                &root.input_hash[..12]
            );
            if let Some(ref hash) = root.output_hash {
                output.push_str(&format!("  Output: {}", &hash[..12]));
            }
            if let Some(finished) = root.finished_at {
                output.push_str(&format!("  Finished: {finished}"));
            }
            if let Some(dur) = root.duration_ms {
                output.push_str(&format!("  Duration: {dur}ms"));
            }
            output.push_str(&format!("\nEvents: {}", arms.len()));
            for arm in &arms {
                output.push_str(&format!(
                    "\n  {} [{}] {} ({})",
                    arm.arm_id,
                    arm.event_type,
                    &arm.details[..12.min(arm.details.len())],
                    arm.timestamp
                ));
            }
            ExecutionOutcome::completed(output)
        }
    }
}

pub fn orch_resume(root_id: &str) -> ExecutionOutcome {
    match orchestration::get_root(root_id) {
        None => ExecutionOutcome::failed("root_not_found", format!("Root {root_id} not found")),
        Some(root) => {
            if root.status != orchestration::ArmStatus::Running {
                return ExecutionOutcome::failed(
                    "not_running",
                    format!(
                        "Root {root_id} is not running (status: {})",
                        root.status.as_str()
                    ),
                );
            }
            match orchestration::find_orphaned_arms()
                .into_iter()
                .find(|a| a.root_id == root_id)
            {
                None => ExecutionOutcome::failed(
                    "no_orphans",
                    format!("No orphaned arms for root {root_id}"),
                ),
                Some(arm) => {
                    let resumed = orchestration::resume_arm(&arm.id)
                        .unwrap_or_else(|e| panic!("resume failed: {e:?}"));
                    ExecutionOutcome::completed(format!(
                        "Resumed arm {} for root {root_id} (status: {})",
                        resumed.id,
                        resumed.status.as_str()
                    ))
                }
            }
        }
    }
}

pub fn orch_retry(arm_id: &str) -> ExecutionOutcome {
    match orchestration::get_arm(arm_id) {
        None => ExecutionOutcome::failed("arm_not_found", format!("Arm {arm_id} not found")),
        Some(arm) => {
            if arm.status != orchestration::ArmStatus::Failed
                && arm.status != orchestration::ArmStatus::TimedOut
            {
                return ExecutionOutcome::failed(
                    "not_retryable",
                    format!(
                        "Arm {arm_id} status is {} (only failed/timed_out can be retried)",
                        arm.status.as_str()
                    ),
                );
            }
            let new_arm = orchestration::create_arm(
                &arm.root_id,
                &arm.name,
                &arm.prompt_hash,
                arm.parent_arm_id.as_deref(),
            );
            ExecutionOutcome::completed(format!(
                "Created retry arm {} for original {} (root: {})",
                new_arm.id, arm_id, arm.root_id
            ))
        }
    }
}

pub fn orch_cancel(root_id: &str) -> ExecutionOutcome {
    match orchestration::get_root(root_id) {
        None => ExecutionOutcome::failed("root_not_found", format!("Root {root_id} not found")),
        Some(root) => {
            if root.status == orchestration::ArmStatus::Completed
                || root.status == orchestration::ArmStatus::Cancelled
            {
                return ExecutionOutcome::failed(
                    "already_done",
                    format!("Root {root_id} is already {}", root.status.as_str()),
                );
            }
            let arms = orchestration::list_events(root_id);
            let mut cancelled = 0;
            for arm in &arms {
                if orchestration::cancel_arm(&arm.arm_id).is_ok() {
                    cancelled += 1;
                }
            }
            let outcome = ExecutionOutcome::failed(
                "cancelled",
                format!("Cancelled root {root_id} and {cancelled} arms"),
            );
            orchestration::finish_root(root_id, &outcome);
            outcome
        }
    }
}

pub fn orch_orphans() -> ExecutionOutcome {
    let orphans = orchestration::find_orphaned_arms();
    if orphans.is_empty() {
        return ExecutionOutcome::completed("No orphaned arms".to_string());
    }
    let mut output = format!("Orphaned arms: {}", orphans.len());
    for arm in &orphans {
        output.push_str(&format!(
            "\n  {} [{}] root={} ({})",
            arm.id,
            arm.name,
            arm.root_id,
            arm.status.as_str()
        ));
    }
    ExecutionOutcome::completed(output)
}

fn execute_component(spec: &str, prompt: &str) -> ExecutionOutcome {
    let candidates: Vec<_> = spec
        .split('|')
        .map(str::trim)
        .filter(|candidate| !candidate.is_empty())
        .collect();
    if candidates.is_empty() {
        return ExecutionOutcome::failed("empty_failover", "Empty blade failover chain");
    }

    let requested = candidates[0];
    let mut failures = Vec::new();
    let mut last_failure = None;

    for candidate in &candidates {
        let outcome = if let Some(outcome) = capability::execute(candidate, prompt) {
            outcome
        } else if registered_blade(candidate) {
            match std::panic::catch_unwind(|| blade::execute(candidate, prompt)) {
                Ok(output) if !output.trim().is_empty() => ExecutionOutcome::completed(output),
                Ok(_) => ExecutionOutcome::failed(
                    "empty_blade_output",
                    format!("{candidate}: empty output"),
                ),
                Err(_) => ExecutionOutcome::failed(
                    "blade_panicked",
                    format!("{candidate}: panic isolated"),
                ),
            }
        } else {
            ExecutionOutcome::failed("blade_unavailable", format!("{candidate}: unavailable"))
        };

        if !outcome.is_failed() {
            if *candidate == requested {
                return outcome;
            }
            return ExecutionOutcome::completed(format!(
                "──◆ failover {requested} -> {candidate} ──✓\n{}",
                outcome.output
            ));
        }

        failures.push(outcome.output.clone());
        last_failure = Some(outcome);
    }

    if candidates.len() == 1 {
        return last_failure.expect("single candidate must produce an outcome");
    }
    ExecutionOutcome::failed(
        "failover_exhausted",
        format!("Blade arm failed: {}", failures.join(", ")),
    )
}

fn registered_blade(name: &str) -> bool {
    blade::list().contains(&name)
        || matches!(
            name,
            "02_Memory_Skills"
                | "crispr_hotfix"
                | "synaptic_pruning"
                | "macrophage"
                | "list"
                | "--list"
                | "-l"
        )
}

fn aggregate<'a>(
    rendered: String,
    outcomes: impl IntoIterator<Item = &'a ExecutionOutcome>,
) -> ExecutionOutcome {
    if let Some(failure) = outcomes.into_iter().find(|outcome| outcome.is_failed()) {
        ExecutionOutcome::failed(
            failure.code.as_deref().unwrap_or("execution_failed"),
            rendered,
        )
    } else {
        ExecutionOutcome::completed(rendered)
    }
}

fn render_arm(spec: &str, outputs: &[(String, ExecutionOutcome)]) -> String {
    let mut hasher = Sha256::new();
    let mut rendered = format!("═══ COMPOSITE ARM: {spec} ═══\n");
    for (index, (component, outcome)) in outputs.iter().enumerate() {
        hasher.update(component.as_bytes());
        hasher.update(outcome.status.as_str().as_bytes());
        hasher.update(outcome.output.as_bytes());
        rendered.push_str(&format!(
            "\n── Component {}: {} [{}] ──\n{}\n",
            index + 1,
            component,
            outcome.status.as_str(),
            outcome.output
        ));
    }
    rendered.push_str(&format!(
        "\n═══ Arm Root: {} ═══",
        short_hash(hasher.finalize().as_slice())
    ));
    rendered
}

fn render_pipeline(results: &[(String, ExecutionOutcome)]) -> String {
    let mut hasher = Sha256::new();
    let mut rendered = format!("═══ OCTOPUS: {} COMPOSITE ARMS ═══\n", results.len());
    for (index, (arm, outcome)) in results.iter().enumerate() {
        hasher.update(arm.as_bytes());
        hasher.update(outcome.status.as_str().as_bytes());
        hasher.update(outcome.output.as_bytes());
        rendered.push_str(&format!(
            "\n━━ Arm {}: {} [{}] ━━\n{}\n",
            index + 1,
            arm,
            outcome.status.as_str(),
            outcome.output
        ));
    }
    rendered.push_str(&format!(
        "\n═══ Octopus Root: {} ═══",
        short_hash(hasher.finalize().as_slice())
    ));
    rendered
}

fn short_hash(bytes: &[u8]) -> String {
    hex::encode(bytes).chars().take(16).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_blade_is_a_typed_failure() {
        let outcome = execute_component("missing-blade", "input");
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("blade_unavailable"));
    }

    #[test]
    fn failover_returns_a_completed_typed_outcome() {
        let outcome = execute_component("missing-blade|code-reader", "fn main() {}");
        assert!(!outcome.is_failed());
        assert!(outcome
            .output
            .contains("failover missing-blade -> code-reader"));
    }

    #[test]
    fn empty_arm_is_a_typed_failure() {
        let outcome = run_arm_outcome("", "input");
        assert_eq!(outcome.code.as_deref(), Some("empty_arm"));
    }

    // V1.4: Real pure-algorithm blade integration tests

    #[test]
    fn summarize_extracts_key_sentences() {
        let input = "Rust is a systems programming language. It provides memory safety. It has zero-cost abstractions. It is used for performance-critical code. The compiler enforces ownership rules.";
        let outcome = run_outcome("summarize", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("Rust"));
    }

    #[test]
    fn summarize_empty_input() {
        let outcome = run_outcome("summarize", "");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("Empty input"));
    }

    #[test]
    fn sag_counts_occurrences() {
        let input = "rust ||| the rust compiler is fast and rust is safe";
        let outcome = run_outcome("sag", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("total=2"));
    }

    #[test]
    fn sag_wrong_format() {
        let outcome = run_outcome("sag", "no separator");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("Usage"));
    }

    #[test]
    fn diagnostics_analyzes_text() {
        let input = "fn main() {\n    println!(\"hello\");\n}";
        let outcome = run_outcome("diagnostics", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("karakterek="));
        assert!(outcome.output.contains("byte="));
    }

    #[test]
    fn diagnostics_empty_input() {
        let outcome = run_outcome("diagnostics", "");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("Üres bemenet"));
    }

    #[test]
    fn batch8_language_detection_works() {
        let outcome = run_outcome("polyglot", "fn main() { println!(\"hi\"); }");
        assert!(!outcome.is_failed());
        assert!(outcome.output.to_lowercase().contains("rust"));
    }

    #[test]
    fn batch8_circuit_breaker_works() {
        let outcome = run_outcome("circuit-breaker", "closed");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("CLOSED"));
    }

    #[test]
    fn batch9_code_review_works() {
        let input = "fn main() {\n    let x = 5;\n    println!(\"{}\", x);\n}";
        let outcome = run_outcome("code-review", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("code-review"));
    }

    #[test]
    fn batch10_geolocation_distance_works() {
        let input = "40.7128 -74.0060 51.5074 -0.1278";
        let outcome = run_outcome("geolocation-distance", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("km") || outcome.output.contains("distance"));
    }

    #[test]
    fn batch11_dna_extract_works() {
        let input = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let outcome = run_outcome("dna-extract", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("dna") || outcome.output.contains("extract"));
    }

    #[test]
    fn batch12_dual_generate_works() {
        let outcome = run_outcome("dual-generate", "test pattern");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("dual") || outcome.output.contains("generate"));
    }

    #[test]
    fn polyglot_detects_rust() {
        let outcome = run_outcome("polyglot", "fn main() { let x = 5; }");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("rust"));
    }

    #[test]
    fn circuit_breaker_closed() {
        let outcome = run_outcome("circuit-breaker", "closed");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("CLOSED"));
    }

    #[test]
    fn code_review_works() {
        let outcome = run_outcome("code-review", "fn main() {}");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("code-review"));
    }

    #[test]
    fn duplicate_detector_works() {
        let outcome = run_outcome("duplicate-detector", "let x = 5;\nlet y = 10;\nlet x = 5;");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("duplicate"));
    }

    #[test]
    fn code_quality_works() {
        let outcome = run_outcome("code-quality", "fn main() {\n    let x = 5;\n}");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("score="));
    }

    #[test]
    fn data_master_works() {
        let outcome = run_outcome("data-master", "1 2 3 4 5");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("count=5"));
    }

    #[test]
    fn retry_policy_works() {
        let outcome = run_outcome("retry-policy", "3 100");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("max_retries=3"));
    }

    #[test]
    fn orch_status_nonexistent_root_fails() {
        let outcome = orch_status("nonexistent-root-id");
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("root_not_found"));
    }

    #[test]
    fn orch_cancel_nonexistent_root_fails() {
        let outcome = orch_cancel("nonexistent-root-id");
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("root_not_found"));
    }

    #[test]
    fn orch_retry_nonexistent_arm_fails() {
        let outcome = orch_retry("nonexistent-arm-id");
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("arm_not_found"));
    }

    #[test]
    fn orch_orphans_returns_completed() {
        let outcome = orch_orphans();
        assert!(!outcome.is_failed());
    }
}
