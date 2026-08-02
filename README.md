# Octopus Exoskeleton

![Octopus Exoskeleton Architecture](./octopus_exoskeleton_blueprint.jpg)

Without Octopus, you give the AI the keyboard and pray. With Octopus, you give it a goal, and Octopus gives it a permission card, a measuring tape, and a diary.

> **AI decides within policy. Blades execute within contracts. The Exoskeleton enforces the boundary.**

Octopus is a policy-gated Rust execution runtime for AI-assisted workflows. It separates what the model wants to do from what the system allows it to do.

## How it works in 10 seconds

1. **ROOT DECISION** — One head owns the decision. One prompt, one accountability.
2. **POLICY GATE** — Every arm must pass a capability gate. If it is not allowed, it fails with a typed error, not a hallucination.
3. **BLADE EXECUTION** — Deterministic work happens in bounded, auditable blades operating within explicit contracts.
4. **AUDIT LOG** — Every root, arm, and event is persisted, SHA-256 chained, and queryable.

> Policy-gated Rust execution runtime for AI-assisted workflows.

Octopus Exoskeleton is a Windows-focused Rust execution layer that separates model decisions from operational execution. The AI interprets the goal and selects an allowed route; deterministic blades perform bounded work under explicit contracts; the runtime enforces capability profiles, authorization, typed failures, snapshots, audit trails, and rollback paths.

It is not a formal safety proof and cannot control actions performed outside its boundary. For operations routed through Octopus, its controls are designed to limit unauthorized, irreversible, or untracked side effects.

Local blades can avoid additional model calls, and independent arms can execute concurrently. End-to-end latency and token use remain workload-dependent; the Bio benchmark measures the process/policy boundary rather than claiming a universal speedup.

## Runtime model

Octopus is an **orchestration topology**, not a scheduler. It models work as a tree: one **head** (root) owns the decision, and multiple **arms** (blades) execute independently. The Rust runtime is the engine that makes this topology real — it creates roots, spawns arms, routes blades through capability gates, persists snapshots, and supports resume, retry, and cancel.

You give it a prompt. It splits the work. Arms run. Results converge. Everything is recorded.

## Why Octopus?

- **One root, one decision** — arms never create sub-roots, so accountability is never diluted
- **Typed failures, always** — Unavailable, Unsupported, execution errors, I/O errors: every failure has a code, not a string
- **Fallible snapshot I/O** — snapshot operations use `Result<T, SnapshotError>` and tested failure paths fail closed.
- **Resume and retry with real state** — resume picks up orphaned arms, retry re-executes with the stored prompt, not a guess
- **Auditable** — every root, arm, and event is persisted. You can `status <id>` any past execution
- **Crash-safe state** — snapshots are replaced atomically, event writes are process-locked, IDs include the process, and malformed status records fail closed
- **Minimal-token Marshal** — local task classification, safe-ready topology filtering, OS-CSPRNG psi selection, compact receipts, and explicit write permission before dispatch
- **225 registry entries with typed status** — 168 are currently marked `real`, 55 are `unavailable`, and 2 are `unsupported` on Windows. The registry includes 33 tested native Bio-Binaries process targets while keeping the Bio crate and executable boundary separate from the Octopus process.

## Quick Start

### Build

```powershell
cargo build --release --locked
```

The binary lands at `target/release/octopus-runtime.exe`.

`code-writer` accepts its transactional contract through stdin as
`path|expected_sha256_or_NEW|content`. Stdin content is byte-preserving,
including multiline text, trailing spaces, and the final newline.

### State Directory

Runtime state (roots, arms, event log, backups and the resonance chain) lives
under a portable, machine-independent default: the OS data-local directory
joined with `octopus-rust-runtime/.octopus-rust` (Windows `%LOCALAPPDATA%`,
Linux `$XDG_DATA_HOME` or `~/.local/share`, macOS
`~/Library/Application Support`), falling back to the current directory. There
is no hardcoded per-developer path in the binary.

```powershell
$env:OCTOPUS_STATE_DIR = "D:\codex\.octopus-rust"  # pin an existing workspace root
octopus-runtime pipeline "summarize || code-analysis" "probe"
```

`OCTOPUS_STATE_DIR` may be absolute or relative (relative paths resolve against
the current directory). Backups default to the `.octopus-rust-backups` sidecar
of the state directory and can be pinned separately with
`OCTOPUS_STATE_BACKUP_DIR`.

