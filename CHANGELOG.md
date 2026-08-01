# Changelog

## v2.9.0 — 2026-08-01

### Added
- Complete Bio-Binaries v0.3.0 project bundled as an independent Cargo crate and native process subsystem instead of ported Octopus modules
- Bounded local Ribosome generation: deterministic embedded template rendering, contained filenames, explicit apply gate, staged rustc compilation, BLAKE3 receipts, no-clobber publication and bounded verified local replication without auto-spawn
- Real Wave-Cryo command data path with validated 1200/600 Hz BFSK contract, `.cryo` integrity load, staged WAV encode, CRC/hash-verified decode, compressed artifact publication and end-to-end roundtrip tests
- Durable versioned WaveField emergent-event sidecar with an 8 MiB load cap, newest-1000 bound, atomic persistence, restart/corruption tests and a real `wave-field events` query
- Functional smoke receipts for generated Ribosome source, compiled binary and successful execution
- Registry, `bio status`, and `bio external <name>` integration for all 33 Bio executables with read/write/control effect classes
- Exact no-shell argument forwarding, 30-second and 512-KiB process limits, explicit mutation authorization, private subsystem runtime state, and durable Octopus root/arm audit snapshots
- Embedded 33-file SHA-256 release inventory with pre-launch integrity refusal and a repeatable `scripts/verify-bio-system.ps1` functional matrix
- A repeatable 33-target paired direct/Octopus benchmark harness, raw CSV evidence, machine identity capture, and 1/2/4/8 concurrency scaling
- Normative Bio benchmark methodology plus a canonical 2026-08-01 result report with explicit claim limits
- Guarded Bio Runtime actuator control plane with two-phase `macrophage`, `synaptic`, and `crispr` plan/apply commands
- Identity/content-bound confirmation tokens plus explicit `--allow-kill` or `--allow-write` effect authorization
- Prepared antigen audit records, exact Synaptic archive inventories, CRC/Merkle gates, transactional CRISPR backup/rollback, and CLI acceptance coverage
- Canonical **Octopus Exoskeleton** product identity and production-safety description
- Native biological homeostasis routing for `macrophage`, immune response planning, synaptic pruning, deterministic Hebbian association and bounded mitosis decomposition
- `homeostasis` and `memory` Marshal task classes that activate bio arms for incident and context-pressure language
- Append-only SHA-256 Resonance Log sidecar with balanced arm counts, root/input/output/topology hashes, idempotent root append and tamper verification
- `resonance --verify --tail N` CLI inspection plus a human-readable workspace journey ledger
- Evidence-bound `manifest` execution with distinct per-arm missions, inputs, effects, path boundaries, completion criteria, stop conditions and evidence rules
- Preflight validation before root creation plus `--allow-write` authorization for write-capable manifest arms
- Completion evidence gates for output text/size, file existence and SHA-256, with receipts persisted in arm snapshots
- Integration coverage for per-arm routing, false-success rejection, contract preflight and explicit write permission
- `docs/PRODUCT_DESCRIPTION.md` covering the control-plane, blade execution and enforced safety boundary
- `marshal <task>` offline task classification and compact topology planning
- Weighted psi selection using operating-system cryptographic entropy (`OsRng`)
- `marshal --execute` dispatch through the existing Octopus pipeline
- Explicit `--allow-write` gate for write-capable topology execution
- Unit and CLI integration coverage for Hungarian classification, weighted boundaries, compact receipts and write refusal
- Independent capability execution-class (`advisory`, local, external, control-plane) and verification-grade (`declared`, `tested`, `observed`) axes
- `capabilities --profile windows-offline` with a measured 131-entry safe-ready projection

### Changed
- The public registry now exposes 225 unique capabilities; all 33 native Bio targets are `real`, local-process and `tested`
- The Windows/offline tested profile now contains 164 entries

### Tests
- Octopus: 357 passing tests (312 unit + 45 integration)
- Bio-Binaries: 62 passing tests
- Native process smoke: 33/33 targets and 7/7 generated artifact checks
- Paired benchmark: 660/660 measured pairs; concurrency scaling: 48/48 jobs
- Bio mutation applies now acquire the exclusive runtime-state lock and require a durable root/arm audit snapshot before touching the target
- Short incident markers use token boundaries, and bounded synaptic analysis reports when input was truncated
- Candidate topologies are filtered to `real`, non-external, at-least-`tested` Windows/offline capabilities before selection
- Marshal receipts carry only task class, candidates, weights and the selected topology instead of repeating task content

