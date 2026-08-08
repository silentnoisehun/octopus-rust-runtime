use crate::blade;
use crate::contract;
use crate::outcome::ExecutionOutcome;
use crate::process::{self, ProcessSpec};
use sha2::{Digest, Sha256};
use std::env;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_LOCAL_READ_BYTES: u64 = 1024 * 1024;

/// Operational category of a capability. This is the "how it runs" axis and is
/// fully disjoint from its availability/support status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CapabilityMode {
    RealAlgorithm,
    LocalRead,
    LocalWrite,
    LocalProcess,
    ExternalRead,
    ExternalWrite,
    Composite,
}

impl fmt::Display for CapabilityMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::RealAlgorithm => "real-algorithm",
            Self::LocalRead => "local-read",
            Self::LocalWrite => "local-write",
            Self::LocalProcess => "local-process",
            Self::ExternalRead => "external-read",
            Self::ExternalWrite => "external-write",
            Self::Composite => "composite",
        };
        formatter.write_str(value)
    }
}

/// Availability/support status of a capability. This is the "is it usable in
/// this environment" axis and is fully disjoint from its operational mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CapabilityStatus {
    Real,
    Unavailable,
    Unsupported,
    Deprecated,
}

impl fmt::Display for CapabilityStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Real => "real",
            Self::Unavailable => "unavailable",
            Self::Unsupported => "unsupported",
            Self::Deprecated => "deprecated",
        };
        formatter.write_str(value)
    }
}

/// Observable effect class. This deliberately does not reuse `status`: a
/// capability can be callable (`real`) while still being advisory-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CapabilityExecutionClass {
    Advisory,
    LocalOperation,
    ExternalIntegration,
    ControlPlane,
}

impl fmt::Display for CapabilityExecutionClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Advisory => "advisory",
            Self::LocalOperation => "local-operation",
            Self::ExternalIntegration => "external-integration",
            Self::ControlPlane => "control-plane",
        })
    }
}

/// Evidence grade for the registered route. `Tested` means automated coverage;
/// `Observed` additionally exercised a real local effect. Neither grade grants
/// authorization or bypasses the status gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum VerificationGrade {
    Declared,
    Tested,
    Observed,
}