### Your First Pipeline

```powershell
octopus-runtime pipeline "summarize || code-analysis" "explain what this codebase does"
```

This creates one root, spawns two arms (`summarize` and `code-analysis`), executes them independently, and converges the results.

### Require Evidence From Every Arm

The legacy `pipeline` command gives every arm the same prompt. Use an evidence-bound manifest when arms need distinct missions, inputs, path boundaries, stop conditions, and completion proof:

```powershell
octopus-runtime manifest examples/evidence-manifest.json
```

The runtime validates the complete manifest before creating snapshots, then runs its arms concurrently under one root. An arm is marked `completed` only after all declared evidence rules pass. Supported evidence kinds are `output_contains`, `min_output_bytes`, `file_exists`, and `file_sha256`. Filesystem evidence must stay inside the arm's `allowed_paths`; write-capable arms additionally require `--allow-write`.

### Activate Biological Homeostasis

The Marshal now recognizes incident and memory-pressure language and activates tested biological analysis arms automatically:

```powershell
octopus-runtime marshal --execute "inspect the crash and deadlock"
octopus-runtime marshal --execute "prune repeated stale context from memory"
octopus-runtime manifest examples/bio-homeostasis-manifest.json
```

`macrophage` performs a bounded input-signature scan, `immune-antibody` maps observed signatures to response plans, `synaptic-pruning[-v2]` measures duplicate context records, `dna-hebbian` builds deterministic association receipts, and `mitosis` proposes bounded work units. These adapters explicitly report `mutation=none` or `advisory-only`; they do not claim that a process was killed, memory was deleted, or a runtime was patched.

### Apply Guarded Bio Actuators

The separate `bio` control plane can perform three real mutations, but never from an advisory blade or an automatic Marshal route. Every operation is two-phase: `plan` inspects the exact target and emits a content- or identity-bound confirmation token; `apply` requires that token plus an explicit effect permission.

```powershell
# Terminate one revalidated, non-protected process.
octopus-runtime bio macrophage plan <PID>
octopus-runtime bio macrophage apply <PID> --confirm <MAC-token> --allow-kill

# Archive Microscope state, run dream consolidation, then verify CRC and Merkle integrity.
octopus-runtime bio synaptic plan
octopus-runtime bio synaptic apply --confirm <SYN-token> --allow-write

# Replace one allowed file transactionally; executable targets default to --version health.
octopus-runtime bio crispr plan <target> <replacement> [--health-arg <arg> ...]
octopus-runtime bio crispr apply <target> <replacement> --confirm <CRI-token> --allow-write [--health-arg <arg> ...]
```

Mutation applies fail closed when the endurance guard or durable audit snapshot is unavailable and hold an exclusive runtime-state lock. Tests inject an isolated guard result; production calls still execute the configured external lease guard. Macrophage binds termination to a revalidated process object and writes a prepared antigen record before the kill. Synaptic accepts only an exact sealed archive inventory and rolls back on validation failure. CRISPR revalidates both staged bytes and the renamed backup before commit, retains the verified backup, and rolls back on a failed health check. A text-file CRISPR operation with no `--health-arg` is explicitly reported as `hash-only`; it is not presented as an executable health check.

`plan` never performs the requested external mutation, but it does create the normal root/arm audit snapshots and Resonance Log entry.

### Run the Native Bio-Binaries Subsystem

The repository includes Bio-Binaries v0.3.0 at `bio-binaries/` as an independent Cargo crate with its own manifest, lockfile, source tree, tests and 33 executable targets. Release executables are reproducible build artifacts and are not committed. Octopus does not port or merge their algorithms; installed builds are cataloged, authorized, SHA-256-verified and started across a process boundary:

```powershell
octopus-runtime bio status
octopus-runtime bio external hox-diff -- .
octopus-runtime bio external viral-infect --allow-mutation -- <fixture> --pattern alpha --replace beta --dry-run
```

Read targets can run directly. Write and control targets fail closed unless `--allow-mutation` is explicit. Arguments are forwarded exactly without shell tokenization; child execution is limited to 64 arguments, 32 KiB of input, 512 KiB of output and 30 seconds. Every executable is pinned by the embedded `bio-binaries/RELEASE_SHA256SUMS` inventory before launch. The filtered child environment preserves the standard Windows toolchain-root variables required for native MSVC discovery without inheriting arbitrary application secrets. Bio self-integrity state and temporary data live in a private subsystem directory instead of colliding with global `%TEMP%` state.

