# Bio-Binaries v0.3.0

**33 bio-inspired native Rust utilities** for local orchestration, analysis, transformation, state modeling, and signal/file processing.

A modular ecosystem where each command uses a biological or physical metaphor. BioMessage v2 is a binary wire format; some CLI output, configuration, registry storage, and legacy bridges use JSON or text.

> **Terminology note:** biological and quantum terms are architectural metaphors for ordinary software algorithms. This project does not claim quantum computation, physical entanglement, electromagnetic actuation, biological life, or machine consciousness.

---

## Architecture

### Layer 1: Bio-Evolution
- `viral-infect` — infection cascade, signal propagation
- `hox-diff` — differential activation, gene expression
- `plasmid-dream` — dream sequence generation
- `plasmid-inject` — vector delivery system
- `mutation-sentinel` — file mutation monitoring

### Layer 2: Quantum-Space
- `telepathy-sync` — entanglement synchronization
- `telepathy-entangle` — quantum correlation
- `eqm-pulse` — electromagnetic pulse shaping
- `eqm-methy` — methylation state encoding
- `grid-warp` — spatial distortion fields
- `path-resonance` — resonance path finding
- `aether-fabric` — topological field mapping
- `aether-excite` — quantum field excitation

### Layer 3: Machine-Brain
- `borg-cube` — collective consciousness node
- `brain-synapse` — synaptic connection modeling
- `brain-connectome` — connectome reconstruction
- `collective-sync` — hive mind synchronization
- `nexus-logic` — logic gate fusion
- `microscope-mem` — memory layer interface
- `ribosome-synth` — bounded Rust source synthesis, compilation, and verified local mitosis
- `vagus-nerve` — internal organ sensing (CPU/RAM → WaveField)

### Layer 4: Resonance & Homeostasis
- `wave-encoder` — wave pattern encoding (432 Hz base)
- `wave-sculptor` — wave shaping filter (DSP)
- `wave-field` — self-organizing interference field
- `iron-resonate` — iron-core resonance profiling
- `magneto-acoustic` — acoustic-magnetic coupling (code → audio)
- `magneto-geo` — geomagnetic field interaction (code heatmap)
- `mycelium-spread` — fungal network propagation (filesystem mapping)
- `homeostasis` — system equilibrium maintenance

### Control & Audio
- `omega-master` — Queen orchestrator (central server)
- `omega-point` — singularity convergence detector
- `wave-cryo-tx` — acoustic cryo-transmitter (BFSK modem)
- `wave-cryo-rx` — acoustic cryo-receiver (BFSK demodulation)

### All Binaries (33)

| Binary | Layer | Description | Status |
|--------|-------|-------------|--------|
| `omega-master` | Control | Queen orchestrator — drone registration, task dispatch, BioMessage protocol | ✅ |
| `omega-point` | Control | Convergence detector — system coherence monitoring | ✅ |
| `viral-infect` | Bio-Evolution | Regex-based code transformation at scale | ✅ |
| `hox-diff` | Bio-Evolution | Project structure differentiator | ✅ |
| `plasmid-dream` | Bio-Evolution | Predictive error analyzer — build runner + trend analysis | ✅ |
| `plasmid-inject` | Bio-Evolution | Surgical file patching at line level | ✅ |
| `mutation-sentinel` | Bio-Evolution | File mutation watcher — auto-freeze on changes | ✅ |
| `telepathy-sync` | Quantum-Space | BLAKE3 delta directory synchronization | ✅ |
| `telepathy-entangle` | Quantum-Space | Inter-process shared state via temp files | ✅ |
| `eqm-pulse` | Quantum-Space | System health monitor + FFT frequency analysis | ✅ |
| `eqm-methy` | Quantum-Space | File consolidator — BLAKE3 integrity index with methylation rate | ✅ |
| `aether-excite` | Quantum-Space | Resource excitation monitor — per-region system load | ✅ |
| `aether-fabric` | Quantum-Space | System topology mapper — process/port/connection graph | ✅ |
| `borg-cube` | Machine-Brain | Parallel command replicator — exponential scaling | ✅ |
| `brain-synapse` | Machine-Brain | File co-change tracker — Hebbian git analysis | ✅ |
| `brain-connectome` | Machine-Brain | Dependency graph builder — static import analysis | ✅ |
| `collective-sync` | Machine-Brain | Experimental consensus client; successful multi-peer reconciliation is not yet verified | ⚠️ |
| `nexus-logic` | Machine-Brain | Knowledge indexer — local full-text trigram search engine | ✅ |
| `ribosome-synth` | Machine-Brain | Deterministic Rust generator + compiled artifact verification + bounded local copies | ✅ |
| `microscope-mem` | Machine-Brain | Compatibility CLI surface; persistent Microscope delegation is not connected yet | ⚠️ |
| `vagus-nerve` | Machine-Brain | Internal organ sensing — CPU/RAM → WaveField | ✅ |
| `wave-encoder` | Resonance | Data→wave encoder — FFT-based file encoding (432 Hz base) | ✅ |
| `wave-sculptor` | Resonance | Frequency filter — digital signal processing on wave packets | ✅ |
| `wave-field` | Resonance | Self-organizing interference field with bounded durable event sidecar | ✅ |
| `iron-resonate` | Resonance | Hardware resonance monitor — HW performance profiler | ✅ |
| `path-resonance` | Resonance | Hot-path detector — filesystem activity heatmap | ✅ |
| `grid-warp` | Resonance | Symlink/junction manager + latency measurement | ✅ |
| `magneto-geo` | Resonance | Error hotspot detector — code quality heatmap scanner | ✅ |
| `magneto-acoustic` | Resonance | Code health sonifier — error patterns to audio (WAV) | ✅ |
| `mycelium-spread` | Resonance | Recursive filesystem mapper — network graph builder | ✅ |
| `homeostasis` | Resonance | System equilibrium maintenance | ✅ |
| `wave-cryo-tx` | Audio | Acoustic CryoFrame transmitter — BFSK modulated WAV output | ✅ |
| `wave-cryo-rx` | Audio | Verified BFSK WAV-file decoder; live audio-device capture is not implemented | ⚠️ |

