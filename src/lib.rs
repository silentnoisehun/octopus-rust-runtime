pub mod approval;
pub mod arm_manifest;
pub mod bio;
mod bio_actuator;
pub mod bio_benchmark;
pub mod bio_system;
#[allow(clippy::all)]
mod blade;
mod capability;
mod composite;
pub mod contract;
pub mod enforcement;
pub mod external;
mod maintenance;
pub mod marshal;
mod mcp;
pub mod orchestration;
mod outcome;
mod process;
pub mod real_blades;
pub mod render;
pub mod resonance;
mod snapshot;
mod state_lock;
mod state_path;

pub use bio_benchmark::{run_benchmarks, BenchmarkConfig};
pub use capability::{
    CapabilityExecutionClass, CapabilityInfo, CapabilityMode, CapabilityProfile, CapabilityStatus,
    VerificationGrade,
};
pub use contract::CapabilityContract;
pub use outcome::{ExecutionOutcome, ExecutionStatus};

pub use orchestration::{recent_events, EventEntry};

pub fn marshal_plan(task: &str) -> ExecutionOutcome {
    marshal::plan_outcome(task)
}

pub fn marshal_dispatch(task: &str, allow_write: bool) -> ExecutionOutcome {
    marshal::dispatch_outcome(task, allow_write)
}

pub fn resonance_status(verify: bool, tail: usize) -> ExecutionOutcome {
    resonance::status_outcome(verify, tail)
}

fn run_control_action(
    name: &str,
    prompt: String,
    action: impl FnOnce() -> ExecutionOutcome,
) -> ExecutionOutcome {
    let root = orchestration::create_root(&prompt);
    if !orchestration::root_snapshot_is_durable(&root.id) {
        return ExecutionOutcome::failed(
            "control_audit_unavailable",
            "refusing control action because the root audit snapshot is not durable",
        );
    }
    let arm = orchestration::create_arm_restricted(&root.id, name, &prompt, None);
    if !orchestration::arm_snapshot_is_durable(&arm.id) {
        let outcome = ExecutionOutcome::failed(
            "control_audit_unavailable",
            "refusing control action because the arm audit snapshot is not durable",
        );
        orchestration::finish_root(&root.id, &outcome);
        return outcome;
    }
    let outcome = action();
    orchestration::finish_arm(&arm.id, &outcome);
    orchestration::finish_root(&root.id, &outcome);
    outcome
}

pub fn bio_macrophage_plan(pid: u32) -> ExecutionOutcome {
    run_control_action("bio-macrophage-plan", format!("pid={pid}"), || {
        bio_actuator::macrophage_plan(pid)
    })
}

pub fn bio_macrophage_apply(pid: u32, confirm: &str, allow_kill: bool) -> ExecutionOutcome {
    run_control_action(
        "bio-macrophage-apply",
        format!("pid={pid};confirm-sha256={}", sha256_text(confirm)),
        || bio_actuator::macrophage_apply(pid, confirm, allow_kill),
    )
}

pub fn bio_synaptic_plan(
    executable: &std::path::Path,
    config: &std::path::Path,
) -> ExecutionOutcome {
    run_control_action(
        "bio-synaptic-plan",
        format!(
            "executable={};config={}",
            executable.display(),
            config.display()
        ),
        || bio_actuator::synaptic_plan(executable, config),
    )
}

pub fn bio_synaptic_apply(
    executable: &std::path::Path,
    config: &std::path::Path,
    confirm: &str,
    allow_write: bool,
) -> ExecutionOutcome {
    run_control_action(
        "bio-synaptic-apply",
        format!(
            "executable={};config={};confirm-sha256={}",
            executable.display(),
            config.display(),
            sha256_text(confirm)
        ),
        || bio_actuator::synaptic_apply(executable, config, confirm, allow_write),
    )
}

pub fn bio_crispr_plan(
    target: &std::path::Path,
    replacement: &std::path::Path,
    health_args: &[String],
) -> ExecutionOutcome {
    run_control_action(
        "bio-crispr-plan",
        format!(
            "target={};replacement={};health={}",
            target.display(),
            replacement.display(),
            health_args.join(" ")
        ),
        || bio_actuator::crispr_plan(target, replacement, health_args),
    )
}