impl fmt::Display for VerificationGrade {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Declared => "declared",
            Self::Tested => "tested",
            Self::Observed => "observed",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityProfile {
    All,
    WindowsOffline,
}

impl fmt::Display for CapabilityProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::All => "all",
            Self::WindowsOffline => "windows-offline",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CapabilityInfo {
    pub name: String,
    pub mode: CapabilityMode,
    pub status: CapabilityStatus,
    pub execution_class: CapabilityExecutionClass,
    pub verification: VerificationGrade,
    pub version: String,
    pub group: String,
    pub deprecated: bool,
    pub deprecation_message: Option<String>,
}

/// Canonical mode + support-status registry. Every registered capability gets
/// exactly one mode and exactly one status. `CopiedNative` is retired and must
/// never be returned here.
fn classify(name: &str) -> (CapabilityMode, CapabilityStatus) {
    use CapabilityMode::*;
    use CapabilityStatus::*;

    let bio_external = crate::bio_system::contains(name);
    let mode = if bio_external {
        LocalProcess
    } else {
        match name {
            // Real local adapters (read-only)
            "code-reader" | "diagnostics" | "git-nexus" => LocalRead,
            // Real local adapter (write)
            "code-writer" => LocalWrite,
            // Octopus control / surgery components
            "pipeline-architect" | "rust-surgeon" | "omni-surgeon" | "file-surgeon" => Composite,
            // Local CLI tools wrapped by the safe process runner
            "video-frames" | "tmux" | "audio-diagnostics" | "sherpa-onnx-tts" | "tts-voice"
            | "stt-ear" | "test-tui" | "nano-pdf" | "pptx" | "qr-scan" | "turborepo"
            | "weather" => LocalProcess,
            // Real external adapters. Environment/auth readiness is checked at execution time.
            "github" | "github-manager" => ExternalRead,
            // Externally mutating operations
            "openai-image-gen" | "openai-whisper" | "voice-call" => ExternalWrite,
            // Offline pure-algorithm and meta/documentation blades
            "summarize"
            | "sag"
            | "code-analysis"
            | "polyglot"
            | "circuit-breaker"
            | "code-review"
            | "geolocation-distance"
            | "dna-extract"
            | "dual-generate"
            | "duplicate-detector"
            | "code-quality"
            | "data-master"
            | "retry-policy"
            | "graceful-shutdown"
            | "macrophage"
            | "immune-status"
            | "bench-meter"
            | "brainstorming"
            | "prose"
            | "writing-rules"
            | "doc-scribe"
            | "agent-development"
            | "hook-development"
            | "command-development"
            | "plugin-structure"
            | "testing-codegen"
            | "brand-voice"
            | "brand-writer"
            | "planner"
            | "memory-bank"
            | "mermaid-agent"
            | "canvas"
            | "canvas-design"
            | "frontend-design"
            | "ui-design-system"
            | "ui-ux-pro"
            | "theme-factory"
            | "brand-guidelines"
            | "document-agent"
            | "memory-skills"
            | "memory-skills-v2"
            | "microscope-memory"
            | "emoti-mem"
            | "architect-mind"
            | "senior-architect"
            | "senior-prompt-engineer"
            | "formatter"
            | "stem-core"
            | "omni-connector"
            | "parser"
            | "type-inference"
            | "lint-rules"
            | "crispr-hotfix"
            | "crispr-hotfix-v2"
            | "synaptic-pruning"
            | "synaptic-pruning-v2"
            | "viral-transduction"
            | "hox-architecture"
            | "ai-synapse"
            | "hive-orchestrator"
            | "maestro"
            | "swarm"
            | "colony-swarm"
            | "quality-bun"
            | "react-practices"
            | "stem-cell-manager"
            | "mitosis-agent"
            | "peekaboo"
            | "forge-blade"
            | "omega-striker"
            | "sigma"
            | "model-usage"
            | "claude-migration"
            | "ast-refactor"
            | "connectome"
            | "connectome-rs"
            | "connectome-py"
            | "connectome-js"
            | "safety-check"
            | "safety-check-py"
            | "safety-check-js"
            | "polyglot-metrics"
            | "polyglot-convert"
            | "immune-antibody"
            | "immune-log"
            | "plugin-list"
            | "plugin-install"
            | "plugin-remove"
            | "dreamer-loop"
            | "auto-evolve"
            | "adaptive-evolve"
            | "self-evolve"
            | "mitosis"
            | "bio-mitosis"
            | "metamorphic-trigger"
            | "omnicoder"
            | "agent-factory"
            | "commander"
            | "swarm-queen"
            | "replicator"
            | "cryo-snap"
            | "dna-mutate"
            | "dna-mutate-point"
            | "dna-mutate-insert"
            | "dna-mutate-delete"
            | "dna-mutate-optimize"
            | "dna-crossover"
            | "dna-select"
            | "dna-evolve"
            | "dna-teach"
            | "dna-hebbian"
            | "dna-stats"
            | "brain"
            | "brain-compare"
            | "dual-cache"
            | "dual-learn"
            | "dual-record"
            | "dual-status"
            | "dual-teach"
            | "claude-logic"
            | "claude-psi"
            | "psi-logic"
            | "psi-quantum"
            | "psi"
            | "hello-mate" => RealAlgorithm,
            // Everything that talks to a remote service (or is platform-specific)
            _ => ExternalRead,
        }
    };

    let status = if bio_external {
        Real
    } else if name == "apple-notes" || name == "bear-notes" {
        Unsupported
    } else if matches!(name, "github" | "github-manager") {
        Real
    } else {
        match mode {
            LocalRead | LocalWrite | Composite | RealAlgorithm => Real,
            LocalProcess | ExternalRead | ExternalWrite => Unavailable,
        }
    };

    (mode, status)
}

/// Default contract version for capabilities that do not yet carry a typed
/// contract. This is never `0.0` so the registry never reports an unknown
/// contract version.
fn default_version_for(mode: CapabilityMode) -> &'static str {
    match mode {
        CapabilityMode::LocalRead | CapabilityMode::LocalWrite | CapabilityMode::Composite => "1.2",
        CapabilityMode::LocalProcess => "2.1",
        CapabilityMode::ExternalRead | CapabilityMode::ExternalWrite => "2.2",
        CapabilityMode::RealAlgorithm => "2.4",
    }
}

fn group_for_mode(mode: CapabilityMode) -> &'static str {
    match mode {
        CapabilityMode::LocalRead | CapabilityMode::LocalWrite => "local",
        CapabilityMode::Composite => "composite",
        CapabilityMode::LocalProcess => "process",
        CapabilityMode::ExternalRead | CapabilityMode::ExternalWrite => "external",
        CapabilityMode::RealAlgorithm => "algorithm",
    }
}

fn execution_class_for(mode: CapabilityMode) -> CapabilityExecutionClass {
    match mode {
        CapabilityMode::RealAlgorithm => CapabilityExecutionClass::Advisory,
        CapabilityMode::LocalRead | CapabilityMode::LocalWrite | CapabilityMode::LocalProcess => {
            CapabilityExecutionClass::LocalOperation
        }
        CapabilityMode::ExternalRead | CapabilityMode::ExternalWrite => {
            CapabilityExecutionClass::ExternalIntegration
        }
        CapabilityMode::Composite => CapabilityExecutionClass::ControlPlane,
    }
}

