//! Precise integration tests — Part 1: Capabilities, routing, version
//!
//! All tests use isolated OCTOPUS_STATE_DIR. No tautologies, no `>=185`.

use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_SEQ: AtomicU64 = AtomicU64::new(1);

fn state_dir() -> PathBuf {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let base = env::temp_dir().join(format!(
        "octopus-test-{}-{}-{now}",
        std::process::id(),
        TEST_SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::create_dir_all(&base);
    base
}

fn binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("octopus-runtime.exe");
    if !path.exists() {
        path.pop();
        path.pop();
        path.push("release");
        path.push("octopus-runtime.exe");
    }
    path
}

fn run<S: AsRef<std::ffi::OsStr>>(args: &[S], sd: &PathBuf) -> (i32, String, String) {
    let output = Command::new(binary())
        .args(args)
        .env("OCTOPUS_STATE_DIR", sd)
        .env("OCTOPUS_ALLOWED_ROOTS", sd)
        .output()
        .expect("binary");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.code().unwrap_or(-1), stdout, stderr)
}

fn run_with_guard<S: AsRef<std::ffi::OsStr>>(
    args: &[S],
    sd: &PathBuf,
    guard: &Path,
) -> (i32, String, String) {
    let output = Command::new(binary())
        .args(args)
        .env("OCTOPUS_STATE_DIR", sd)
        .env("OCTOPUS_ALLOWED_ROOTS", sd)
        .env("OCTOPUS_ENDURANCE_GUARD", guard)
        .output()
        .expect("binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn receipt_field(output: &str, label: &str) -> String {
    let prefix = format!("{label}: ");
    output
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("missing receipt field '{label}' in: {output}"))
        .to_string()
}