Run the repeatable functional smoke matrix with:

```powershell
.\scripts\verify-bio-system.ps1
```

The current matrix executes one safe functional path through every native target and verifies generated wave/sculpt/audio artifacts plus a real Ribosome source, compiled binary and executable receipt. The Bio crate also proves the CryoFrame -> BFSK WAV -> CryoFrame command path and durable WaveField-event restart path in Rust tests.

### Benchmark the Complete Bio Subsystem

The paired benchmark harness runs every Bio target directly and through the Octopus policy, integrity and process boundary with identical arguments, alternating order and isolated state:

```powershell
.\scripts\benchmark-bio-system.ps1 -Warmup 3 -Samples 20 -Parallelism @(1,2,4,8) -ParallelRepeats 3
```

The current v0.3 diagnostic pilot completed all 33 module cases in both direct and Octopus lanes with three measured samples per lane. Median-of-module-medians latency was 28.053 ms direct and 52.517 ms through Octopus; the median paired boundary cost was 24.157 ms. Concurrency was disabled, so this pilot validates current harness coverage and small-fixture behavior, not throughput or scaling.

The committed 660-pair and 48-job report is historical pre-v0.3 evidence. Ribosome, Wave-Cryo and WaveField semantics changed afterward, so those numbers are not current v0.3 performance claims. See [the benchmark result and current-pilot note](./docs/BIO_BENCHMARK_RESULTS_20260801.md) and [methodology/claim limits](./docs/BIO_BENCHMARK_METHODOLOGY.md).

### Verify the Resonance Chain

Every finished Octopus root is appended to a sidecar SHA-256 chain containing root status, arm counts and input/output/topology hashes. The sidecar survives state-directory replacement and rejects duplicate roots or modified entries:

```powershell
octopus-runtime resonance --verify --tail 10
```

Deployments may maintain a separate human-readable journey ledger; it is not part of this repository or the technical verification chain.

### Let the Marshal Select the Topology

```powershell
octopus-runtime marshal "diagnose the failing parser tests"
octopus-runtime marshal --execute "verify the runtime source"
octopus-runtime marshal --execute --allow-write "<pipeline boundary contract>"
```

`marshal` is a thin technical control plane. It does not solve or repeat the task: it classifies the request with offline rules, keeps only `windows-offline`, non-external, at-least-`tested` topologies, selects among them with weighted operating-system entropy, and emits a compact auditable receipt. Without `--execute` it only plans. Write-capable topologies are refused unless `--allow-write` is also present; safety and authorization are never randomized.

### Check What Happened

```powershell
octopus-runtime status root-12345-1-1783870125403
```

### List Everything Available

```powershell
octopus-runtime list              # all 225 capabilities
octopus-runtime capabilities      # full registry with status, effect class and evidence grade
octopus-runtime capabilities --profile windows-offline  # 164 safe-ready entries
```

## CLI Reference

| Command | What it does |
|---------|-------------|
| `list` | List all 225 capability names. |
| `capabilities [--profile all\|windows-offline]` | List capability status, execution class and verification grade; optionally keep only the 164 Windows/offline safe-ready routes. |
| `run <blade> <prompt>` | Run one blade as a standalone arm |
| `arm <blade> <prompt>` | Create and execute a single arm |
| `pipeline <spec> <prompt>` | Run composite arms under one root. Use `+` for sequential, `\|\|` for parallel. |
| `manifest [--allow-write] <path\|->` | Validate and execute a v1 per-arm JSON manifest; `-` reads exact JSON from stdin. Completion is gated by declared evidence. |
| `resonance [--verify] [--tail N]` | Verify and inspect the append-only root-level SHA-256 resonance chain. |
| `bio status` | Report the separate bundled Bio crate, release-pin state and availability of all 33 executables. |
| `bio external <name> [--allow-mutation] -- [args...]` | Launch one SHA-256-pinned Bio executable with exact arguments and effect authorization. |
| `bio <macrophage\|synaptic\|crispr> <plan\|apply>` | Plan or explicitly authorize guarded process termination, Microscope consolidation, or transactional file replacement. |
| `marshal [--execute] [--allow-write] <task>` | Select a safe topology with a minimal-token, psi-weighted Marshal; dispatch only when explicitly requested. |
| `status <root-or-arm-id>` | Query execution status and duration |
| `resume <arm-id>` | Resume an orphaned or incomplete arm |
| `retry <arm-id>` | Re-execute an arm with its original stored prompt |
| `cancel <arm-id>` | Cancel a running arm |
| `orphans` | List arms without a finished root |
| `state-audit [--stale-hours N|--stale-minutes N]` | Read-only legacy-schema, stale-run and event-quality audit |
| `state-repair [--stale-hours N|--stale-minutes N]` | Back up and normalize legacy state before closing stale runs and cleaning invalid events |
| `state-backup create` | Create, seal and verify an immutable state backup with a SHA-256 inventory |
| `state-backup verify <state-id>` | Verify a sealed backup, or report a readable legacy backup as unsealed |
| `state-restore plan <state-id>` | Validate a sealed backup and show a non-mutating restore plan plus exact confirmation token |
| `state-restore apply <state-id> --confirm <state-id>` | Restore under an exclusive lock with a sealed pre-backup, same-volume stage and crash journal |
| `state-restore recover` | Recover, roll back or finish an interrupted journaled restore transaction |
| `mcp` | Output MCP-compatible tool list |