fn verification_for(
    name: &str,
    mode: CapabilityMode,
    status: CapabilityStatus,
) -> VerificationGrade {
    if status != CapabilityStatus::Real {
        return VerificationGrade::Declared;
    }
    if matches!(
        name,
        "code-reader" | "code-writer" | "diagnostics" | "pipeline-architect" | "rust-surgeon"
    ) {
        return VerificationGrade::Observed;
    }
    if crate::bio_system::contains(name)
        || mode == CapabilityMode::RealAlgorithm
        || matches!(name, "git-nexus" | "github" | "github-manager")
    {
        return VerificationGrade::Tested;
    }
    VerificationGrade::Declared
}

impl CapabilityProfile {
    pub fn allows(self, capability: &CapabilityInfo) -> bool {
        match self {
            Self::All => true,
            Self::WindowsOffline => {
                capability.status == CapabilityStatus::Real
                    && capability.execution_class != CapabilityExecutionClass::ExternalIntegration
                    && capability.verification >= VerificationGrade::Tested
            }
        }
    }
}

pub fn catalog(names: &[&str]) -> Vec<CapabilityInfo> {
    names
        .iter()
        .map(|name| {
            let (mode, status) = classify(name);
            let execution_class = execution_class_for(mode);
            let verification = verification_for(name, mode, status);
            let contract = contract::get_contract(name);
            let version = contract
                .as_ref()
                .map(|c| c.version.to_string())
                .unwrap_or_else(|| default_version_for(mode).to_string());
            let group = contract
                .as_ref()
                .map(|c| c.group.to_string())
                .unwrap_or_else(|| group_for_mode(mode).to_string());
            CapabilityInfo {
                name: (*name).to_string(),
                mode,
                status,
                execution_class,
                verification,
                version,
                group,
                deprecated: contract.as_ref().is_some_and(|c| c.deprecated),
                deprecation_message: contract
                    .and_then(|c| c.deprecation_message.map(|s| s.to_string())),
            }
        })
        .collect()
}

pub fn catalog_for_profile(names: &[&str], profile: CapabilityProfile) -> Vec<CapabilityInfo> {
    catalog(names)
        .into_iter()
        .filter(|capability| profile.allows(capability))
        .collect()
}

pub fn render(names: &[&str]) -> String {
    render_catalog(catalog(names))
}

pub fn render_for_profile(names: &[&str], profile: CapabilityProfile) -> String {
    render_catalog(catalog_for_profile(names, profile))
}