### Fixed
- Corrected the Bio homeostasis CLI binary-name parsing defect discovered by the 33-target smoke matrix
- Corrected WaveStore binary decoding of minimal packets; the fixed-width preamble is 32 bytes, not 42, so multi-packet persistence now round-trips
- Isolated Bio integrity sidecars from unrelated global `%TEMP%` pins while retaining stable shared Bio temporary state
- Reclassified `vagus-nerve` from read to write because snapshot mode injects packets into WaveStore
- Synaptic restore rejects incomplete, duplicate, extra, or content-mismatched archive manifests
- CRISPR revalidates the renamed backup before commit and reports rollback failures instead of silently swallowing them
- The endurance guard fails closed when its script is missing, and Macrophage terminates only after same-object identity revalidation
- Removed duplicate test attributes that inflated earlier counts; the canonical candidate suite is now 344 unique tests
- Removed a stale atomic import and restored the package-scoped zero-warning quality gate
- Restored 33 callable runtime adapters and integrations to the public registry without restoring duplicate batch implementations

## v2.8.1 — 2026-07-22

### Fixed
- `code-writer` now validates `path|expected_hash|content` fields independently, preserving multiline content, pipe characters, empty files, trailing spaces and final newlines.
- `run code-writer` and `arm code-writer` preserve raw stdin bytes instead of trimming file content.

### Tests
- Added end-to-end visual-arm coverage through the real `arm code-writer` CLI and snapshot lifecycle; the candidate suite contained 309 unique passing tests at that revision.

## v2.8.0 — 2026-07-16

### Added
- `state-restore plan <state-id>` non-mutating validation and exact confirmation contract
- `state-restore apply <state-id> --confirm <state-id>` with sealed pre-restore backup
- `state-restore recover` plus automatic recovery before ordinary CLI state access
- Cross-process shared/exclusive state locking outside the replaceable state directory
- Same-volume staged directory swap with an external phase journal
- Crash-window tests for rollback, verified commit, corrupt candidate rejection and completed-rollback proof
- Cross-process integration coverage proving restore waits while MCP holds a shared state session

### Changed
- Long-running CLI and MCP commands hold a shared state lock for their full lifetime
- Backup creation and state repair hold an exclusive state lock for a consistent snapshot
- Configured relative state paths are normalized to absolute paths
- Backup copies are synced before sealing, and directory read failures are no longer silently skipped

### Fixed
- Restore rejects backup directories nested inside the live state tree
- Exact backup-id confirmation prevents accidental restore selection
- A failed first journal write cleans its staged candidate without touching live state
- Recovery can prove a completed rollback by matching live state to the sealed pre-restore backup
- A corrupted published candidate is quarantined, rolled back and removed only after the previous state validates
- Windows test builds disable oversized PDB generation and incremental linking to avoid `LNK1140`
- Invalid file/symlink state paths are rejected before a cross-process lock file can be created
- Dot-prefixed live state names now produce single-dot lock, journal and restore sidecars

## v2.7.0 — 2026-07-15

### Added
- `state-backup create` for sealed backups with sorted per-file SHA-256 inventory, byte totals and a completion marker
- `state-backup verify <state-id>` for sealed integrity checks and explicit legacy-unsealed reporting
- Process-isolated default state directories for unit tests, preventing accidental writes to live state
- Corruption, legacy-backup and end-to-end backup CLI regression coverage

### Changed
- New backups are first built under a partial name, verified in place, and published only after the manifest matches the payload
- State path selection is centralized across orchestration, lifecycle snapshots and maintenance

### Fixed
- Unit tests can no longer inherit the live workspace state default
- Tampered, incomplete, duplicated or structurally invalid backup payloads fail verification
- Manifest path handling is component-based and constrained to direct `state-*` backup identifiers

## v2.6.0 — 2026-07-14

### Added
- `state-audit --stale-hours N|--stale-minutes N` read-only state quality report
- `state-repair --stale-hours N|--stale-minutes N` backup-first legacy migration and stale-run recovery
- Dedicated `maintenance.rs` state owner with isolated audit/repair regression tests

### Changed
- Legacy orchestration snapshots migrate to schema 2 with JSON-encoded prompts
- Root child backlinks are reconstructed from valid arm records without dropping existing links
- Old running/resumed records are closed as `timed_out` after the configured threshold

### Fixed
- Invalid/interleaved event records are preserved in the timestamped backup and removed from the live log
- Legacy first-level root-as-parent references are normalized to no parent
- State repair is atomic per file and never mutates before a complete backup exists

## v2.5.1 — 2026-07-14

### Added
- Cross-process event-log lock with stale-lock recovery
- Atomic same-directory replacement for root, arm, and lifecycle snapshots
- Regression coverage for 16 concurrent runtime processes and hostile multiline prompts

### Changed
- Root and arm IDs include the process ID to prevent cross-process collisions
- Orchestration arm snapshots use schema 2 with JSON-encoded prompts
- Root snapshots persist child backlinks when arms are created
- Invalid or missing snapshot statuses are rejected instead of defaulting to running
- Integration state directories include process, sequence, and timestamp isolation

### Fixed
- Interleaved `events.log` records under parallel process load
- Truncated snapshots caused by direct overwrite/append operations
- Multiline prompts injecting fake fields into persisted arm snapshots
- First-level arms incorrectly referencing the root as a parent arm

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
