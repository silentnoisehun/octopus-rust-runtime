use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn state_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "octopus-manifest-{label}-{}-{stamp}",
        std::process::id()
    ))
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_octopus-runtime"))
}

fn run_manifest(source: &str, state_dir: &Path, allow_write: bool) -> Output {
    let mut command = Command::new(binary());
    command
        .arg("--plain")
        .arg("manifest")
        .arg("-")
        .env("OCTOPUS_STATE_DIR", state_dir)
        .env("OCTOPUS_ENFORCE", "0")
        .env("OCTOPUS_DEV_MODE", "1")
        .env("OCTOPUS_ALLOWED_ROOTS", state_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if allow_write {
        command.arg("--allow-write");
    }

    let mut child = command.spawn().expect("spawn manifest command");
    child
        .stdin
        .take()
        .expect("manifest stdin")
        .write_all(source.as_bytes())
        .expect("write manifest stdin");
    child.wait_with_output().expect("wait for manifest")
}

fn snapshots_under(state_dir: &Path, kind: &str) -> Vec<String> {
    let dir = state_dir.join(kind);
    let mut snapshots = Vec::new();
    if !dir.exists() {
        return snapshots;
    }
    for entry in fs::read_dir(dir).expect("read snapshot directory") {
        let path = entry.expect("snapshot entry").path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("snap") {
            let snapshot = fs::read_to_string(path).expect("read snapshot");
            if kind != "arms" || snapshot.starts_with("OCTOPUS ARM\n") {
                snapshots.push(snapshot);
            }
        }
    }
    snapshots
}

#[test]
fn manifest_routes_distinct_inputs_and_records_evidence_receipts() {
    let state = state_dir("routing");
    fs::create_dir_all(&state).expect("create state directory");
    let source_path = state.join("sample.rs");
    fs::write(&source_path, "fn alpha() {}\n").expect("write source fixture");

    let manifest = json!({
        "schema": "octopus.arm-manifest/v1",
        "objective": "Prove that each arm receives its own declared input",
        "arms": [
            {
                "id": "reader",
                "spec": "code-reader",
                "mission": "Read the declared source file",
                "input": source_path,
                "effect": "read",
                "completion": "The local read receipt is present",
                "stop_condition": "Stop after reading the file",
                "allowed_paths": [state],
                "evidence": [
                    { "kind": "output_contains", "value": "LOCAL READ" },
                    { "kind": "output_contains", "value": "sample.rs" }
                ]
            },
            {
                "id": "analyzer",
                "spec": "code-analysis",
                "mission": "Analyze the inline Rust snippet",
                "input": "fn beta() {}",
                "effect": "read",
                "completion": "Exactly one function is reported",
                "stop_condition": "Stop after static analysis",
                "evidence": [
                    { "kind": "output_contains", "value": "fn=1" }
                ]
            }
        ]
    });

    let output = run_manifest(&manifest.to_string(), &state, false);
    assert!(
        output.status.success(),
        "manifest failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("EVIDENCE-BOUND OCTOPUS MANIFEST"));
    assert_eq!(stdout.matches("EVIDENCE RECEIPT").count(), 2);

    let arms = snapshots_under(&state, "arms");
    assert_eq!(arms.len(), 2);
    assert!(arms
        .iter()
        .all(|snapshot| snapshot.contains("status: completed")));
    assert!(arms
        .iter()
        .any(|snapshot| snapshot.contains("name: reader:code-reader")));
    assert!(arms
        .iter()
        .any(|snapshot| snapshot.contains("name: analyzer:code-analysis")));
    let roots = snapshots_under(&state, "roots");
    assert_eq!(roots.len(), 1);
    assert!(roots[0].contains("status: completed"));
}

#[test]
fn unmet_evidence_fails_the_arm_and_root() {
    let state = state_dir("evidence-failure");
    let manifest = json!({
        "schema": "octopus.arm-manifest/v1",
        "objective": "Reject an arm whose claim is not proven",
        "arms": [
            {
                "id": "proven",
                "spec": "code-analysis",
                "mission": "Count a function",
                "input": "fn alpha() {}",
                "effect": "read",
                "completion": "One function is reported",
                "stop_condition": "Stop after analysis",
                "evidence": [{ "kind": "output_contains", "value": "fn=1" }]
            },
            {
                "id": "unproven",
                "spec": "code-analysis",
                "mission": "Demonstrate a deliberately impossible claim",
                "input": "let value = 1;",
                "effect": "read",
                "completion": "The impossible marker is reported",
                "stop_condition": "Stop after analysis",
                "evidence": [{ "kind": "output_contains", "value": "IMPOSSIBLE_MARKER" }]
            }
        ]
    });

    let output = run_manifest(&manifest.to_string(), &state, false);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("EVIDENCE GATE FAILED"), "stderr={stderr}");

    let arms = snapshots_under(&state, "arms");
    assert_eq!(arms.len(), 2);
    assert!(arms
        .iter()
        .any(|snapshot| snapshot.contains("status: failed")));
    let roots = snapshots_under(&state, "roots");
    assert_eq!(roots.len(), 1);
    assert!(roots[0].contains("status: failed"));
}

#[test]
fn contract_mismatch_is_rejected_before_snapshots_are_created() {
    let state = state_dir("preflight");
    let manifest = json!({
        "schema": "octopus.arm-manifest/v1",
        "objective": "Reject task prose passed to a file-path arm",
        "arms": [
            {
                "id": "bad-reader",
                "spec": "code-reader",
                "mission": "Read a file",
                "input": "inspect\nthe repository",
                "effect": "read",
                "completion": "A file is read",
                "stop_condition": "Stop after reading",
                "allowed_paths": [state],
                "evidence": [{ "kind": "min_output_bytes", "value": 1 }]
            },
            {
                "id": "valid-analysis",
                "spec": "code-analysis",
                "mission": "Analyze code",
                "input": "fn alpha() {}",
                "effect": "read",
                "completion": "One function is reported",
                "stop_condition": "Stop after analysis",
                "evidence": [{ "kind": "output_contains", "value": "fn=1" }]
            }
        ]
    });

    let output = run_manifest(&manifest.to_string(), &state, false);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot contain newlines"),
        "stderr={stderr}"
    );
    assert!(snapshots_under(&state, "arms").is_empty());
    assert!(snapshots_under(&state, "roots").is_empty());
}