pub fn bio_crispr_apply(
    target: &std::path::Path,
    replacement: &std::path::Path,
    health_args: &[String],
    confirm: &str,
    allow_write: bool,
) -> ExecutionOutcome {
    run_control_action(
        "bio-crispr-apply",
        format!(
            "target={};replacement={};health={};confirm-sha256={}",
            target.display(),
            replacement.display(),
            health_args.join(" "),
            sha256_text(confirm)
        ),
        || bio_actuator::crispr_apply(target, replacement, health_args, confirm, allow_write),
    )
}

pub fn bio_system_status() -> ExecutionOutcome {
    run_control_action(
        "bio-system-status",
        "bio-binaries-status".to_string(),
        bio_system::status,
    )
}

pub fn bio_external_run(name: &str, args: &str, allow_mutation: bool) -> ExecutionOutcome {
    run_control_action(
        "bio-external-run",
        format!(
            "name={name};allow-mutation={allow_mutation};args-sha256={}",
            sha256_text(args)
        ),
        || bio_system::external::execute(name, args, allow_mutation),
    )
}

fn sha256_text(value: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(value.as_bytes()))
}

pub struct StateCommandGuard {
    _inner: state_lock::StateLockGuard,
}

pub fn state_command_guard() -> Result<StateCommandGuard, String> {
    state_lock::acquire_shared(&state_path::state_dir(), state_lock::configured_timeout())
        .map(|guard| StateCommandGuard { _inner: guard })
}

pub fn state_exclusive_command_guard() -> Result<StateCommandGuard, String> {
    state_lock::acquire_exclusive(&state_path::state_dir(), state_lock::configured_timeout())
        .map(|guard| StateCommandGuard { _inner: guard })
}

pub fn run(blade_name: &str, prompt: &str) -> String {
    run_outcome(blade_name, prompt).output
}

pub fn run_outcome(blade_name: &str, prompt: &str) -> ExecutionOutcome {
    if matches!(blade_name, "pipeline-architect" | "rust-surgeon") {
        return run_arm_outcome(blade_name, prompt);
    }
    // Create orchestration root for top-level blade calls
    let root = orchestration::create_root(prompt);
    let root_id = root.id.clone();
    let arm = orchestration::create_arm_restricted(&root_id, blade_name, prompt, None);
    let arm_id = arm.id.clone();

    let outcome = execute_blade_under_root(&root_id, blade_name, prompt, Some(&root_id));

    orchestration::finish_arm(&arm_id, &outcome);
    orchestration::finish_root(&root_id, &outcome);
    outcome
}

/// Execute a single blade under an existing orchestration root. Does NOT create a new root.
/// Creates a snapshot, runs the blade, finishes the snapshot, returns the outcome.
fn execute_blade_under_root(
    _root_id: &str,
    blade_name: &str,
    prompt: &str,
    parent_id: Option<&str>,
) -> ExecutionOutcome {
    let snapshot_result = snapshot::ArmSnapshot::try_start(blade_name, prompt, parent_id);
    let outcome = match snapshot_result {
        Ok(mut snap) => {
            let outcome = execute_component(blade_name, prompt);
            if let Err(e) = snap.try_finish(&outcome) {
                return ExecutionOutcome::failed(
                    "snapshot_finish_failed",
                    format!("[{blade_name}] snapshot finish failed: {e}"),
                );
            }
            outcome
        }
        Err(error) => {
            return ExecutionOutcome::failed(
                "snapshot_io_failed",
                format!("[{blade_name}] snapshot start failed: {error}"),
            );
        }
    };
    outcome
}

pub fn run_arm(spec: &str, prompt: &str) -> String {
    run_arm_outcome(spec, prompt).output
}

pub fn run_arm_outcome(spec: &str, prompt: &str) -> ExecutionOutcome {
    let root = orchestration::create_root(prompt);
    let root_id = root.id.clone();
    let arm = orchestration::create_arm_restricted(&root_id, spec, prompt, None);
    let arm_id = arm.id.clone();
    let outcome = execute_arm_under_root(&root_id, spec, prompt, Some(&root_id));
    orchestration::finish_arm(&arm_id, &outcome);
    orchestration::finish_root(&root_id, &outcome);
    outcome
}

