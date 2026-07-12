//! Precise integration tests — Part 1: Capabilities, routing, version
//!
//! All tests use isolated OCTOPUS_STATE_DIR. No tautologies, no `>=185`.

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_SEQ: AtomicU64 = AtomicU64::new(1);

fn state_dir() -> PathBuf {
    let base = env::temp_dir().join(format!(
        "octopus-test-{}",
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

fn run(args: &[&str], sd: &PathBuf) -> (i32, String, String) {
    let output = Command::new(binary())
        .args(args)
        .env("OCTOPUS_STATE_DIR", sd)
        .output()
        .expect("binary");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.code().unwrap_or(-1), stdout, stderr)
}

#[test]
fn list_exactly_191() {
    let sd = state_dir();
    let (code, out, _) = run(&["list"], &sd);
    assert_eq!(code, 0);
    let lines: Vec<_> = out.lines().collect();
    assert_eq!(lines.len(), 191, "list count must be exactly 191");
    let unique: std::collections::HashSet<&str> = lines.iter().cloned().collect();
    assert_eq!(unique.len(), 191, "list must have 191 unique entries");
}

#[test]
fn caps_exactly_191() {
    let sd = state_dir();
    let (code, out, _) = run(&["capabilities"], &sd);
    assert_eq!(code, 0);
    let lines: Vec<_> = out.lines().collect();
    assert_eq!(lines.len(), 191);
    let unique: std::collections::HashSet<&str> = lines.iter().cloned().collect();
    assert_eq!(unique.len(), 191);
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
fn version_is_2_5_0() {
    let sd = state_dir();
    let (code, out, _) = run(&["--version"], &sd);
    assert_eq!(code, 0);
    assert!(out.contains("2.5.0"));
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