fn run_with_stdin(args: &[&str], sd: &PathBuf, stdin: &[u8]) -> (i32, String, String) {
    let mut child = Command::new(binary())
        .args(args)
        .env("OCTOPUS_STATE_DIR", sd)
        .env("OCTOPUS_ALLOWED_ROOTS", sd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin)
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait for binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn resonance_path(sd: &Path) -> PathBuf {
    let name = sd
        .file_name()
        .and_then(|value| value.to_str())
        .expect("state directory name")
        .trim_start_matches('.');
    sd.parent()
        .expect("state directory parent")
        .join(format!(".{name}.resonance.log"))
}

#[test]
fn list_exactly_225() {
    let sd = state_dir();
    let (code, out, _) = run(&["list"], &sd);
    assert_eq!(code, 0);
    let lines: Vec<_> = out.lines().collect();
    assert_eq!(
        lines.len(),
        225,
        "list must include 192 Octopus + 33 Bio targets"
    );
    let unique: std::collections::HashSet<&str> = lines.iter().cloned().collect();
    assert_eq!(unique.len(), 225, "list must have 225 unique entries");
    for name in ["viral-infect", "hox-diff", "omega-master", "microscope-mem"] {
        assert!(unique.contains(name), "missing bundled Bio target: {name}");
    }
}

#[test]
fn caps_exactly_225() {
    let sd = state_dir();
    let (code, out, _) = run(&["capabilities"], &sd);
    assert_eq!(code, 0);
    let lines: Vec<_> = out.lines().collect();
    assert_eq!(lines.len(), 225);
    let unique: std::collections::HashSet<&str> = lines.iter().cloned().collect();
    assert_eq!(unique.len(), 225);
}

#[test]
fn windows_offline_profile_filters_external_and_unverified_routes() {
    let sd = state_dir();
    let (code, out, err) = run(&["capabilities", "--profile", "windows-offline"], &sd);
    assert_eq!(code, 0, "{err}");
    let lines: Vec<_> = out.lines().collect();
    assert!(!lines.is_empty());
    assert_eq!(lines.len(), 164);
    assert!(lines.iter().any(|line| line.starts_with("code-reader\t")));
    assert!(lines.iter().any(|line| line.starts_with("summarize\t")));
    assert!(lines.iter().any(|line| line.starts_with("hox-diff\t")));
    assert!(!lines.iter().any(|line| line.starts_with("github\t")));
    assert!(!lines.iter().any(|line| line.starts_with("notion\t")));
    assert!(!lines.iter().any(|line| line.starts_with("apple-notes\t")));
    assert!(lines.iter().all(|line| line.split('\t').count() >= 7));
    assert!(lines
        .iter()
        .all(|line| !line.contains("external-integration")));
    assert!(lines.iter().all(|line| !line.contains("\tdeclared\t")));
}

#[test]
fn list_and_caps_names_match() {
    let sd = state_dir();
    let (_, list_out, _) = run(&["list"], &sd);
    let (_, caps_out, _) = run(&["capabilities"], &sd);
    let list_names: std::collections::HashSet<&str> = list_out.lines().collect();
    let caps_first: std::collections::HashSet<&str> = caps_out
        .lines()
        .filter_map(|l| l.split('\t').next())
        .collect();
    assert_eq!(list_names, caps_first);
}

#[test]
fn known_blade_exit_0() {
    let sd = state_dir();
    let (code, out, _) = run(&["run", "summarize", "test text"], &sd);
    assert_eq!(code, 0);
    assert!(out.contains("[summarize]"));
}

#[test]
fn code_writer_arm_preserves_multiline_stdin_exactly() {
    let sd = state_dir();
    let target = sd.join("visual-arm-output.txt");
    let content = b"first | line\nsecond line  \n\n";
    let mut payload = format!("{}|NEW|", target.display()).into_bytes();
    payload.extend_from_slice(content);

    let (code, out, err) = run_with_stdin(&["--plain", "arm", "code-writer"], &sd, &payload);

    assert_eq!(code, 0, "{err}");
    assert_eq!(std::fs::read(&target).expect("written file"), content);
    assert!(out.contains("COMPOSITE ARM: code-writer"), "{out}");
    assert!(out.contains("Arm Root: root-"), "{out}");
    let snapshot_count = std::fs::read_dir(sd.join("arms"))
        .expect("arm snapshots")
        .filter_map(Result::ok)
        .count();
    assert!(snapshot_count >= 1);
}

#[test]
fn unavailable_blade_exit_nonzero() {
    let sd = state_dir();
    let (code, _, err) = run(&["run", "nano-pdf", "test"], &sd);
    assert_ne!(code, 0);
    // The error message says "is unavailable in this environment"
    assert!(
        err.contains("unavailable"),
        "stderr should say unavailable: {err}"
    );
}

#[test]
fn unsupported_blade_exit_nonzero() {
    let sd = state_dir();
    let (code, _, err) = run(&["run", "apple-notes", "test"], &sd);
    assert_ne!(code, 0);
    assert!(err.contains("not supported"), "{err}");
}

#[test]
fn unknown_blade_exit_nonzero() {
    let sd = state_dir();
    let (code, _, _) = run(&["run", "nonexistent", "test"], &sd);
    assert_ne!(code, 0);
}

#[test]
fn version_matches_package_metadata() {
    let sd = state_dir();
    let (code, out, _) = run(&["--version"], &sd);
    assert_eq!(code, 0);
    assert!(out.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn invalid_state_dir_typed_failure() {
    let sd = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let (code, _, err) = run(&["run", "summarize", "test"], &sd);
    assert_ne!(code, 0);
    assert!(
        err.contains("snapshot_io_failed") || err.contains("failed") || err.contains("denied"),
        "typed failure expected, got: {err}"
    );
}

#[test]
fn marshal_plan_is_compact_and_non_mutating() {
    let sd = state_dir();
    let (code, out, err) = run(
        &["marshal", "diagnose", "the", "failing", "parser", "tests"],
        &sd,
    );
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("MARSHAL / PSI ROUTE"), "{out}");
    assert!(out.contains("task_class=diagnose"), "{out}");
    assert!(out.contains("selected="), "{out}");
    assert!(!out.contains("the failing parser tests"), "{out}");
}

#[test]
fn marshal_write_dispatch_requires_explicit_permission() {
    let sd = state_dir();
    let (code, _, err) = run(&["marshal", "--execute", "javítsd", "a", "kódot"], &sd);
    assert_ne!(code, 0);
    assert!(err.contains("requires --allow-write"), "{err}");
}

#[test]
fn macrophage_runs_the_native_scoped_incident_scan() {
    let sd = state_dir();
    let (code, out, err) = run(
        &["run", "macrophage", "panic", "after", "a", "memory", "leak"],
        &sd,
    );
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("finding=memory_leak"), "{out}");
    assert!(out.contains("finding=panic_crash"), "{out}");
    assert!(out.contains("action=advisory-only"), "{out}");
}

#[test]
fn synaptic_pruning_reports_observed_duplicate_counts() {
    let sd = state_dir();
    let (code, out, err) =
        run_with_stdin(&["run", "synaptic-pruning-v2"], &sd, b"alpha\nbeta\nalpha");
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("input=3"), "{out}");
    assert!(out.contains("kept=2"), "{out}");
    assert!(out.contains("pruned=1"), "{out}");
}