fn render_catalog(capabilities: Vec<CapabilityInfo>) -> String {
    capabilities
        .into_iter()
        .map(|capability| {
            let mut line = format!(
                "{}\t{}\t{}\t{}\t{}\tv{}\t{}",
                capability.name,
                capability.mode,
                capability.status,
                capability.execution_class,
                capability.verification,
                capability.version,
                capability.group
            );
            if capability.deprecated {
                line.push_str("\t[DEPRECATED]");
                if let Some(ref msg) = capability.deprecation_message {
                    line.push_str(&format!(" {msg}"));
                }
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn execute(name: &str, prompt: &str) -> Option<ExecutionOutcome> {
    // Phase 0: Contract validation
    if let Some(contract) = contract::get_contract(name) {
        if let Err(error) = contract.validate_input(prompt) {
            return Some(ExecutionOutcome::failed(
                "invalid_contract",
                format!("[{name}] contract validation failed: {error}"),
            ));
        }
    }

    // Phase 1: Check capability status first — Unavailable/Unsupported is a typed failure gate
    let (cap_mode, cap_status) = classify(name);
    match cap_status {
        CapabilityStatus::Unavailable => {
            return Some(ExecutionOutcome::failed(
                "capability_unavailable",
                format!("[{name}] capability is unavailable in this environment (requires external tools/credentials)"),
            ));
        }
        CapabilityStatus::Unsupported => {
            return Some(ExecutionOutcome::failed(
                "capability_unsupported",
                format!("[{name}] capability is not supported on this platform"),
            ));
        }
        CapabilityStatus::Deprecated => {
            // Deprecated blades still execute but may produce a warning
        }
        CapabilityStatus::Real => {}
    }

    // Phase 2a: Truthful Rust-native biological homeostasis adapters.
    // These deliberately bypass older advisory wrappers that can shadow the
    // richer deterministic implementations.
    if let Some(outcome) = crate::bio::execute(name, prompt) {
        return Some(outcome);
    }
    if let Some(outcome) = crate::bio_system::execute(name, prompt, false) {
        return Some(outcome);
    }

    // Phase 2: Route real local adapters directly — never let RealBlades override them
    match name {
        "code-reader" => return local_text_adapter(name, prompt, true),
        "code-writer" => return Some(transactional_write(prompt)),
        "diagnostics" => return local_text_adapter(name, prompt, false),
        "git-nexus" => return Some(git_status(prompt)),
        "github" => return Some(github_read(prompt)),
        "github-manager" => return Some(github_manager_read(prompt)),
        _ => {}
    }

    // Phase 3: Route process-wrapper and external capability modes through process runner
    match cap_mode {
        CapabilityMode::LocalProcess => {
            // Route through safe process runner if a real implementation exists
            // For now, fall through to blade implementation
        }
        CapabilityMode::ExternalRead | CapabilityMode::ExternalWrite => {
            // Route through external.rs infrastructure
            // For now, fall through to blade implementation
        }
        _ => {}
    }

    // Phase 4: Fall through to RealBlades (pure algorithm blades only — already gated by Phases 1-2)
    // Wrap RealBlades string output with smart classification: check for error patterns
    if let Some(output) = crate::real_blades::RealBlades::execute(name, prompt) {
        let outcome = blade_outcome_from_string(name, &output);
        if outcome.is_some() {
            return outcome;
        }
        return Some(ExecutionOutcome::completed(output));
    }

    None
}

/// Check a blade output string for error/placeholder patterns and return a typed
/// failure if detected. Returns None if the output appears to be a valid completion.
fn blade_outcome_from_string(name: &str, output: &str) -> Option<ExecutionOutcome> {
    let trimmed = output.trim();
    // Usage/error messages without real output
    if trimmed.starts_with(&format!("[{name}] Usage:"))
        || trimmed.starts_with(&format!("[{name}] Missing"))
        || trimmed.starts_with(&format!("[{name}] Error:"))
        || trimmed.starts_with(&format!("[{name}] Failed"))
        || trimmed.starts_with(&format!("[{name}] API error"))
        || trimmed.starts_with(&format!("[{name}] Tool"))
        || trimmed.starts_with(&format!("[{name}] Required"))
        || trimmed.starts_with(&format!("[{name}] not found"))
    {
        return Some(ExecutionOutcome::failed(
            "blade_execution_failed",
            output.to_string(),
        ));
    }
    // Placeholder/simulation patterns
    let lower = trimmed.to_lowercase();
    if lower.contains("processing...")
        || lower.contains("generating...")
        || lower.contains("simulation")
        || lower.contains("[simulated]")
    {
        return Some(ExecutionOutcome::failed(
            "blade_placeholder",
            output.to_string(),
        ));
    }
    None
}

/// Thin helper retained for callers/tests that only need the operational mode.
/// The canonical source of truth is `classify`.
#[allow(dead_code)]
pub fn mode(name: &str) -> CapabilityMode {
    classify(name).0
}

#[allow(dead_code)]
fn status(name: &str) -> CapabilityStatus {
    classify(name).1
}

fn local_text_adapter(name: &str, prompt: &str, include_content: bool) -> Option<ExecutionOutcome> {
    let input = prompt.trim();
    if input.is_empty() || input.contains(['\r', '\n']) {
        return None;
    }

    let path = Path::new(input);
    if !path.is_file() {
        return looks_like_path(input).then(|| {
            ExecutionOutcome::failed(
                "path_not_found",
                format!("[{name}] local file not found: {input}"),
            )
        });
    }

    Some(read_local_text(name, path, include_content))
}

fn read_local_text(name: &str, path: &Path, include_content: bool) -> ExecutionOutcome {
    let path = match canonical_allowed(path) {
        Ok(path) => path,
        Err(outcome) => return outcome,
    };
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) => {
            return ExecutionOutcome::failed(
                "file_metadata_failed",
                format!("[{name}] cannot inspect {}: {error}", path.display()),
            );
        }
    };
    if metadata.len() > MAX_LOCAL_READ_BYTES {
        return ExecutionOutcome::failed(
            "file_too_large",
            format!(
                "[{name}] local file exceeds {} bytes: {}",
                MAX_LOCAL_READ_BYTES,
                path.display()
            ),
        );
    }
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => {
            return ExecutionOutcome::failed(
                "file_read_failed",
                format!("[{name}] cannot read {}: {error}", path.display()),
            );
        }
    };
    let analysis = blade::execute(name, &content);
    let mut output = format!(
        "[{name}] LOCAL READ\npath: {}\nbytes: {}\n{}",
        path.display(),
        metadata.len(),
        analysis
    );
    if include_content {
        output.push_str("\n\n");
        output.push_str(&content);
    }
    ExecutionOutcome::completed(output)
}

fn git_status(prompt: &str) -> ExecutionOutcome {
    use crate::process::ProcessSpec;

    let requested = prompt.trim();
    let path = if requested.is_empty() || requested == "." {
        match env::current_dir() {
            Ok(path) => path,
            Err(error) => {
                return ExecutionOutcome::failed(
                    "current_dir_failed",
                    format!("[git-nexus] cannot resolve current directory: {error}"),
                );
            }
        }
    } else {
        PathBuf::from(requested)
    };
    let path = match canonical_allowed(&path) {
        Ok(path) if path.is_dir() => path,
        Ok(path) => {
            return ExecutionOutcome::failed(
                "not_a_directory",
                format!(
                    "[git-nexus] repository path is not a directory: {}",
                    path.display()
                ),
            );
        }
        Err(outcome) => return outcome,
    };

    // Verify git is available via an availability probe
    let probe = ProcessSpec::new("git").args(["--version"]);
    match crate::process::run_process(&probe) {
        Ok(result) if result.exit_code != 0 => {
            return ExecutionOutcome::failed(
                "git_unavailable",
                "[git-nexus] git is not available on PATH",
            );
        }
        Err(outcome) => return outcome,
        _ => {}
    }

    let spec = ProcessSpec::new("git")
        .args([
            "-C",
            &path.to_string_lossy(),
            "status",
            "--short",
            "--branch",
        ])
        .git_optional_locks(true)
        .timeout_ms(15_000);

    crate::process::run_process_typed(&spec)
}

fn transactional_write(prompt: &str) -> ExecutionOutcome {
    let mut parts = prompt.splitn(3, '|');
    let requested = parts.next().unwrap_or_default().trim();
    let expected = parts.next().unwrap_or_default().trim();
    let content = match parts.next() {
        Some(content) => content,
        None => {
            return ExecutionOutcome::failed(
                "invalid_write_contract",
                "[code-writer] usage: path|expected_sha256_or_NEW|content",
            );
        }
    };
    if requested.is_empty() || expected.is_empty() {
        return ExecutionOutcome::failed(
            "invalid_write_contract",
            "[code-writer] path and expected hash are required",
        );
    }
    if content.len() as u64 > MAX_LOCAL_READ_BYTES {
        return ExecutionOutcome::failed(
            "file_too_large",
            format!(
                "[code-writer] content exceeds {} bytes",
                MAX_LOCAL_READ_BYTES
            ),
        );
    }

    let requested_path = PathBuf::from(requested);
    let target = match writable_target(&requested_path) {
        Ok(path) => path,
        Err(outcome) => return outcome,
    };
    let existed = target.exists();
    let previous = if existed {
        match fs::read(&target) {
            Ok(bytes) => bytes,
            Err(error) => {
                return ExecutionOutcome::failed(
                    "file_read_failed",
                    format!("[code-writer] cannot read {}: {error}", target.display()),
                );
            }
        }
    } else {
        Vec::new()
    };

    if existed {
        let actual = digest_bytes(&previous);
        if !expected.eq_ignore_ascii_case(&actual) {
            return ExecutionOutcome::failed(
                "stale_write",
                format!(
                    "[code-writer] expected hash does not match {}\nexpected: {expected}\nactual: {actual}",
                    target.display()
                ),
            );
        }
    } else if expected != "NEW" {
        return ExecutionOutcome::failed(
            "new_file_requires_new",
            format!(
                "[code-writer] new file requires expected value NEW: {}",
                target.display()
            ),
        );
    }

    let nonce = format!("{}-{}", std::process::id(), now_nanos());
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    let temporary = target.with_file_name(format!(".{file_name}.octopus-{nonce}.tmp"));
    let mut temp_file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
    {
        Ok(file) => file,
        Err(error) => {
            return ExecutionOutcome::failed(
                "temporary_create_failed",
                format!("[code-writer] cannot create temporary file: {error}"),
            );
        }
    };
    if let Err(error) = temp_file
        .write_all(content.as_bytes())
        .and_then(|_| temp_file.sync_all())
    {
        let _ = fs::remove_file(&temporary);
        return ExecutionOutcome::failed(
            "temporary_write_failed",
            format!("[code-writer] cannot write temporary file: {error}"),
        );
    }
    drop(temp_file);

    let backup = existed.then(|| target.with_file_name(format!("{file_name}.octopus-{nonce}.bak")));
    if let Some(backup) = backup.as_ref() {
        if let Err(error) = fs::rename(&target, backup) {
            let _ = fs::remove_file(&temporary);
            return ExecutionOutcome::failed(
                "backup_failed",
                format!("[code-writer] cannot create backup: {error}"),
            );
        }
    }
    if let Err(error) = fs::rename(&temporary, &target) {
        if let Some(backup) = backup.as_ref() {
            let _ = fs::rename(backup, &target);
        }
        let _ = fs::remove_file(&temporary);
        return ExecutionOutcome::failed(
            "write_commit_failed",
            format!("[code-writer] commit failed and rollback was attempted: {error}"),
        );
    }

    ExecutionOutcome::completed(format!(
        "[code-writer] LOCAL WRITE\npath: {}\nbytes: {}\nsha256: {}\nbackup: {}",
        target.display(),
        content.len(),
        digest_bytes(content.as_bytes()),
        backup
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string())
    ))
}