#[test]
fn write_arm_requires_explicit_permission_and_then_proves_the_file() {
    let state = state_dir("write-gate");
    fs::create_dir_all(&state).expect("create state directory");
    let target = state.join("written.txt");
    let writer_input = format!("{}|NEW|evidence bound write", target.display());
    let manifest = json!({
        "schema": "octopus.arm-manifest/v1",
        "objective": "Gate and prove a declared write",
        "arms": [
            {
                "id": "writer",
                "spec": "code-writer",
                "mission": "Create the declared fixture",
                "input": writer_input,
                "effect": "write",
                "completion": "The target file exists",
                "stop_condition": "Stop after one write",
                "allowed_paths": [state],
                "evidence": [{ "kind": "file_exists", "path": target }]
            },
            {
                "id": "analysis",
                "spec": "code-analysis",
                "mission": "Provide an independent read-only result",
                "input": "fn alpha() {}",
                "effect": "read",
                "completion": "One function is reported",
                "stop_condition": "Stop after analysis",
                "evidence": [{ "kind": "output_contains", "value": "fn=1" }]
            }
        ]
    });
    let source = manifest.to_string();

    let denied = run_manifest(&source, &state, false);
    assert!(!denied.status.success());
    assert!(!target.exists());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("--allow-write"));

    let allowed = run_manifest(&source, &state, true);
    assert!(
        allowed.status.success(),
        "allowed write failed: stdout={} stderr={}",
        String::from_utf8_lossy(&allowed.stdout),
        String::from_utf8_lossy(&allowed.stderr)
    );
    assert_eq!(
        fs::read_to_string(target).expect("read written evidence"),
        "evidence bound write"
    );
}