## Architecture

### Capability Routing

Every blade passes through four phases before execution:

```
Blade requested
  → Phase 1: Status gate (Unavailable? Unsupported? → typed failure, stop)
  → Phase 2: Direct adapter route (6 adapters skip advisory wrapping)
  → Phase 3: Safe process/external infrastructure, including pinned Bio executables
  → Phase 4: RealBlades smart wrapper (detects usage/placeholder/simulation)
  → Execute
```

The six direct adapters are `code-reader`, `code-writer`, `diagnostics`, `git-nexus`, `github`, and `github-manager`. `pipeline-architect + rust-surgeon` runs through the transactional composite route; deterministic analysis blades such as `summarize` run through the advisory wrapper.

The registry exposes three independent axes:

- **status** — `real`, `unavailable`, `unsupported`, or `deprecated`;
- **execution class** — `advisory`, `local-operation`, `external-integration`, or `control-plane`;
- **verification grade** — `declared`, `tested`, or `observed`.

`windows-offline` requires `real`, rejects `external-integration`, and requires at least `tested`. It does not grant write authority.

### Orchestration Lifecycle

```
create_root(prompt)                   ─── root owns the decision
  ├─ create_arm(root_id, name, prompt) ─── arm receives the contract
  │    └─ execute_blade_under_root     ─── blade runs, no sub-root created
  │         └─ finish_arm(id, outcome) ─── arm reports back
  └─ finish_root(id, outcome)          ─── root closes the book
```

Snapshot events are appended at each step. The root ID is a real, queryable identifier — never a hash.
After root completion, a separate append-only resonance sidecar records balanced arm counts plus input, output and topology hashes. Its hash chain is independent from mutable status rendering.

### Project Structure

```
octopus-rust-runtime/
├── src/
│   ├── lib.rs              Entry point, CLI dispatch, internal executors
│   ├── arm_manifest.rs     Per-arm schema, preflight boundaries and completion evidence gates
│   ├── bio.rs              Truthful homeostasis adapters and deterministic bio activation outputs
│   ├── bio_actuator.rs     Guarded plan/apply process, memory and file actuators
│   ├── bio_system/         Separate Bio subsystem catalog, integrity pins and process adapter
│   ├── capability.rs       Routing, effect/evidence axes, profiles and blade gates
│   ├── orchestration.rs    Root/arm lifecycle, create_arm_restricted
│   ├── resonance.rs        Append-only SHA-256 root chain, verification and CLI report
│   ├── snapshot.rs         Result-based snapshot I/O and typed failures
│   ├── maintenance.rs      Backup sealing/verification, state audit and repair
│   ├── marshal.rs          Minimal-token task classification and psi topology selection
│   ├── state_lock.rs       Cross-process shared/exclusive state lock
│   ├── state_path.rs       Live/test state-directory isolation
│   ├── real_blades.rs      Blade implementations
│   ├── mcp.rs              MCP tool serialization
│   └── process.rs          Safe process execution with timeout
├── tests/
│   ├── integration_main.rs     25 integration tests (invariants, Bio routing/actuators, integrity, resonance, profiles and Marshal CLI)
│   ├── integration_lifecycle.rs 16 lifecycle tests (root, status, backup, restore, locks, retry, cancel, concurrency)
│   └── integration_manifest.rs  4 manifest tests (routing, evidence failure, preflight and write permission)
├── examples/
│   ├── evidence-manifest.json   Read-only manifest with two independent arm contracts
│   └── bio-homeostasis-manifest.json Evidence-bound macrophage, pruning and mitosis arms
├── scripts/
│   ├── verify-bio-system.ps1   33-target native functional smoke and artifact validation
│   └── benchmark-bio-system.ps1 Paired direct/Octopus latency and concurrency benchmark
├── bio-binaries/               Independent v0.3.0 Cargo crate with 33 executable targets
├── docs/
│   ├── CAPABILITY_MATRIX.md    Full blade inventory with status
│   ├── BIO_BENCHMARK_METHODOLOGY.md Normative 33-target measurement and claim protocol
│   ├── BIO_BENCHMARK_RESULTS_20260801.md Recorded paired latency and scaling evidence
│   ├── PRODUCT_DESCRIPTION.md  Canonical product identity, safety boundary and operating model
│   └── OCTOPUS_2_PRODUCTION_PLAN.md  Roadmap and verified state
├── Cargo.toml
├── README.md
├── CHANGELOG.md
└── .gitignore
```