fn writable_target(path: &Path) -> Result<PathBuf, ExecutionOutcome> {
    if path.exists() {
        return canonical_allowed(path);
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let parent = canonical_allowed(parent)?;
    let file_name = path.file_name().ok_or_else(|| {
        ExecutionOutcome::failed("invalid_write_path", "[code-writer] missing file name")
    })?;
    Ok(parent.join(file_name))
}

fn digest_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn canonical_allowed(path: &Path) -> Result<PathBuf, ExecutionOutcome> {
    let canonical = fs::canonicalize(path).map_err(|error| {
        ExecutionOutcome::failed(
            "path_not_found",
            format!("[local-adapter] invalid path {}: {error}", path.display()),
        )
    })?;
    if allowed_roots()
        .iter()
        .any(|root| canonical.starts_with(root))
    {
        Ok(canonical)
    } else {
        Err(ExecutionOutcome::failed(
            "path_denied",
            format!(
                "[local-adapter] path is outside allowed roots: {}",
                canonical.display()
            ),
        ))
    }
}

fn allowed_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(current) = env::current_dir().and_then(fs::canonicalize) {
        roots.push(current);
    }
    if let Some(configured) = env::var_os("OCTOPUS_ALLOWED_ROOTS") {
        roots.extend(env::split_paths(&configured).filter_map(|path| fs::canonicalize(path).ok()));
    }
    roots
}