#[test]
fn marshal_activates_homeostasis_for_incident_language() {
    let sd = state_dir();
    let (code, out, err) = run(
        &["marshal", "inspect", "the", "crash", "and", "deadlock"],
        &sd,
    );
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("task_class=homeostasis"), "{out}");
    assert!(out.contains("macrophage"), "{out}");
}

#[test]
fn marshal_executes_memory_homeostasis_arms() {
    let sd = state_dir();
    let (code, out, err) = run(
        &[
            "marshal",
            "--execute",
            "prune",
            "repeated",
            "stale",
            "context",
            "from",
            "memory",
        ],
        &sd,
    );
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("task_class=memory"), "{out}");
    assert!(out.contains("[synaptic-pruning"), "{out}");
    assert!(out.contains("[dna-hebbian]"), "{out}");
}

#[test]
fn bio_macrophage_plan_is_non_mutating_and_apply_is_permission_gated() {
    let sd = state_dir();
    let pid = std::process::id().to_string();
    let plan_args = vec![
        "bio".to_string(),
        "macrophage".to_string(),
        "plan".to_string(),
        pid.clone(),
    ];
    let (code, out, err) = run(&plan_args, &sd);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("MACROPHAGE PLAN"), "{out}");
    assert!(out.contains("mode: dry-run"), "{out}");
    assert!(out.contains("confirm: MAC-"), "{out}");

    let apply_args = vec![
        "bio".to_string(),
        "macrophage".to_string(),
        "apply".to_string(),
        pid,
        "--confirm".to_string(),
        "invalid".to_string(),
    ];
    let (code, _, err) = run(&apply_args, &sd);
    assert_ne!(code, 0);
    assert!(err.contains("requires --allow-kill"), "{err}");
}

#[test]
fn bio_synaptic_plan_is_hash_bound_and_apply_is_permission_gated() {
    let sd = state_dir();
    let data = sd.join("microscope-data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("activations.bin"), b"activation-state").unwrap();
    let executable = sd.join("microscope-mem.exe");
    std::fs::write(&executable, b"test executable identity").unwrap();
    let config = sd.join("config.toml");
    std::fs::write(
        &config,
        format!("[paths]\noutput_dir = \"{}\"\n", data.display()),
    )
    .unwrap();

    let plan_args = vec![
        "bio".to_string(),
        "synaptic".to_string(),
        "plan".to_string(),
        "--executable".to_string(),
        executable.display().to_string(),
        "--config".to_string(),
        config.display().to_string(),
    ];
    let (code, out, err) = run(&plan_args, &sd);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("SYNAPTIC PLAN"), "{out}");
    assert!(
        out.contains("action: archive -> dream -> CRC -> Merkle"),
        "{out}"
    );
    assert!(out.contains("confirm: SYN-"), "{out}");

    let apply_args = vec![
        "bio".to_string(),
        "synaptic".to_string(),
        "apply".to_string(),
        "--executable".to_string(),
        executable.display().to_string(),
        "--config".to_string(),
        config.display().to_string(),
        "--confirm".to_string(),
        "invalid".to_string(),
    ];
    let (code, _, err) = run(&apply_args, &sd);
    assert_ne!(code, 0);
    assert!(err.contains("requires --allow-write"), "{err}");
}

