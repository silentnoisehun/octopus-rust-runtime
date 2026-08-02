use std::env;
use std::path::PathBuf;

fn state_dir() -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    env::temp_dir().join(format!(
        "octopus-snapshot-finish-test-{}-{stamp}",
        std::process::id()
    ))
}

#[test]
fn snapshot_finish_write_failure_returns_a_typed_outcome_without_panicking() {
    let state = state_dir();
    env::set_var("OCTOPUS_STATE_DIR", &state);
    env::set_var("OCTOPUS_TEST_FAIL_SNAPSHOT_FINISH", "permission-denied");

    let blade_result =
        std::panic::catch_unwind(|| octopus_runtime::run_outcome("summarize", "probe"));
    let arm_result = std::panic::catch_unwind(|| {
        octopus_runtime::run_arm_outcome("summarize + diagnostics", "probe")
    });

    env::remove_var("OCTOPUS_TEST_FAIL_SNAPSHOT_FINISH");
    env::remove_var("OCTOPUS_STATE_DIR");

    for (path, expected_name, result) in [
        ("blade", "summarize", blade_result),
        ("composite arm", "summarize + diagnostics", arm_result),
    ] {
        let outcome = result.unwrap_or_else(|_| panic!("{path} finish failure must not panic"));
        assert!(outcome.is_failed(), "{path}: {outcome:?}");
        assert_eq!(
            outcome.code.as_deref(),
            Some("snapshot_finish_failed"),
            "{path}: {outcome:?}"
        );
        assert!(
            outcome.output.contains(&format!(
                "[{expected_name}] snapshot finish failed: I/O error: injected snapshot finish write failure"
            )),
            "{path}: {}",
            outcome.output
        );
    }
}