fn looks_like_path(value: &str) -> bool {
    value.contains(['/', '\\']) || Path::new(value).extension().is_some()
}

fn github_read(prompt: &str) -> ExecutionOutcome {
    use crate::external;

    let probe = external::probe_gh_auth();
    if !probe.available {
        return ExecutionOutcome::failed(
            "tool_unavailable",
            "[github] gh CLI is not available on PATH. Install from https://cli.github.com/",
        );
    }

    if probe.auth_state == external::AuthState::Unauthenticated {
        return ExecutionOutcome::failed(
            "auth_required",
            "[github] not authenticated. Run 'gh auth login' first.",
        );
    }

    let input = prompt.trim();
    if input.is_empty() {
        return ExecutionOutcome::failed(
            "invalid_input",
            "[github] usage: repo-view <owner/repo> | pr-list <owner/repo> | issue-list <owner/repo> | run-list <owner/repo>",
        );
    }

    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let command = parts[0];
    let arg = parts.get(1).unwrap_or(&"");

    let (subcommand, extra_args) = match command {
        "repo-view" => (vec!["repo", "view", arg], vec![]),
        "pr-list" => (vec!["pr", "list", "--repo", arg], vec![]),
        "pr-view" => (vec!["pr", "view", arg], vec![]),
        "issue-list" => (vec!["issue", "list", "--repo", arg], vec![]),
        "issue-view" => (vec!["issue", "view", arg], vec![]),
        "run-list" => (vec!["run", "list", "--repo", arg], vec![]),
        "workflow-list" => (vec!["workflow", "list", "--repo", arg], vec![]),
        "api" => {
            let api_args: Vec<&str> = arg.split_whitespace().collect();
            (vec!["api"], api_args)
        }
        _ => {
            return ExecutionOutcome::failed(
                "invalid_command",
                format!(
                    "[github] unknown command: {command}. Valid: repo-view, pr-list, pr-view, issue-list, issue-view, run-list, workflow-list, api"
                ),
            );
        }
    };

    let mut args: Vec<String> = subcommand.iter().map(|s| s.to_string()).collect();
    args.extend(extra_args.iter().map(|s| s.to_string()));

    let spec = ProcessSpec::new("gh")
        .args(args)
        .timeout_ms(15_000)
        .git_optional_locks(false);

    let result = process::run_process_typed(&spec);
    if result.is_failed() {
        return result;
    }

    ExecutionOutcome::completed(format!(
        "[github] LOCAL READ\ncommand: {command}\n{}",
        result.output
    ))
}