## Quality Gates

Every change must pass these, in order:

```powershell
cargo fmt --check                        # Formatting
cargo clippy --locked --all-targets -- -D warnings  # Zero warnings
cargo test --locked                      # All Octopus tests green (361 currently)
cargo test --manifest-path bio-binaries/Cargo.toml --locked -j1  # 62 Bio tests
cargo build --release --locked           # Release binary
```

Generated release binaries and Cargo target trees, including the bundled Bio crate's local `target/`, stay outside Git history and must be rebuilt from the committed sources.

Then verify invariants:

```powershell
octopus-runtime list          # 225, 225 unique
octopus-runtime capabilities  # 225, 225 unique
octopus-runtime capabilities --profile windows-offline  # 164, no external/declared entries
.\scripts\verify-bio-system.ps1  # 33/33 native functional paths plus artifact receipts
.\scripts\benchmark-bio-system.ps1 -Warmup 3 -Samples 20  # paired latency benchmark
octopus-runtime pipeline "summarize || code-analysis" "probe"
# Must produce real root ID, exit 0
```

## Verified v2.9.0 state — 2026-08-01

| Metric | Value |
|--------|-------|
| Octopus tests | 361 (315 unit + 46 integration), 0 failed |
| Bio-Binaries tests | 62, 0 failed |
| Runtime Clippy | clean, `--all-targets -- -D warnings` (Rust stable); CI actions are commit-pinned |
| Test hygiene | duplicate test attributes removed; every reported test is unique |
| Capabilities | 225 unique: 168 `real`, 55 `unavailable`, 2 `unsupported` |
| Windows/offline profile | 164 entries; 0 external integrations; 0 `declared` routes |
| Native Bio subsystem | 33/33 functional smoke paths; 7/7 artifact checks |
| Bio release integrity | 33 embedded SHA-256 pins; tampered executable refused before launch |
| Current v0.3 pilot | 33/33 direct and Octopus cases; 3 measured samples/lane; no concurrency claim |
| Historical benchmark | 660/660 pre-v0.3 pairs and 48/48 scaling jobs; retained as historical evidence only |

## Known limitations

- Windows 64-bit is the tested platform; Linux is untested and macOS-only blades return typed `Unsupported` outcomes.
- Macrophage, synaptic, and CRISPR apply operations require `OCTOPUS_ENDURANCE_GUARD` to name the external lease-guard PowerShell script; there is no machine-specific fallback.
- The controls apply only to operations routed through Octopus and have not undergone a formal verification or independent security audit.
- Bio `microscope-mem` persistence delegation, successful multi-peer `collective-sync`, and live audio-device RX remain incomplete.
- BioMessage keyed authentication exists, but JOIN admission/session enforcement is incomplete; do not expose `omega-master` to untrusted networks.
- Release executables are not stored in Git. Build them locally and regenerate the pin inventory for the exact binaries being deployed.

Full version history in [CHANGELOG.md](./CHANGELOG.md).

## License

MIT — see [LICENSE](./LICENSE).