---

## Protocol

**v2 BioMessage** (primary binary wire format)
- Header: 60 bytes (BLAKE3 auth, nonce replay protection)
- Payload: TLV-style field encoding
- 22 opcodes: JOIN, TASK, RESULT, HEARTBEAT, CLONE, GENOME, APOPTOSIS, FREEZE, THAW, CRISPR_PATCH, IMMUNE_ALERT, HOMEO_SYNC, MICRO_QUERY, and more
- Drones use `bio_client::DroneClient` over UDP
- Concurrent, with an in-process `NonceWindow` for replay detection

**v1 Echo-X** (Legacy — deprecated)

---

## Binary-first local storage

| Module | Format | Purpose |
|--------|--------|---------|
| wave_store | Custom TLV | Wave packet archive |
| cryo | bincode + zlib | Cryogenic snapshots (FFT spectral state) |
| microscope_mem | compatibility surface | Persistent Microscope delegation is not connected yet |
| wave_field events | versioned bincode sidecar | Newest 1,000 emergent events, 8 MiB load cap |
| eqm_methy | bincode | Methylation state |
| telepathy_entangle | bincode | Entanglement records |

Core wave/cryo/event state uses binary formats. The omega registry and some external/legacy bridges use JSON or text.

---

## Build & Run

```bash
# Dev build (fast check)
cargo check

# Release build
cargo build --release

# Test single drone
./target/release/omega-master status

# Queen server (orchestrator)
./target/release/omega-master start --port 8888

# List all available binaries
ls target/release/*.exe | grep -v deps | sed 's|target/release/||;s|\.exe$||'
```

---

## Key Components

| Module | Description |
|--------|-------------|
| `bio_protocol.rs` | Binary protocol: encode/decode, TLV fields, checksum, replay window |
| `cryo.rs` | FFT-based spectral snapshots (CryoFrame), freeze/thaw engine |
| `acoustic.rs` | BFSK modem: Goertzel demodulation, hand-written WAV I/O |
| `auth.rs` | Keyed BLAKE3 message authentication, tokens, local consistency sidecars |
| `bio_client.rs` | UDP DroneClient: JOIN, heartbeat, result send |
| `system.rs` | sysinfo-based system snapshot (CPU, RAM, disk, processes) |
| `omega_master.rs` | Queen orchestrator: drone registry, task dispatch, CLI |
| `mitosis.rs` | Safe template rendering, rustc staging, hashing, bounded replication |
| `wave_field.rs` | Interference rules and durable emergent-event persistence |

Project shape: 33 native binary targets plus their shared Rust library modules.

---

## Dependencies (Minimal, Focused)

