//! Integration tests for Wave Echo subsystem

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_SEQ: AtomicU64 = AtomicU64::new(1);

fn state_dir() -> PathBuf {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let base = env::temp_dir().join(format!(
        "octopus-wave-test-{}-{}-{now}",
        std::process::id(),
        TEST_SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::create_dir_all(&base);
    base
}

fn binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin_name = format!("octopus-runtime{}", env::consts::EXE_SUFFIX);
    path.push("target");
    path.push("debug");
    path.push(&bin_name);
    if !path.exists() {
        path.pop();
        path.pop();
        path.push("release");
        path.push(&bin_name);
    }
    path
}

fn run_wave(
    args: &[&str],
    sd: &PathBuf,
    vault: &PathBuf,
    spine: &PathBuf,
) -> (i32, String, String) {
    let mut full_args = vec!["wave-echo"];
    full_args.extend(args);

    let output = Command::new(binary())
        .args(&full_args)
        .env("OCTOPUS_STATE_DIR", sd)
        .env("OCTOPUS_ALLOWED_ROOTS", sd)
        .env("OCTOPUS_WAVE_VAULT", vault)
        .env("OCTOPUS_SPINE_PATH", spine)
        .output()
        .expect("binary execution");

    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn test_wave_echo_cli_flow() {
    let sd = state_dir();
    let vault = sd.join("vault.wv3");
    let spine = sd.join("spine.rgs");

    // 1. Add native template
    let txt = sd.join("templates.txt");
    fs::write(
        &txt,
        "cli-demo ::: rust, cli, demo ::: WRITE||{{path}}@@@fn main() { println!(\"{{name}}\"); } ::: rust, clap ::: 0.85\n",
    )
    .unwrap();

    let (code, out, err) = run_wave(&["add-native", txt.to_str().unwrap()], &sd, &vault, &spine);
    assert_eq!(code, 0, "add-native failed: {err}");
    assert!(out.contains("ADD NATIVE: added 1"));

    // 2. Vault list
    let (code, out, err) = run_wave(&["list"], &sd, &vault, &spine);
    assert_eq!(code, 0, "list failed: {err}");
    assert!(out.contains("cli-demo"));

    // 3. Vault search
    let (code, out, err) = run_wave(&["vault-search", "rust cli demo"], &sd, &vault, &spine);
    assert_eq!(code, 0, "vault-search failed: {err}");
    assert!(out.contains("cli-demo"));

    // 4. Activate
    let target = sd.join("output.rs");
    let (code, _out, err) = run_wave(
        &[
            "activate",
            "1",
            "--task",
            "demo",
            "--name",
            "mytool",
            "--path",
            target.to_str().unwrap(),
            "--exec",
        ],
        &sd,
        &vault,
        &spine,
    );
    assert_eq!(code, 0, "activate failed: {err}");
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        "fn main() { println!(\"mytool\"); }"
    );

    // 5. Feedback LTP
    let (code, out, err) = run_wave(&["feedback", "1"], &sd, &vault, &spine);
    assert_eq!(code, 0, "feedback failed: {err}");
    assert!(out.contains("LTP (+0.02)"));

    // 6. Status and Pump
    let (code, out, err) = run_wave(&["status"], &sd, &vault, &spine);
    assert_eq!(code, 0, "status failed: {err}");
    assert!(out.contains("WAVE ECHO STATUS"));

    let (code, out, err) = run_wave(&["pump"], &sd, &vault, &spine);
    assert_eq!(code, 0, "pump failed: {err}");
    assert!(out.contains("WAVE PUMP"));
}