#[test]
fn bio_crispr_cli_requires_permission_then_commits_the_confirmed_bytes() {
    let sd = state_dir();
    let target = sd.join("target.txt");
    let replacement = sd.join("replacement.txt");
    std::fs::write(&target, "old").unwrap();
    std::fs::write(&replacement, "new").unwrap();
    let plan_args = vec![
        "bio".to_string(),
        "crispr".to_string(),
        "plan".to_string(),
        target.display().to_string(),
        replacement.display().to_string(),
    ];
    let (code, out, err) = run(&plan_args, &sd);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("CRISPR PLAN"), "{out}");
    let confirmation = receipt_field(&out, "confirm");

    let denied_args = vec![
        "bio".to_string(),
        "crispr".to_string(),
        "apply".to_string(),
        target.display().to_string(),
        replacement.display().to_string(),
        "--confirm".to_string(),
        confirmation.clone(),
    ];
    let (code, _, err) = run(&denied_args, &sd);
    assert_ne!(code, 0);
    assert!(err.contains("requires --allow-write"), "{err}");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "old");

    let mut apply_args = denied_args;
    apply_args.push("--allow-write".to_string());
    let guard = sd.join("test-endurance-guard.ps1");
    std::fs::write(
        &guard,
        "param([string]$Command)\nif ($Command -eq 'Guard') { exit 0 }\nexit 2\n",
    )
    .unwrap();
    let (code, out, err) = run_with_guard(&apply_args, &sd, &guard);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("CRISPR APPLY"), "{out}");
    assert!(out.contains("status: committed"), "{out}");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "new");
    let backup = receipt_field(&out, "backup");
    assert_eq!(std::fs::read_to_string(backup).unwrap(), "old");
}

#[test]
fn bio_subsystem_status_reports_the_separate_bundled_crate() {
    let sd = state_dir();
    let (code, out, err) = run(&["bio", "status"], &sd);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("BIO SUBSYSTEM"), "{out}");
    assert!(out.contains("layout: separate-bundled-crate"), "{out}");
    assert!(out.contains("bio-binaries\\Cargo.toml"), "{out}");
    assert!(out.contains("availability:"), "{out}");
    assert!(out.contains("33/33 SHA-256 release pins embedded"), "{out}");
}

#[test]
fn bundled_bio_mutation_is_refused_without_explicit_authorization() {
    let sd = state_dir();
    let (code, _, err) = run(&["run", "viral-infect", "payload"], &sd);
    assert_ne!(code, 0);
    assert!(
        err.contains("without explicit mutation authorization"),
        "{err}"
    );
}

#[test]
fn bio_external_rejects_an_unpinned_executable_before_launch() {
    let sd = state_dir();
    let directory = sd.join("bio-bin");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::copy(binary(), directory.join("hox-diff.exe")).unwrap();
    let output = Command::new(binary())
        .args(["bio", "external", "hox-diff", "--", "--version"])
        .env("OCTOPUS_STATE_DIR", &sd)
        .env("OCTOPUS_ALLOWED_ROOTS", &sd)
        .env("OCTOPUS_BIO_BIN_DIR", &directory)
        .output()
        .expect("binary");
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        !output.status.success(),
        "unpinned executable must be refused"
    );
    assert!(
        stderr.contains("SHA-256 mismatch for 'hox-diff'"),
        "{stderr}"
    );
}

#[test]
fn resonance_log_chains_every_finished_root() {
    let sd = state_dir();
    let (first, _, first_err) = run(&["run", "summarize", "first resonance"], &sd);
    assert_eq!(first, 0, "{first_err}");
    let (second, _, second_err) = run(&["run", "code-analysis", "fn alpha() {}"], &sd);
    assert_eq!(second, 0, "{second_err}");

    let (code, out, err) = run(&["resonance", "--verify", "--tail", "2"], &sd);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("integrity: verified"), "{out}");
    assert!(out.contains("entries: 2"), "{out}");
    assert!(out.contains("#1 "), "{out}");
    assert!(out.contains("#2 "), "{out}");
}

#[test]
fn resonance_verification_detects_tampering() {
    let sd = state_dir();
    let (code, _, err) = run(&["run", "summarize", "tamper proof"], &sd);
    assert_eq!(code, 0, "{err}");
    let path = resonance_path(&sd);
    let content = std::fs::read_to_string(&path).expect("resonance log");
    std::fs::write(&path, content.replace("status=completed", "status=failed"))
        .expect("tamper fixture");

    let (verify_code, _, verify_err) = run(&["resonance", "--verify"], &sd);
    assert_ne!(verify_code, 0);
    assert!(verify_err.contains("hash mismatch"), "{verify_err}");
}