/// Execute a composite arm under an existing root. Does NOT create a new root.
/// Handles `+`-separated components sequentially. Used by pipeline threads and resume/retry.
fn execute_arm_under_root(
    root_id: &str,
    spec: &str,
    prompt: &str,
    parent_id: Option<&str>,
) -> ExecutionOutcome {
    execute_arm_under_root_gated(root_id, spec, prompt, parent_id, None)
}

fn execute_arm_under_root_gated(
    root_id: &str,
    spec: &str,
    prompt: &str,
    parent_id: Option<&str>,
    manifest_arm: Option<&arm_manifest::ManifestArm>,
) -> ExecutionOutcome {
    let components: Vec<_> = spec
        .split('+')
        .map(str::trim)
        .filter(|component| !component.is_empty())
        .collect();
    if components.is_empty() {
        return ExecutionOutcome::failed("empty_arm", "Empty composite arm");
    }

    let snapshot_result = snapshot::ArmSnapshot::try_start(spec, prompt, parent_id);
    let mut outputs = Vec::new();
    let mut outcome = ExecutionOutcome::failed("snapshot_io_failed", "snapshot failed");
    if let Ok(mut snap) = snapshot_result {
        let mut context = prompt.to_string();
        let mut boundary = None;
        for component in &components {
            outcome = match *component {
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
        let rendered = render_arm(root_id, spec, &outputs);
        outcome = aggregate(rendered, outputs.iter().map(|(_, o)| o));
        if let Some(arm) = manifest_arm {
            outcome = arm_manifest::enforce_evidence(arm, outcome);
        }
        if let Err(e) = snap.try_finish(&outcome) {
            return ExecutionOutcome::failed(
                "snapshot_finish_failed",
                format!("[{root_id}] snapshot finish failed: {e}"),
            );
        }
    }
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

    // Create exactly ONE orchestration root for the pipeline
    let root = orchestration::create_root(prompt);
    let root_id = root.id.clone();

    // Spawn parallel arms using rootless inner executor
    let mut handles = Vec::new();
    for arm_spec in &arms {
        let arm_spec = arm_spec.to_string();
        let prompt = prompt.to_string();
        let rid = root_id.clone();
        handles.push(std::thread::spawn(move || {
            // Create arm record linked to the pipeline root
            let arm = orchestration::create_arm_restricted(&rid, &arm_spec, &prompt, None);
            let arm_id = arm.id.clone();
            // Use rootless arm executor â€” does NOT create a new root
            let outcome = execute_arm_under_root(&rid, &arm_spec, &prompt, Some(&arm_id));
            orchestration::finish_arm(&arm_id, &outcome);
            (arm_spec, outcome)
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

    let rendered = render_pipeline(&root_id, &results);
    let pipeline_outcome = aggregate(rendered, results.iter().map(|(_, outcome)| outcome));
    orchestration::finish_root(&root_id, &pipeline_outcome);
    pipeline_outcome
}

pub fn run_manifest_outcome(source: &str, allow_write: bool) -> ExecutionOutcome {
    let manifest = match arm_manifest::parse_and_validate(source, allow_write) {
        Ok(manifest) => manifest,
        Err(error) => return ExecutionOutcome::failed(error.code, error.message),
    };

    let root = orchestration::create_root(&manifest.objective);
    let root_id = root.id.clone();
    let mut handles = Vec::new();

    for arm in manifest.arms.clone() {
        let rid = root_id.clone();
        handles.push(std::thread::spawn(move || {
            let record_name = format!("{}:{}", arm.id, arm.spec);
            let record = orchestration::create_arm_restricted(&rid, &record_name, &arm.input, None);
            let outcome = execute_arm_under_root_gated(
                &rid,
                &arm.spec,
                &arm.input,
                Some(&record.id),
                Some(&arm),
            );
            orchestration::finish_arm(&record.id, &outcome);
            (record_name, outcome)
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.join().unwrap_or_else(|_| {
            (
                "failed-manifest-arm".to_string(),
                ExecutionOutcome::failed("arm_thread_panicked", "Manifest arm thread panicked"),
            )
        }));
    }

    let rendered = render_manifest(&root_id, &manifest.objective, &results);
    let outcome = aggregate(rendered, results.iter().map(|(_, outcome)| outcome));
    orchestration::finish_root(&root_id, &outcome);
    outcome
}

const RUNTIME_ONLY_CAPABILITIES: &[&str] = &[
    "1password",
    "apple-notes",
    "bear-notes",
    "claude-migration",
    "clawhub",
    "discord",
    "eightctl",
    "forge-blade",
    "github-manager",
    "git-nexus",
    "gog",
    "goplaces",
    "hello-mate",
    "himalaya",
    "incubator",
    "lobster-scraper",
    "mcporter",
    "merge-pr",
    "merge-pr-v1",
    "macrophage",
    "mitosis-agent",
    "nano-pdf",
    "notion",
    "openai-whisper",
    "pptx",
    "sherpa-onnx-tts",
    "stt-ear",
    "tmux",
    "tts-voice",
    "turborepo",
    "video-frames",
    "voice-call",
    "wacli",
    "weather",
    "pipeline-architect",
];

pub fn list() -> Vec<&'static str> {
    let mut blades = blade::list();
    for capability in RUNTIME_ONLY_CAPABILITIES {
        if !blades.contains(capability) {
            blades.push(capability);
        }
    }
    for capability in bio_system::public_names() {
        if !blades.contains(&capability) {
            blades.push(capability);
        }
    }
    blades
}

pub fn capabilities() -> Vec<CapabilityInfo> {
    capability::catalog(&list())
}

pub fn capabilities_for_profile(profile: CapabilityProfile) -> Vec<CapabilityInfo> {
    capability::catalog_for_profile(&list(), profile)
}

pub fn render_capabilities() -> String {
    capability::render(&list())
}

pub fn render_capabilities_for_profile(profile: CapabilityProfile) -> String {
    capability::render_for_profile(&list(), profile)
}

pub fn run_mcp() {
    mcp::run();
}

pub fn state_audit(stale_minutes: u64) -> ExecutionOutcome {
    if stale_minutes == 0 {
        return ExecutionOutcome::failed(
            "invalid_stale_window",
            "stale-minutes must be at least 1",
        );
    }
    match maintenance::audit(std::time::Duration::from_secs(
        stale_minutes.saturating_mul(60),
    )) {
        Ok(report) => ExecutionOutcome::completed(report.render()),
        Err(error) => ExecutionOutcome::failed("state_audit_failed", error),
    }
}

pub fn state_repair(stale_minutes: u64) -> ExecutionOutcome {
    if stale_minutes == 0 {
        return ExecutionOutcome::failed(
            "invalid_stale_window",
            "stale-minutes must be at least 1",
        );
    }
    match maintenance::repair(std::time::Duration::from_secs(
        stale_minutes.saturating_mul(60),
    )) {
        Ok(report) => ExecutionOutcome::completed(report.render()),
        Err(error) => ExecutionOutcome::failed("state_repair_failed", error),
    }
}

pub fn state_backup_create() -> ExecutionOutcome {
    match maintenance::create_backup() {
        Ok(path) => {
            let Some(backup_id) = path.file_name().and_then(|value| value.to_str()) else {
                return ExecutionOutcome::failed(
                    "state_backup_failed",
                    "created backup has no valid identifier",
                );
            };
            match maintenance::verify_backup(backup_id) {
                Ok(report) if report.sealed => ExecutionOutcome::completed(format!(
                    "OCTOPUS STATE BACKUP CREATE\nbackup: {}\npath: {}\nsealed: true\nfiles: {}\nbytes: {}\nintegrity: verified",
                    report.backup_id,
                    path.display(),
                    report.files,
                    report.bytes,
                )),
                Ok(_) => ExecutionOutcome::failed(
                    "state_backup_unsealed",
                    "new backup was published without a sealed manifest",
                ),
                Err(error) => ExecutionOutcome::failed("state_backup_verify_failed", error),
            }
        }
        Err(error) => ExecutionOutcome::failed("state_backup_failed", error),
    }
}

pub fn state_backup_verify(backup_id: &str) -> ExecutionOutcome {
    match maintenance::verify_backup(backup_id) {
        Ok(report) => ExecutionOutcome::completed(report.render()),
        Err(error) => ExecutionOutcome::failed("state_backup_verify_failed", error),
    }
}

pub fn state_restore_plan(backup_id: &str) -> ExecutionOutcome {
    match maintenance::plan_restore(backup_id) {
        Ok(report) => ExecutionOutcome::completed(report.render()),
        Err(error) => ExecutionOutcome::failed("state_restore_plan_failed", error),
    }
}

pub fn state_restore_apply(backup_id: &str, confirmation: &str) -> ExecutionOutcome {
    match maintenance::restore_backup(backup_id, confirmation) {
        Ok(report) => {
            orchestration::init_from_disk();
            ExecutionOutcome::completed(report.render())
        }
        Err(error) => ExecutionOutcome::failed("state_restore_failed", error),
    }
}

pub fn state_restore_recover() -> ExecutionOutcome {
    match maintenance::recover_interrupted_restore() {
        Ok(report) => {
            if report != maintenance::RestoreRecoveryReport::None {
                orchestration::init_from_disk();
            }
            ExecutionOutcome::completed(report.render())
        }
        Err(error) => ExecutionOutcome::failed("state_restore_recovery_failed", error),
    }
}

pub fn state_restore_auto_recover() -> ExecutionOutcome {
    state_restore_recover()
}

pub fn orch_init() {
    orchestration::init_from_disk();
}

pub fn orch_status(root_id: &str) -> ExecutionOutcome {
    match orchestration::get_root(root_id) {
        None => ExecutionOutcome::failed("root_not_found", format!("Root {root_id} not found")),
        Some(root) => {
            let arm_records: Vec<orchestration::ArmRecord> = root
                .children
                .iter()
                .filter_map(|child_id| orchestration::get_arm(child_id))
                .collect();
            let output = render::render_status_octopus(&root, &arm_records);
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
            let orphaned = orchestration::find_orphaned_arms()
                .into_iter()
                .filter(|a| a.root_id == root_id)
                .collect::<Vec<_>>();
            if orphaned.is_empty() {
                return ExecutionOutcome::failed(
                    "no_orphans",
                    format!("No orphaned arms for root {root_id}"),
                );
            }
            let mut resumed = 0usize;
            let mut failures = Vec::new();
            for arm in &orphaned {
                // Mark arm as resumed
                match orchestration::resume_arm(&arm.id) {
                    Ok(_) => {
                        // Actually dispatch the work using the stored prompt
                        let outcome = execute_component(&arm.name, &arm.prompt);
                        orchestration::finish_arm(&arm.id, &outcome);
                        if outcome.is_failed() {
                            failures.push((arm.id.clone(), outcome.code.unwrap_or_default()));
                        }
                        resumed += 1;
                    }
                    Err(e) => {
                        failures.push((arm.id.clone(), e.code.unwrap_or_default()));
                    }
                }
            }
            if failures.is_empty() {
                let outcome = ExecutionOutcome::completed(format!(
                    "Resumed and completed {} arms for root {root_id}",
                    resumed
                ));
                orchestration::finish_root(root_id, &outcome);
                outcome
            } else {
                let outcome = ExecutionOutcome::failed(
                    "resume_failed",
                    format!(
                        "Resumed {resumed} arms for root {root_id}, {} failed: {:?}",
                        failures.len(),
                        failures
                            .iter()
                            .map(|(id, _)| id.as_str())
                            .collect::<Vec<_>>()
                    ),
                );
                orchestration::finish_root(root_id, &outcome);
                outcome
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
            // Create a new arm with proper parent link
            let new_arm =
                orchestration::create_arm(&arm.root_id, &arm.name, &arm.prompt, Some(arm_id));
            // Actually execute the retry using the stored prompt
            let outcome = execute_component(&arm.name, &arm.prompt);
            orchestration::finish_arm(&new_arm.id, &outcome);
            if outcome.is_failed() {
                outcome
            } else {
                ExecutionOutcome::completed(format!(
                    "Retry arm {} completed for original {} (root: {})",
                    new_arm.id, arm_id, arm.root_id
                ))
            }
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
            let children = root.children.clone();
            let mut cancelled = 0;
            let mut not_cancellable = 0usize;
            for child_id in &children {
                match orchestration::get_arm(child_id) {
                    Some(arm) if arm.status == orchestration::ArmStatus::Running => {
                        // Mark arm as cancelled in state (no process-level kill; blades run synchronously in this process)
                        if orchestration::cancel_arm(child_id).is_ok() {
                            cancelled += 1;
                        } else {
                            not_cancellable += 1;
                        }
                    }
                    _ => {}
                }
            }
            let outcome = if cancelled > 0 {
                ExecutionOutcome::failed(
                    "cancelled",
                    format!("Cancelled root {root_id}: {cancelled} arms cancelled, {not_cancellable} not cancellable"),
                )
            } else {
                ExecutionOutcome::failed(
                    "cancellation_not_supported",
                    format!("Root {root_id}: no running arms could be cancelled ({not_cancellable} arms not cancellable)"),
                )
            };
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
    // Fail-closed commitment gate: when active, a blade only reaches the
    // native executor if the Microscope state says Allow. A Deny or a
    // load/gate error returns before any blade executor is invoked.
    if let Some(cfg) = crate::enforcement::EnforcementConfig::from_env() {
        match crate::enforcement::gate(spec, prompt, &cfg) {
            Ok(crate::enforcement::Gate::Allow) => {}
            Ok(crate::enforcement::Gate::Deny(reason)) => {
                return ExecutionOutcome::failed(
                    "enforcement_blocked",
                    format!("[{spec}] refused by commitment gate: {reason}"),
                );
            }
            Err(reason) => {
                return ExecutionOutcome::failed(
                    "enforcement_fail_closed",
                    format!("[{spec}] refused (fail-closed): {reason}"),
                );
            }
        }
    }

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
    list().contains(&name)
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

fn render_arm(root_id: &str, spec: &str, outputs: &[(String, ExecutionOutcome)]) -> String {
    let mut rendered = format!("═══ COMPOSITE ARM: {spec} ═══\n");
    for (index, (component, outcome)) in outputs.iter().enumerate() {
        rendered.push_str(&format!(
            "\n── Component {}: {} [{}] ──\n{}\n",
            index + 1,
            component,
            outcome.status.as_str(),
            outcome.output
        ));
    }
    rendered.push_str(&format!("\n═══ Arm Root: {root_id} ═══"));
    rendered
}

fn render_pipeline(root_id: &str, results: &[(String, ExecutionOutcome)]) -> String {
    let mut rendered = format!("═══ OCTOPUS: {} COMPOSITE ARMS ═══\n", results.len());
    for (index, (arm, outcome)) in results.iter().enumerate() {
        rendered.push_str(&format!(
            "\n━━ Arm {}: {} [{}] ━━\n{}\n",
            index + 1,
            arm,
            outcome.status.as_str(),
            outcome.output
        ));
    }
    rendered.push_str(&format!("\n═══ Octopus Root: {root_id} ═══"));
    rendered
}

fn render_manifest(
    root_id: &str,
    objective: &str,
    results: &[(String, ExecutionOutcome)],
) -> String {
    let mut rendered = format!(
        "EVIDENCE-BOUND OCTOPUS MANIFEST\nschema: {}\nroot: {root_id}\nobjective: {objective}\narms: {}\n",
        arm_manifest::SCHEMA,
        results.len()
    );
    for (index, (arm, outcome)) in results.iter().enumerate() {
        rendered.push_str(&format!(
            "\n-- Arm {}: {} [{}] --\n{}\n",
            index + 1,
            arm,
            outcome.status.as_str(),
            outcome.output
        ));
    }
    rendered.push_str(&format!("\nMANIFEST ROOT: {root_id}"));
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test setup: enable dev bypass so tests can run without enforcement state.
    // This is the explicit, audited development bypass path.
    fn ensure_dev_bypass() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            std::env::set_var("OCTOPUS_ENFORCE", "0");
            std::env::set_var("OCTOPUS_DEV_MODE", "1");
        });
    }

    #[test]
    fn unknown_blade_is_a_typed_failure() {
        ensure_dev_bypass();
        let outcome = execute_component("missing-blade", "input");
        assert!(outcome.is_failed());
        // Unknown blades are classified as Unavailable by the capability registry
        assert_eq!(outcome.code.as_deref(), Some("capability_unavailable"));
    }

    #[test]
    fn failover_returns_a_completed_typed_outcome() {
        ensure_dev_bypass();
        let outcome = execute_component("missing-blade|code-reader", "fn main() {}");
        assert!(!outcome.is_failed());
        assert!(outcome
            .output
            .contains("failover missing-blade -> code-reader"));
    }

    #[test]
    fn empty_arm_is_a_typed_failure() {
        ensure_dev_bypass();
        let outcome = run_arm_outcome("", "input");
        assert_eq!(outcome.code.as_deref(), Some("empty_arm"));
    }

    // V1.4: Real pure-algorithm blade integration tests

    #[test]
    fn summarize_extracts_key_sentences() {
        ensure_dev_bypass();
        let input = "Rust is a systems programming language. It provides memory safety. It has zero-cost abstractions. It is used for performance-critical code. The compiler enforces ownership rules.";
        let outcome = run_outcome("summarize", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("Rust"));
    }

    #[test]
    fn summarize_empty_input() {
        ensure_dev_bypass();
        let outcome = run_outcome("summarize", "");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("Empty input"));
    }

    #[test]
    fn sag_counts_occurrences() {
        ensure_dev_bypass();
        let input = "rust ||| the rust compiler is fast and rust is safe";
        let outcome = run_outcome("sag", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("total=2"));
    }

    #[test]
    fn sag_wrong_format() {
        ensure_dev_bypass();
        let outcome = run_outcome("sag", "no separator");
        // Wrong format input returns a typed failure (usage error)
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("blade_execution_failed"));
    }

    #[test]
    fn diagnostics_analyzes_text() {
        ensure_dev_bypass();
        let input = "fn main() {\n    println!(\"hello\");\n}";
        let outcome = run_outcome("diagnostics", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("karakterek="));
        assert!(outcome.output.contains("byte="));
    }

    #[test]
    fn diagnostics_empty_input() {
        ensure_dev_bypass();
        let outcome = run_outcome("diagnostics", "");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("Üres bemenet"));
    }

    #[test]
    fn batch8_language_detection_works() {
        ensure_dev_bypass();
        let outcome = run_outcome("polyglot", "fn main() { println!(\"hi\"); }");
        assert!(!outcome.is_failed());
        assert!(outcome.output.to_lowercase().contains("rust"));
    }

    #[test]
    fn batch8_circuit_breaker_works() {
        ensure_dev_bypass();
        let outcome = run_outcome("circuit-breaker", "closed");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("CLOSED"));
    }

    #[test]
    fn batch9_code_review_works() {
        ensure_dev_bypass();
        let input = "fn main() {\n    let x = 5;\n    println!(\"{}\", x);\n}";
        let outcome = run_outcome("code-review", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("code-review"));
    }

    #[test]
    fn batch10_geolocation_distance_works() {
        ensure_dev_bypass();
        let input = "40.7128 -74.0060 51.5074 -0.1278";
        let outcome = run_outcome("geolocation-distance", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("km") || outcome.output.contains("distance"));
    }

    #[test]
    fn batch11_dna_extract_works() {
        ensure_dev_bypass();
        let input = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let outcome = run_outcome("dna-extract", input);
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("dna") || outcome.output.contains("extract"));
    }

    #[test]
    fn batch12_dual_generate_works() {
        ensure_dev_bypass();
        let outcome = run_outcome("dual-generate", "test pattern");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("dual") || outcome.output.contains("generate"));
    }

    #[test]
    fn polyglot_detects_rust() {
        ensure_dev_bypass();
        let outcome = run_outcome("polyglot", "fn main() { let x = 5; }");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("rust"));
    }

    #[test]
    fn circuit_breaker_closed() {
        ensure_dev_bypass();
        let outcome = run_outcome("circuit-breaker", "closed");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("CLOSED"));
    }

    #[test]
    fn code_review_works() {
        ensure_dev_bypass();
        let outcome = run_outcome("code-review", "fn main() {}");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("code-review"));
    }

    #[test]
    fn duplicate_detector_works() {
        ensure_dev_bypass();
        let outcome = run_outcome("duplicate-detector", "let x = 5;\nlet y = 10;\nlet x = 5;");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("duplicate"));
    }

    #[test]
    fn code_quality_works() {
        ensure_dev_bypass();
        let outcome = run_outcome("code-quality", "fn main() {\n    let x = 5;\n}");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("score="));
    }

    #[test]
    fn data_master_works() {
        ensure_dev_bypass();
        let outcome = run_outcome("data-master", "1 2 3 4 5");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("count=5"));
    }

    #[test]
    fn retry_policy_works() {
        ensure_dev_bypass();
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