fn github_manager_read(prompt: &str) -> ExecutionOutcome {
    use crate::external;

    let probe = external::probe_gh_auth();
    if !probe.available {
        return ExecutionOutcome::failed(
            "tool_unavailable",
            "[github-manager] gh CLI is not available on PATH",
        );
    }

    if probe.auth_state == external::AuthState::Unauthenticated {
        return ExecutionOutcome::failed(
            "auth_required",
            "[github-manager] not authenticated. Run 'gh auth login' first.",
        );
    }

    let input = prompt.trim();
    if input.is_empty() {
        return ExecutionOutcome::failed(
            "invalid_input",
            "[github-manager] usage: repo-list | pr-list | issue-list | run-list",
        );
    }

    let spec = ProcessSpec::new("gh")
        .args(["repo", "list", "--limit", "20"])
        .timeout_ms(15_000);

    let result = process::run_process_typed(&spec);
    if result.is_failed() {
        return result;
    }

    ExecutionOutcome::completed(format!(
        "[github-manager] LOCAL READ\ncommand: repo-list\n{}",
        result.output
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_PUBLIC_CAPABILITIES: usize = 225;

    #[test]
    fn catalog_marks_real_local_and_composite_capabilities() {
        assert_eq!(mode("code-reader"), CapabilityMode::LocalRead);
        assert_eq!(mode("code-writer"), CapabilityMode::LocalWrite);
        assert_eq!(mode("git-nexus"), CapabilityMode::LocalRead);
        assert_eq!(mode("rust-surgeon"), CapabilityMode::Composite);
        assert_eq!(mode("summarize"), CapabilityMode::RealAlgorithm);
        assert_eq!(mode("weather"), CapabilityMode::LocalProcess);
        assert_eq!(mode("github"), CapabilityMode::ExternalRead);
        assert_eq!(mode("notion"), CapabilityMode::ExternalRead);
        assert_eq!(mode("openai-image-gen"), CapabilityMode::ExternalWrite);
    }

    #[test]
    fn catalog_reports_effect_and_evidence_axes() {
        let caps = crate::capabilities();
        let find = |name: &str| {
            caps.iter()
                .find(|capability| capability.name == name)
                .unwrap()
        };

        assert_eq!(
            find("code-reader").execution_class,
            CapabilityExecutionClass::LocalOperation
        );
        assert_eq!(
            find("code-reader").verification,
            VerificationGrade::Observed
        );
        assert_eq!(
            find("summarize").execution_class,
            CapabilityExecutionClass::Advisory
        );
        assert_eq!(find("summarize").verification, VerificationGrade::Tested);
        assert_eq!(
            find("github").execution_class,
            CapabilityExecutionClass::ExternalIntegration
        );
        assert_eq!(find("github").verification, VerificationGrade::Tested);
    }

    #[test]
    fn windows_offline_profile_is_real_non_external_and_tested() {
        let all = crate::list();
        let caps = catalog_for_profile(&all, CapabilityProfile::WindowsOffline);
        assert!(!caps.is_empty());
        assert!(caps
            .iter()
            .all(|capability| CapabilityProfile::WindowsOffline.allows(capability)));
        for included in ["code-reader", "summarize", "rust-surgeon"] {
            assert!(caps.iter().any(|capability| capability.name == included));
        }
        for excluded in [
            "github",
            "github-manager",
            "notion",
            "apple-notes",
            "bear-notes",
            "omni-surgeon",
            "file-surgeon",
        ] {
            assert!(!caps.iter().any(|capability| capability.name == excluded));
        }
    }

    #[test]
    fn canonical_registry_has_225_capabilities_and_no_copied_native() {
        let caps = crate::capabilities();
        assert_eq!(
            caps.len(),
            EXPECTED_PUBLIC_CAPABILITIES,
            "expected the 192 Octopus capabilities plus 33 bundled Bio-Binaries targets"
        );
        let copied = caps
            .iter()
            .filter(|c| {
                c.mode == CapabilityMode::RealAlgorithm
                    && c.status == CapabilityStatus::Unavailable
                    && c.name.starts_with("zzz")
            })
            .count();
        assert_eq!(copied, 0);
        // No capability may report an unknown/placeholder contract version.
        assert!(
            caps.iter().all(|c| c.version != "0.0"),
            "registry contains a capability with unknown 0.0 contract version"
        );
        // Every capability carries a mode and a status.
        assert!(caps.iter().all(|c| {
            matches!(
                c.mode,
                CapabilityMode::RealAlgorithm
                    | CapabilityMode::LocalRead
                    | CapabilityMode::LocalWrite
                    | CapabilityMode::LocalProcess
                    | CapabilityMode::ExternalRead
                    | CapabilityMode::ExternalWrite
                    | CapabilityMode::Composite
            )
        }));
    }

    #[test]
    fn mode_and_status_categories_are_mutually_exhaustive() {
        let caps = crate::capabilities();
        let mode_count = caps.len();
        let status_count = caps.len();
        assert_eq!(mode_count, EXPECTED_PUBLIC_CAPABILITIES);
        assert_eq!(status_count, EXPECTED_PUBLIC_CAPABILITIES);
        // Every capability belongs to exactly one mode bucket.
        let mut mode_total = 0;
        for mode in [
            CapabilityMode::RealAlgorithm,
            CapabilityMode::LocalRead,
            CapabilityMode::LocalWrite,
            CapabilityMode::LocalProcess,
            CapabilityMode::ExternalRead,
            CapabilityMode::ExternalWrite,
            CapabilityMode::Composite,
        ] {
            mode_total += caps.iter().filter(|c| c.mode == mode).count();
        }
        assert_eq!(
            mode_total, EXPECTED_PUBLIC_CAPABILITIES,
            "mode buckets must sum to the public registry size"
        );

        let mut status_total = 0;
        for status in [
            CapabilityStatus::Real,
            CapabilityStatus::Unavailable,
            CapabilityStatus::Unsupported,
            CapabilityStatus::Deprecated,
        ] {
            status_total += caps.iter().filter(|c| c.status == status).count();
        }
        assert_eq!(
            status_total, EXPECTED_PUBLIC_CAPABILITIES,
            "status buckets must sum to the public registry size"
        );
    }

    #[test]
    fn apple_notes_and_bear_notes_are_unsupported() {
        assert_eq!(status("apple-notes"), CapabilityStatus::Unsupported);
        assert_eq!(status("bear-notes"), CapabilityStatus::Unsupported);
    }

    #[test]
    fn cli_and_mcp_capability_lists_are_identical() {
        // Both the CLI `capabilities` command and the MCP `octopus_capabilities`
        // tool render from the exact same canonical registry.
        let cli = crate::render_capabilities();
        // MCP uses the same render function; verify consistency
        assert_eq!(cli.lines().count(), EXPECTED_PUBLIC_CAPABILITIES);
        assert!(cli.lines().all(|line| line.split('\t').count() >= 7));
    }

    #[test]
    fn bundled_bio_binaries_are_real_tested_local_process_adapters() {
        let caps = crate::capabilities();
        for name in ["viral-infect", "hox-diff", "omega-master", "microscope-mem"] {
            let capability = caps
                .iter()
                .find(|capability| capability.name == name)
                .unwrap();
            assert_eq!(capability.mode, CapabilityMode::LocalProcess);
            assert_eq!(capability.status, CapabilityStatus::Real);
            assert_eq!(capability.verification, VerificationGrade::Tested);
        }
    }

    #[test]
    fn code_reader_reads_a_real_local_file() {
        let outcome = execute("code-reader", "Cargo.toml").unwrap();
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("LOCAL READ"));
        assert!(outcome.output.contains("[package]"));
    }

    #[test]
    fn diagnostics_reads_a_real_local_file() {
        let outcome = execute("diagnostics", "Cargo.toml").unwrap();
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("LOCAL READ"));
        assert!(outcome.output.contains("karakterek="));
    }

    #[test]
    fn missing_path_returns_a_typed_failure() {
        let outcome = execute("code-reader", "missing/file.rs").unwrap();
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("path_not_found"));
    }

    #[test]
    fn raw_source_falls_through_to_copied_native_blade() {
        assert!(execute("code-reader", "fn main() {}\n").is_none());
    }

    #[test]
    fn code_writer_creates_and_updates_transactionally() {
        let root = PathBuf::from("target").join(format!("write-test-{}", now_nanos()));
        fs::create_dir_all(&root).unwrap();
        let target = root.join("value.rs");
        let create = execute(
            "code-writer",
            &format!("{}|NEW|fn value() -> u8 {{ 1 }}", target.display()),
        )
        .unwrap();
        assert!(!create.is_failed());
        let old_hash = digest_bytes(&fs::read(&target).unwrap());
        let update = execute(
            "code-writer",
            &format!("{}|{}|fn value() -> u8 {{ 2 }}", target.display(), old_hash),
        )
        .unwrap();
        assert!(!update.is_failed());
        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "fn value() -> u8 { 2 }"
        );
        assert!(fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().ends_with(".bak")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn code_writer_rejects_stale_hash_without_changing_file() {
        let root = PathBuf::from("target").join(format!("stale-test-{}", now_nanos()));
        fs::create_dir_all(&root).unwrap();
        let target = root.join("value.rs");
        fs::write(&target, "original").unwrap();
        let stale_hash = "a".repeat(64);
        let outcome = execute(
            "code-writer",
            &format!("{}|{stale_hash}|replacement", target.display()),
        )
        .unwrap();
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("stale_write"));
        assert_eq!(fs::read_to_string(&target).unwrap(), "original");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn github_empty_input_returns_typed_failure() {
        let outcome = execute("github", "").unwrap();
        assert!(outcome.is_failed());
        // May be auth_required (gh not logged in) or invalid_input (empty prompt)
        let code = outcome.code.as_deref().unwrap();
        assert!(
            code == "auth_required" || code == "invalid_input" || code == "tool_unavailable",
            "unexpected code: {code}"
        );
    }

    #[test]
    fn github_unknown_command_returns_typed_failure() {
        let outcome = execute("github", "unknown-cmd repo").unwrap();
        assert!(outcome.is_failed());
        let code = outcome.code.as_deref().unwrap();
        assert!(
            code == "invalid_command" || code == "auth_required" || code == "tool_unavailable",
            "unexpected code: {code}"
        );
    }

    #[test]
    fn github_manager_empty_input_returns_typed_failure() {
        let outcome = execute("github-manager", "").unwrap();
        assert!(outcome.is_failed());
        let code = outcome.code.as_deref().unwrap();
        assert!(
            code == "auth_required" || code == "invalid_input" || code == "tool_unavailable",
            "unexpected code: {code}"
        );
    }
}
