# Octopus Rust Runtime v2.5

Standalone native Rust runtime extracted from Hope's blade layer.

The release build exposes 190 registered blade names from the copied dispatch table plus the `pipeline-architect` composite (191 total). All thirteen Rust source files in Hope's `src/blade` tree are copied into this crate, including twelve function batches, helpers, resilient dispatch, threaded pipeline execution, tests, and Merkle result binding.

The standalone runtime currently passes 253 Rust tests, including typed outcomes, real local adapters, transactional guarded writes, MCP error propagation, exact-boundary locking, single-cut surgery, safe process runner, git-nexus refactored to use it, snapshots, real GitHub adapters, approval layer, persistent orchestration, CLI orchestration commands, MCP orchestration tools, typed capability contracts, pure algorithm blades, process wrapper blades, external API blades, meta/documentation blades, and the copied blade suite.

Every execution returns one typed `ExecutionOutcome` with `completed` or `failed` status and an optional stable failure code. That same outcome drives snapshot state, CLI exit codes, composite-arm aggregation, pipeline aggregation, and MCP `isError`; status is no longer inferred from rendered text.

## Commands

```powershell
octopus-runtime list
octopus-runtime capabilities
octopus-runtime run code-reader "src/main.rs"
octopus-runtime run diagnostics "src/lib.rs"
octopus-runtime run git-nexus "."
octopus-runtime run code-writer "src/value.rs|NEW|fn value() -> u8 { 1 }"
octopus-runtime arm "pipeline-architect + rust-surgeon" "Repair the Rust runtime"
octopus-runtime pipeline "pipeline-architect + rust-surgeon || code-reader + diagnostics" "Repair the Rust runtime"
octopus-runtime status <root-id>
octopus-runtime resume <root-id>
octopus-runtime retry <arm-id>
octopus-runtime cancel <root-id>
octopus-runtime orphans
octopus-runtime mcp
```

Prompts can also arrive through standard input. `arm` mixes `+`-separated skills sequentially inside one composite arm. `pipeline` executes `||`-separated composite arms in parallel on native Rust threads.

## Capabilities

`capabilities` reports the truthful execution mode of every registered name:

- `local-read`: a real, read-only local adapter;
- `local-write`: a real, guarded transactional write adapter;
- `composite`: an Octopus control component;
- `copied-native`: the copied native Rust implementation, which may be deterministic or simulated rather than an external integration.

### Real Adapters (v1.2-1.3)

- **code-reader**: real file read with path validation, 1 MiB limit, allowed-root enforcement, typed failures
- **code-writer**: transactional write with `path|expected_sha256_or_NEW|content`, stale-hash protection, atomic rename, backup, rollback
- **diagnostics**: real file read with analysis output
- **git-nexus**: real `git status` via safe process runner with timeout, NO_COLOR, GIT_OPTIONAL_LOCKS, availability probe
- **pipeline-architect + rust-surgeon**: exact-boundary lock and transactional code replacement

### Safe Process Runner (v1.3)

New `src/process.rs` module provides:

- Direct `std::process::Command` — no shell execution
- Executable allowlist (sh, bash, cmd, powershell blocked)
- Structured argument building
- Environment variable filtering (PATH, SystemRoot preserved; secrets not leaked)
- Configurable timeout with child kill
- stdout/stderr size limits (1 MiB default)
- UTF-8 and binary output handling
- Typed stable error codes: `shell_blocked`, `argument_injection`, `process_spawn_failed`, `process_timeout`, `non_zero_exit`, `invalid_cwd`, `empty_executable`
- Exit code preservation
- Secret redaction in output
- NO_COLOR by default
- Optional GIT_OPTIONAL_LOCKS=0

### Capability Matrix

Full audit of all 191 capabilities: [`docs/CAPABILITY_MATRIX.md`](./docs/CAPABILITY_MATRIX.md)

The evidence-driven plan for promoting every blade to a truthful, production
quality state is [`docs/OCTOPUS_2_PRODUCTION_PLAN.md`](./docs/OCTOPUS_2_PRODUCTION_PLAN.md).

Status summary:
- **real**: 7 adapters verified with tests (code-reader, code-writer, diagnostics, git-nexus, github, github-manager, pipeline-architect)
- **real-algorithm**: 191 blades with real deterministic implementations in `real_blades.rs`
- **unavailable**: 24 external services (credentials/CLI needed)
- **unsupported**: 2 (macOS-only: apple-notes, bear-notes)

## Architecture

### Modules

| Module | Purpose |
|--------|---------|
| `lib.rs` | Public API: run, run_arm, run_pipeline, list, capabilities |
| `main.rs` | CLI entry point with clap |
| `outcome.rs` | Typed ExecutionOutcome with status and code |
| `capability.rs` | Real local adapters and capability catalog |
| `contract.rs` | Typed input/output contracts with validation (v2.0) |
| `real_blades.rs` | Pure algorithm blade implementations (v2.1) |
| `composite.rs` | pipeline-architect + rust-surgeon boundary contract |
| `process.rs` | Safe process runner (v1.3) |
| `external.rs` | External adapter infrastructure (v1.5) |
| `approval.rs` | Approval layer with tokens and audit (v1.6) |
| `orchestration.rs` | Persistent orchestration with lifecycle (v1.7) |
| `snapshot.rs` | Automatic .snap lifecycle management |
| `mcp.rs` | MCP server over stdio |
| `blade/` | Copied Hope dispatch and 12 batch files |