```toml
sysinfo = "0.30"           # System info
blake3 = "1.5"             # BLAKE3 hashing
clap = "4"                 # CLI parsing
serde = "1.0"              # Serialization
bincode = "1.3"            # Binary encoding
serde_json = "1.0"         # JSON (external APIs only)
tokio = "1"                # Async runtime
colored = "2"              # Terminal colors
memmap2 = "0.9"            # Memory mapping
chrono = "0.4"             # Timestamps
notify = "6"               # File watching
rustfft = "6"              # FFT spectral analysis
flate2 = "1.0"             # Zlib compression
qrcode = "0.14"            # QR code generation
reqwest = "0.13.2"         # HTTP (external API bridge)
```

---

## Project Structure

```
src/
├── lib.rs                  # Module declarations
├── commands/               # 33 command modules
│   ├── mod.rs
│   ├── omega_master.rs     # Queen orchestrator
│   ├── viral_infect.rs     # Code transformer
│   ├── telepathy_sync.rs   # Delta sync
│   ├── ...                 # 30 more command modules
├── bin/                    # 33 binary entry points
│   ├── omega_master.rs
│   ├── viral_infect.rs
│   └── ...                 # 31 more entry points
├── bio_protocol.rs         # v2 BioMessage format
├── bio_client.rs           # Drone client protocol
├── auth.rs                 # BLAKE3 authentication
├── cryo.rs                 # Cryogenic snapshots (FFT)
├── acoustic.rs             # BFSK modem
├── wave_store.rs           # Wave packet storage
├── system.rs               # System monitoring
├── wave_field.rs           # Wave field simulation
├── magneto.rs              # Magnetic field utilities
├── mitosis.rs              # Binary replication
├── leash.rs                # Metabolic token management
└── ...

Cargo.toml                  # Dependencies & metadata
Cargo.lock                  # Locked versions
```

---

## Security mechanisms and limits

1. **Local consistency sidecar** — BLAKE3 detects changes relative to a writable local baseline. This is not tamper-proofing and does not replace signed releases.
2. **Message authentication** — BioMessage supports keyed-BLAKE3 tags. JOIN admission and session-token enforcement are incomplete; do not expose `omega-master` to untrusted networks.
3. **Replay detection** — a rolling in-process nonce window rejects recently observed nonces while the process is running.
4. **Checksums** — CRC64 detects corrupted BioMessage bytes; it is not an authentication mechanism.
5. **Bounded replication benchmark** — `borg-cube --max-power` is hard-capped at 4, for at most 16 concurrent instances in one stage.
6. **Acoustic error detection** — CRC-16 CCITT detects corrupted acoustic frames. Forward error correction is not implemented.
7. **Constant-time tag comparison** — authentication-tag comparison avoids simple timing leakage; this does not complete network admission security.

---

## Author

**Máté Róbert (Silent)** — bio-binaries orchestration system

A research project exploring bio-inspired computing principles through a Rust ecosystem. The names are metaphors for ordinary algorithms: propagation, synchronization, association, signal processing, and homeostatic control.

Research-grade, usable software with verified local functionality. It has not undergone an independent security audit or production deployment qualification.

## Octopus integration boundary

Bio-Binaries is a separately built native Rust subsystem. Octopus does not merge its implementation into the runtime core: installed executables are invoked through the Octopus process, policy, integrity, and snapshot boundary. Direct CLI execution is intended for development; integrated operation goes through `octopus-runtime bio`.

---

## Quick Start

```bash
# Check compilation
cargo check

# Run the Queen server
cargo run --release --bin omega-master -- status

# Try code transformation (dry run)
cargo run --release --bin viral-infect -- . --pattern "TODO" --replace "DONE" --dry-run

# Real CryoFrame -> BFSK WAV -> CryoFrame self-test
cargo run --release --bin wave-cryo-tx -- test

# Plan a contained source+binary generation without writing
cargo run --release --bin ribosome-synth -- generate --name demo_drone --output-root ./generated

# Apply the verified generation (the output directory must already exist)
cargo run --release --bin ribosome-synth -- generate --name demo_drone --output-root ./generated --apply

# Read durable WaveField events
cargo run --release --bin wave-field -- events --limit 10
```

**Verified snapshot**: 62 Rust tests passed; the release build passed; all 33 installed command surfaces passed functional smoke; 7/7 artifact checks passed. Ribosome generation/replication, Cryo codec I/O, acoustic file roundtrip, and WaveField event persistence are verified. Persistent `microscope-mem` delegation, successful multi-peer `collective-sync`, and live audio-device RX remain explicitly incomplete.

Last verified: 2026-08-01 | `cargo test --locked -j1`: 62 tests passed | release build passed
