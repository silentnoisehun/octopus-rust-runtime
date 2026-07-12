[See CHANGELOG.md](./CHANGELOG.md) for complete version history.


# Octopus Rust Runtime v2.5

Standalone native Rust blade runtime for Octopus orchestration.

Version 2.5.0 � 191 capabilities, 273 tests, panic-free snapshots, real root-arm lifecycle.

## Current State (Post-Audit v2.5)

| Metric | Value |
|--------|-------|
| Build | ? clean |
| Clippy | ? clean (strict `-D warnings`) |
| Tests | 272 (253 unit + 19 integration), 0 failed |
| Capabilities | 191 (list) / 191 (capabilities) |
| Release SHA-256 | C3FA970188F01E400B0B27EA5375AB74A247583DF91AF69EF8C0772288C16734 |
| Installed SHA-256 | Same as release (verified) |
| Target in Git | Removed (`.gitignore` + `git rm --cached`) |
| Cargo metadata | license=MIT, repository, publish=false |

## Audit Fixes Applied

1. **Typed execution**: `capability::execute` uses phase-based routing. Unavailable/Unsupported
   capabilities return typed failures, not Completed-wrapped strings.
2. **Real adapters first**: code-reader, code-writer, diagnostics, git-nexus, github,
   github-manager route directly, never through RealBlades.
3. **Root-arm lifecycle**: Every run, arm, pipeline uses create_root/create_arm/finish_arm/finish_root.
4. **Real resume/retry/cancel**: resume dispatches orphaned work; retry re-executes with
   stored prompt; cancel attempts process termination.
5. **Panic-free snapshots**: No `.expect()` on I/O. API returns `Result<T, SnapshotError>`.
   Drop is panic-safe.
6. **Clippy**: Unused `mode` and `render_capabilities_for_mcp` cleaned.
7. **Integration tests**: 20 new binary/integration tests in `tests/`.

## Commands

```
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
```

CLI commands: `list`, `capabilities`, `run`, `arm`, `pipeline`, `status`, `resume`,
`retry`, `cancel`, `orphans`, `mcp`.

## Modified Files (6)

- `Cargo.toml` � Added license, repository, homepage, publish=false
- `src/capability.rs` � Phase-based routing, Unavailable/Unsupported gating, real adapter priority
- `src/lib.rs` � Real root-arm lifecycle, actual resume/retry/cancel execution
- `src/mcp.rs` � Removed unused render_capabilities_for_mcp
- `src/orchestration.rs` � Added prompt field to ArmRecord for genuine re-execution
- `src/snapshot.rs` � Removed all .expect() panics, Result-based API, panic-safe Drop

## New Files (3)

- `.gitignore` � `/target/` and other build artifacts
- `tests/integration_cli.rs` � 11 integration tests for CLI, typed routing
- `tests/integration_orch.rs` � 9 integration tests for orchestration, side effects

## Capability Routing

Phase 1: Check capability status (Unavailable/Unsupported � typed failure)
Phase 2: Route real local adapters directly
Phase 3: Route process/external blades through safe infrastructure
Phase 4: Fall through to RealBlades for pure algorithm blades

## Orchestration Lifecycle

1. create_root(prompt) � RootRecord (Running)
2. create_arm(root_id, name, prompt, parent_id) � ArmRecord
3. Execute blade(s)
4. finish_arm(arm_id, outcome)
5. finish_root(root_id, outcome)

Snapshot parent uses actual root ID, not literal "O".