### Snapshot System

Every `run` and every arm inside `pipeline` receives an automatic Rust-managed snapshot before execution and a completion record afterward. Snapshots are human-readable `.snap` files with a plain append-only `events.log` under `D:\codex\.octopus-rust` by default. Set `OCTOPUS_STATE_DIR` to override the location. No Python or JSON is used for runtime snapshots.

### External Adapter Infrastructure (v1.5)

The `external.rs` module provides:

- Availability probes for executables and credentials
- Authentication state detection (gh auth status)
- Rate limit detection from stderr
- Secret redaction in output
- Helper functions: `probe_gh_auth`, `probe_executable`, `require_auth`, `redact_tokens`

### Approval Layer (v1.6)

The `approval.rs` module provides:

- Action plans with typed operations and parameters
- Approval tokens with expiry and optional deny
- One-time token consumption with replay protection
- Audit log of all approval decisions
- Idempotency support for repeated requests

### Persistent Orchestration (v1.7)

The `orchestration.rs` module provides:

- Root and arm records with full lifecycle tracking
- Parent-child relationships between roots and arms
- File locking to prevent concurrent modifications
- Retry policy with configurable limits and backoff
- Circuit breaker for fault tolerance
- Orphaned arm detection and cleanup
- Resume from interrupted executions
- Disk persistence with atomic writes
- Snap file parsing for root and arm records

### CLI Orchestration Commands (v1.8)

New CLI commands for managing orchestration:

- `status <root-id>` — Show status of a root and its events
- `resume <root-id>` — Resume an interrupted orchestration
- `retry <arm-id>` — Retry a failed/timed-out arm
- `cancel <root-id>` — Cancel a running root and all its arms
- `orphans` — List orphaned arms

### MCP Orchestration Tools (v1.9)

New MCP tools for orchestration management:

- `octopus_status(root_id)` — query root status
- `octopus_resume(root_id)` — resume interrupted orchestration
- `octopus_retry(arm_id)` — retry failed arm
- `octopus_cancel(root_id)` — cancel running orchestration
- `octopus_orphans()` — list orphaned arms

### Typed Capability Contracts (v2.0)

Each capability now declares a typed contract with:

- **Version**: semantic version string (e.g., "1.2", "1.5")
- **Group**: category (local, external, composite, unknown)
- **Input**: typed field definitions with validation rules
- **Output**: expected output format
- **Deprecation**: optional deprecation message

Input types: `file_path`, `text`, `hash`, `command`, `json`, `any`

The `capabilities` command now shows version and group for each blade. Input validation runs before blade execution for real adapters.

## MCP

`octopus-runtime mcp` starts a native MCP server over stdio. It exposes:

- `octopus_list`
- `octopus_capabilities`
- `octopus_run`
- `octopus_arm`
- `octopus_pipeline`
- `octopus_status` — query root orchestration status
- `octopus_resume` — resume interrupted orchestration
- `octopus_retry` — retry a failed arm
- `octopus_cancel` — cancel running orchestration
- `octopus_orphans` — list orphaned arms

Drop-in configs are available in [`opencode.json`](./opencode.json) and [`.mcp.json`](./.mcp.json). Point Claude Code, OpenCode, or any MCP client at `octopus-runtime.exe mcp`.

Execution failures remain valid MCP tool results with `isError: true` and machine-readable `status` and `code` metadata. Malformed MCP requests still use JSON-RPC errors.

## Source

The `src/blade` tree is a standalone copy of Hope Ultimate's native blade dispatch and all twelve function batches. The crate has no path dependency on Hope.

The copied blade module is Clippy-isolated at its module boundary so the twelve batch files remain byte-identical to Hope. Standalone runtime, snapshot, composite, CLI, and MCP code remains subject to the crate's strict Clippy gate.

## Testing

253 tests across 12 modules:

- `process.rs` — safe runner: shell blocking, injection prevention, timeout, cwd validation, secret redaction, Windows compatibility
- `capability.rs` — real file adapters, transactional writes, stale-hash rejection, path enforcement, GitHub adapters
- `outcome.rs` — typed status and exit codes
- `composite.rs` — boundary lock and surgical replacement
- `external.rs` — external adapter probes, auth detection, rate limiting, secret redaction
- `approval.rs` — action plans, approval tokens, deny, replay protection, idempotency
- `orchestration.rs` — root/arm records, lifecycle, locking, retry, circuit breaker, orphan detection, resume, persistence
- `mcp.rs` — MCP tool result format
- `real_blades.rs` — all 191 blade implementations with comprehensive tests
- `blade/` — copied batch tests (batch8, 9, 10, 11, 12)
- `lib.rs` — failover, empty arm, unknown blade, capability audit, pure-algorithm verification

## Build

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

Release binary SHA-256 verified against installed binary.
