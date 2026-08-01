use crate::capability::{CapabilityMode, CapabilityStatus};
use crate::contract::{self, InputType};
use crate::{capabilities, list, ExecutionOutcome};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const SCHEMA: &str = "octopus.arm-manifest/v1";
const MAX_MANIFEST_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestEffect {
    Read,
    Write,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EvidenceRequirement {
    OutputContains { value: String },
    MinOutputBytes { value: usize },
    FileExists { path: String },
    FileSha256 { path: String, sha256: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestArm {
    pub id: String,
    pub spec: String,
    pub mission: String,
    pub input: String,
    pub effect: ManifestEffect,
    pub completion: String,
    pub stop_condition: String,
    #[serde(default)]
    pub allowed_paths: Vec<String>,
    pub evidence: Vec<EvidenceRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmManifest {
    pub schema: String,
    pub objective: String,
    pub arms: Vec<ManifestArm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestError {
    pub code: &'static str,
    pub message: String,
}

impl ManifestError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn parse_and_validate(source: &str, allow_write: bool) -> Result<ArmManifest, ManifestError> {
    if source.len() > MAX_MANIFEST_BYTES {
        return Err(ManifestError::new(
            "manifest_too_large",
            format!("manifest exceeds {MAX_MANIFEST_BYTES} bytes"),
        ));
    }
    let manifest: ArmManifest = serde_json::from_str(source).map_err(|error| {
        ManifestError::new(
            "manifest_invalid_json",
            format!("invalid manifest JSON: {error}"),
        )
    })?;
    validate(&manifest, allow_write)?;
    Ok(manifest)
}

fn validate(manifest: &ArmManifest, allow_write: bool) -> Result<(), ManifestError> {
    if manifest.schema != SCHEMA {
        return Err(ManifestError::new(
            "manifest_schema_unsupported",
            format!("expected schema '{SCHEMA}', got '{}'", manifest.schema),
        ));
    }
    if manifest.objective.trim().is_empty() {
        return Err(ManifestError::new(
            "manifest_objective_missing",
            "manifest objective must not be empty",
        ));
    }
    if manifest.arms.len() < 2 {
        return Err(ManifestError::new(
            "manifest_topology_invalid",
            "an Octopus manifest requires at least two independent arms",
        ));
    }

    let registry: HashSet<_> = list().into_iter().collect();
    let catalog: HashMap<_, _> = capabilities()
        .into_iter()
        .map(|capability| (capability.name.clone(), capability))
        .collect();
    let mut ids = HashSet::new();

    for arm in &manifest.arms {
        validate_text_fields(arm)?;
        if !ids.insert(arm.id.as_str()) {
            return Err(ManifestError::new(
                "manifest_duplicate_arm",
                format!("duplicate arm id '{}'", arm.id),
            ));
        }
        if arm.spec.contains("||") {
            return Err(ManifestError::new(
                "manifest_nested_topology",
                format!("arm '{}' spec must not contain parallel '||'", arm.id),
            ));
        }

        let components: Vec<_> = arm
            .spec
            .split('+')
            .map(str::trim)
            .filter(|component| !component.is_empty())
            .collect();
        if components.is_empty() {
            return Err(ManifestError::new(
                "manifest_empty_arm",
                format!("arm '{}' has no blade components", arm.id),
            ));
        }

        let mut inferred_write = false;
        for component in &components {
            for candidate in component.split('|').map(str::trim) {
                if !registry.contains(candidate) {
                    return Err(ManifestError::new(
                        "manifest_unknown_blade",
                        format!("arm '{}' references unknown blade '{candidate}'", arm.id),
                    ));
                }
                let capability = &catalog[candidate];
                if capability.status != CapabilityStatus::Real {
                    return Err(ManifestError::new(
                        "manifest_blade_unavailable",
                        format!(
                            "arm '{}' blade '{}' is {}",
                            arm.id, candidate, capability.status
                        ),
                    ));
                }
                inferred_write |=
                    matches!(
                        capability.mode,
                        CapabilityMode::LocalWrite | CapabilityMode::ExternalWrite
                    ) || matches!(candidate, "rust-surgeon" | "omni-surgeon" | "file-surgeon");
            }
        }

        if inferred_write && arm.effect != ManifestEffect::Write {
            return Err(ManifestError::new(
                "manifest_effect_mismatch",
                format!(
                    "arm '{}' contains a write-capable blade but declares read",
                    arm.id
                ),
            ));
        }
        if arm.effect == ManifestEffect::Write && !allow_write {
            return Err(ManifestError::new(
                "manifest_write_confirmation_required",
                format!("arm '{}' requires explicit --allow-write", arm.id),
            ));
        }
        if arm.effect == ManifestEffect::Write && arm.allowed_paths.is_empty() {
            return Err(ManifestError::new(
                "manifest_write_boundary_missing",
                format!("write arm '{}' requires at least one allowed path", arm.id),
            ));
        }

        validate_component_contracts(arm, &components)?;
        validate_path_boundaries(arm, &components)?;
        validate_evidence(arm)?;
    }
    Ok(())
}

fn validate_text_fields(arm: &ManifestArm) -> Result<(), ManifestError> {
    for (field, value) in [
        ("id", arm.id.as_str()),
        ("spec", arm.spec.as_str()),
        ("mission", arm.mission.as_str()),
        ("input", arm.input.as_str()),
        ("completion", arm.completion.as_str()),
        ("stop_condition", arm.stop_condition.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(ManifestError::new(
                "manifest_required_field_missing",
                format!("arm '{}' field '{field}' must not be empty", arm.id),
            ));
        }
    }
    Ok(())
}

fn validate_component_contracts(
    arm: &ManifestArm,
    components: &[&str],
) -> Result<(), ManifestError> {
    for candidate in components[0].split('|').map(str::trim) {
        if let Some(capability_contract) = contract::get_contract(candidate) {
            capability_contract
                .validate_input(&arm.input)
                .map_err(|error| {
                    ManifestError::new(
                        "manifest_input_contract_mismatch",
                        format!("arm '{}' blade '{}': {error}", arm.id, candidate),
                    )
                })?;
        }
    }

    for component in components.iter().skip(1) {
        for candidate in component.split('|').map(str::trim) {
            if matches!(candidate, "rust-surgeon") {
                continue;
            }
            if let Some(capability_contract) = contract::get_contract(candidate) {
                let requires_bound_input = capability_contract.input.iter().any(|input| {
                    input.required && !matches!(input.input_type, InputType::Text | InputType::Any)
                });
                if requires_bound_input {
                    return Err(ManifestError::new(
                        "manifest_component_input_unbound",
                        format!(
                            "arm '{}' later component '{}' requires explicit typed input; split it into its own arm",
                            arm.id, candidate
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_path_boundaries(arm: &ManifestArm, components: &[&str]) -> Result<(), ManifestError> {
    let roots = arm
        .allowed_paths
        .iter()
        .map(|path| normalize_path(path))
        .collect::<Result<Vec<_>, _>>()?;
    let first_blade = components[0].split('|').next().unwrap_or_default().trim();
    let mut declared_paths = Vec::new();

    if let Some(capability_contract) = contract::get_contract(first_blade) {
        if capability_contract
            .input
            .iter()
            .any(|input| input.required && input.input_type == InputType::FilePath)
        {
            declared_paths.push(arm.input.split('|').next().unwrap_or_default().trim());
        }
    }
    for requirement in &arm.evidence {
        match requirement {
            EvidenceRequirement::FileExists { path }
            | EvidenceRequirement::FileSha256 { path, .. } => declared_paths.push(path),
            _ => {}
        }
    }

    if !declared_paths.is_empty() && roots.is_empty() {
        return Err(ManifestError::new(
            "manifest_path_boundary_missing",
            format!(
                "arm '{}' uses filesystem paths but allowed_paths is empty",
                arm.id
            ),
        ));
    }
    for path in declared_paths {
        let normalized = normalize_path(path)?;
        if !roots.iter().any(|root| path_is_within(&normalized, root)) {
            return Err(ManifestError::new(
                "manifest_path_denied",
                format!("arm '{}' path '{}' is outside allowed_paths", arm.id, path),
            ));
        }
    }
    Ok(())
}

fn validate_evidence(arm: &ManifestArm) -> Result<(), ManifestError> {
    if arm.evidence.is_empty() {
        return Err(ManifestError::new(
            "manifest_evidence_missing",
            format!("arm '{}' requires at least one evidence rule", arm.id),
        ));
    }
    for requirement in &arm.evidence {
        match requirement {
            EvidenceRequirement::OutputContains { value } if value.is_empty() => {
                return Err(ManifestError::new(
                    "manifest_evidence_invalid",
                    format!("arm '{}' has an empty output_contains value", arm.id),
                ));
            }
            EvidenceRequirement::MinOutputBytes { value } if *value == 0 => {
                return Err(ManifestError::new(
                    "manifest_evidence_invalid",
                    format!(
                        "arm '{}' min_output_bytes must be greater than zero",
                        arm.id
                    ),
                ));
            }
            EvidenceRequirement::FileSha256 { sha256, .. }
                if sha256.len() != 64
                    || !sha256
                        .chars()
                        .all(|character| character.is_ascii_hexdigit()) =>
            {
                return Err(ManifestError::new(
                    "manifest_evidence_invalid",
                    format!(
                        "arm '{}' file_sha256 must be 64 hexadecimal characters",
                        arm.id
                    ),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn enforce_evidence(arm: &ManifestArm, outcome: ExecutionOutcome) -> ExecutionOutcome {
    if outcome.is_failed() {
        return outcome;
    }

    let mut receipts = Vec::new();
    let mut failures = Vec::new();
    for (index, requirement) in arm.evidence.iter().enumerate() {
        let result = verify_requirement(requirement, &outcome.output);
        match result {
            Ok(detail) => receipts.push(format!("evidence[{}]=passed {detail}", index + 1)),
            Err(error) => failures.push(format!("evidence[{}]={error}", index + 1)),
        }
    }

    if !failures.is_empty() {
        return ExecutionOutcome::failed(
            "evidence_not_satisfied",
            format!(
                "EVIDENCE GATE FAILED\narm: {}\nmission: {}\ncompletion: {}\n{}\n\nORIGINAL OUTPUT\n{}",
                arm.id,
                arm.mission,
                arm.completion,
                failures.join("\n"),
                outcome.output
            ),
        );
    }

    ExecutionOutcome::completed(format!(
        "{}\n\nEVIDENCE RECEIPT\narm: {}\nmission: {}\ncompletion: {}\nstop: {}\n{}",
        outcome.output,
        arm.id,
        arm.mission,
        arm.completion,
        arm.stop_condition,
        receipts.join("\n")
    ))
}

fn verify_requirement(requirement: &EvidenceRequirement, output: &str) -> Result<String, String> {
    match requirement {
        EvidenceRequirement::OutputContains { value } => output
            .contains(value)
            .then(|| format!("output_contains={value}"))
            .ok_or_else(|| format!("missing output text '{value}'")),
        EvidenceRequirement::MinOutputBytes { value } => (output.len() >= *value)
            .then(|| format!("output_bytes={} minimum={value}", output.len()))
            .ok_or_else(|| {
                format!(
                    "output has {} bytes, expected at least {value}",
                    output.len()
                )
            }),
        EvidenceRequirement::FileExists { path } => Path::new(path)
            .is_file()
            .then(|| format!("file_exists={path}"))
            .ok_or_else(|| format!("file does not exist: {path}")),
        EvidenceRequirement::FileSha256 { path, sha256 } => {
            let data = fs::read(path).map_err(|error| format!("cannot read {path}: {error}"))?;
            let actual = hex::encode(Sha256::digest(data));
            actual
                .eq_ignore_ascii_case(sha256)
                .then(|| format!("file_sha256={path}:{actual}"))
                .ok_or_else(|| {
                    format!("file hash mismatch: {path} expected={sha256} actual={actual}")
                })
        }
    }
}

fn normalize_path(value: &str) -> Result<PathBuf, ManifestError> {
    if value.trim().is_empty() {
        return Err(ManifestError::new(
            "manifest_path_invalid",
            "filesystem path must not be empty",
        ));
    }
    let source = PathBuf::from(value);
    let absolute = if source.is_absolute() {
        source
    } else {
        std::env::current_dir()
            .map_err(|error| ManifestError::new("manifest_path_invalid", error.to_string()))?
            .join(source)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    Ok(normalized)
}

fn path_is_within(path: &Path, root: &Path) -> bool {
    let path = path.to_string_lossy().to_lowercase();
    let root = root
        .to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_lowercase();
    path == root
        || path
            .strip_prefix(&root)
            .is_some_and(|suffix| suffix.starts_with(['\\', '/']))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(extra_arm: &str) -> String {
        format!(
            r#"{{
  "schema": "octopus.arm-manifest/v1",
  "objective": "verify two independent arms",
  "arms": [
    {{
      "id": "analysis",
      "spec": "code-analysis",
      "mission": "analyze Rust source",
      "input": "fn alpha() {{}}",
      "effect": "read",
      "completion": "function count is reported",
      "stop_condition": "after one analysis",
      "evidence": [{{"kind":"output_contains","value":"fn=1"}}]
    }},
    {extra_arm}
  ]
}}"#
        )
    }

    fn summarize_arm() -> &'static str {
        r#"{
      "id": "summary",
      "spec": "summarize",
      "mission": "summarize a sentence",
      "input": "Octopus evidence gate",
      "effect": "read",
      "completion": "summary names Octopus",
      "stop_condition": "after one summary",
      "evidence": [{"kind":"output_contains","value":"Octopus"}]
    }"#
    }

    #[test]
    fn parses_two_distinct_arm_inputs() {
        let manifest = parse_and_validate(&source(summarize_arm()), false).unwrap();
        assert_eq!(manifest.arms.len(), 2);
        assert_ne!(manifest.arms[0].input, manifest.arms[1].input);
    }

    #[test]
    fn rejects_duplicate_ids() {
        let duplicate = summarize_arm().replace("summary", "analysis");
        let error = parse_and_validate(&source(&duplicate), false).unwrap_err();
        assert_eq!(error.code, "manifest_duplicate_arm");
    }

    #[test]
    fn rejects_missing_evidence() {
        let arm = summarize_arm().replace(
            r#""evidence": [{"kind":"output_contains","value":"Octopus"}]"#,
            r#""evidence": []"#,
        );
        let error = parse_and_validate(&source(&arm), false).unwrap_err();
        assert_eq!(error.code, "manifest_evidence_missing");
    }

    #[test]
    fn evidence_failure_changes_completed_outcome_to_failure() {
        let manifest = parse_and_validate(&source(summarize_arm()), false).unwrap();
        let outcome = enforce_evidence(
            &manifest.arms[0],
            ExecutionOutcome::completed("analysis without expected marker"),
        );
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("evidence_not_satisfied"));
    }

    #[test]
    fn path_comparison_is_case_insensitive_and_boundary_aware() {
        assert!(path_is_within(
            Path::new(r"D:\CODEX\project\src\lib.rs"),
            Path::new(r"d:\codex\project")
        ));
        assert!(!path_is_within(
            Path::new(r"D:\codex\project-escape\file"),
            Path::new(r"D:\codex\project")
        ));
    }
}
