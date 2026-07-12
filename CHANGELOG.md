# Changelog

## v2.5.0 — 2026-07-12

### Added
- Inner executor functions without root creation (`execute_blade_under_root`, `execute_arm_under_root`)
- Composite-aware dispatcher for resume/retry using stored prompts
- `create_arm_restricted` orchestration method
- `try_start` / `try_finish` fallible snapshot API
- `blade_outcome_from_string` smart wrapper for usage/placeholder detection
- Precise integration tests: 19 tests with isolated `OCTOPUS_STATE_DIR`
- `tests/integration_main.rs` — 10 tests: cap invariants, typed routing, invalid state
- `tests/integration_lifecycle.rs` — 9 tests: pipeline root, status, resume/retry/cancel
- `.gitignore` with `/target/` and build artifacts
- CHANGELOG.md

### Changed
- `run_pipeline_outcome` creates exactly 1 root with N child arms (no nested roots)
- `render_arm`/`render_pipeline` show real orchestration root ID instead of hash
- `ArmRecord.prompt` stores actual prompt text for genuine re-execution
- `persist_arm` includes multiline `prompt:` field
- `capability::execute` Phase 4 uses `blade_outcome_from_string` for typed failures
- `ArmSnapshot::start` removed noop fallback (now panics on I/O error)
- `ArmSnapshot::finish` deprecated in favor of `try_finish` returning `Result`
- `let _ =` replaced with explicit error handling on persistence paths
- `sag_wrong_format` test updated: usage error → typed `blade_execution_failed`
- Integration tests use exact 191 count (no `>=185`)
- Cargo.toml: license=MIT, repository, homepage, publish=false

### Fixed
- Pipeline no longer creates nested/duplicate orchestration roots
- Resume/retry now dispatches actual work (not just record updates)
- Snapshot I/O failures return typed `snapshot_io_failed` (no silent Completed)
- Unavailable/Unsupported blades return typed failures, not Completed-wrapped strings
- Real adapters (code-reader, git-nexus, github) route before RealBlades
- `short_hash` unused function removed
- Clippy: unused `mode` made pub, `render_capabilities_for_mcp` removed
- Unused `sha2`/`hex` imports removed from `lib.rs`
- readonly snapshot `start`/`id` methods marked `#[allow(dead_code)]`

### Removed
- Noop snapshot fallback in `ArmSnapshot::start`
- `render_capabilities_for_mcp` (unused wrapper)
- `short_hash` function (unused after root ID migration)
- `>=185` tautological assertions in integration tests
