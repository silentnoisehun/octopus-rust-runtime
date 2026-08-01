# Bio-Binaries benchmark methodology

Status: normative benchmark protocol for the 33 separately bundled Bio-Binaries targets.

This protocol answers four different questions and keeps their results separate:

1. How fast is the native Bio executable by itself?
2. How fast is the production path through Octopus policy, integrity, audit, snapshot and process boundaries?
3. What is the measured Octopus boundary cost for the same successful work?
4. Which workloads scale when several independent Bio jobs run concurrently?

It does **not** assume that a short runtime proves algorithmic quality, that an intentional sampling delay is overhead, or that one-machine results establish a world record.

## 1. Artifacts and paths under test

Benchmark release executables only. Do not build, format, test, update hashes, scan recursively, or clean targets while a timed run is in progress.

- Direct lane: `bio-binaries/target/release/<name>.exe`
- Octopus lane: `target/release/octopus-runtime.exe bio external <name> ...`
- Installed-lane confirmation, when required: `C:/Users/mater/.agents/skills/octopus/bin/octopus-runtime.exe`
- Pin source: `bio-binaries/RELEASE_SHA256SUMS`
- Catalog and effect policy: `src/bio_system/external.rs`

Before the first warm-up, record the SHA-256 and byte size of Octopus, its core executable when present, and all 33 Bio executables. A result set is comparable only when those hashes, the fixture manifest and the benchmark harness hash match. Octopus verifies a Bio executable's SHA-256 on every production invocation; that read and hash are intentionally included in the Octopus lane.

## 2. Required environment capture

Write a human-readable `environment.txt` beside every result set. At minimum it must contain:

- UTC start/end time and local timezone;
- repository commit and dirty-worktree marker;
- Windows edition, version and build;
- CPU model, physical cores, logical processors and active clock information;
- installed and available RAM;
- storage model, bus/media type, filesystem, volume and free space for executable, fixture and result paths;
- AC/battery state and Windows power plan;
- Rust/Cargo versions and release build flags;
- PowerShell version and benchmark harness hash;
- executable hashes and sizes;
- antivirus/EDR state and exclusions, without disabling protection for the benchmark;
- `OCTOPUS_*`, `TEMP`, `TMP`, `RUSTFLAGS`, `CARGO_*` and proxy variables that can affect execution;
- baseline CPU, memory and disk utilization sampled for 30 seconds before the suite.

Run on AC power with a fixed power plan. Close interactive builds, indexers and unrelated load generators. Do not claim an idle run unless baseline CPU remains below 5% and there is no sustained fixture-volume I/O. Record exceptions instead of hiding them. Do not mix measurements from different boots, power plans, binary hashes or storage volumes in one aggregate.

## 3. Deterministic fixtures

Generate fixtures once from a recorded seed, hash every file, and preserve a manifest. Use absolute paths in all child arguments.

| Fixture | Required contents | Default measured profile |
|---|---|---|
| `tree-S` | 128 files, 4 KiB each, balanced across `src`, `tests`, `docs`, `config`; deterministic Rust/text/JSON mix and known `alpha` tokens | correctness and acoustic workloads |
| `tree-M` | 4,096 files, 16 KiB each, 64 directories, same deterministic mix | primary directory workload |
| `tree-L` | 32,768 files, 32 KiB each | optional scaling profile; never merge with `tree-M` results |
| `text-M` | one deterministic 8 MiB UTF-8 file with known line count and match positions | single-file transform/hash workloads |
| `payload-M` | one deterministic 8 MiB binary payload plus its SHA-256 | encoder and byte-throughput workloads |
| `git-M` | repository with 512 deterministic commits, fixed author/date/tree shape, no remotes | Git graph workload |
| `wave-M` | valid wave packet set generated from a pinned fixture and a pre-seeded WaveStore image | wave processing workload |
| `queen-M` | isolated queen key/registry and loopback-only responder with fixed response | control/network workload |

Each direct/Octopus pair receives byte-identical but separate fixture and state copies. Setup, cloning, hashing, cleanup and artifact verification happen outside the timed interval. A write/control sample always gets a fresh per-iteration directory; a failed sample is preserved for diagnosis. No benchmark may point at the repository, user profile, production memory, production WaveStore or a non-loopback network address.

Use separate state roots for the two lanes:

```text
<run>/state/direct/<target>/<pair>/
<run>/state/octopus/<target>/<pair>/
```

Set `TEMP` and `TMP` to the current lane state. Set `OCTOPUS_STATE_DIR`, `OCTOPUS_BIO_STATE_DIR`, `OCTOPUS_BIO_BIN_DIR` and `OCTOPUS_ALLOWED_ROOTS` explicitly for the Octopus lane. Mirror the child-visible `TEMP`, `TMP` and `BIO_INTEGRITY_DIR` layout in the direct lane. Environment setup itself remains outside the timer.

## 4. Paired direct-versus-Octopus protocol

For one target and fixture profile, one pair is:

```text
direct:  <bio-dir>/<name>.exe <exact child arguments>
octopus: octopus-runtime.exe bio external <name> [--allow-mutation] -- <same exact child arguments>
```

The argument vector must be identical; do not pass a shell-joined approximation. Write/control targets require `--allow-mutation` only on the Octopus side. The direct child working directory and the child working directory selected by Octopus must be recorded. Prefer absolute input/output paths so that working-directory differences cannot change the work.

Use a fixed, published random seed to choose `direct -> Octopus` or `Octopus -> direct` within each pair. Balance the two orders exactly, apart from one unavoidable extra sample when the count is odd. Finish a pair before starting the next, and use a 250 ms quiet interval between pairs. Never run the paired lanes concurrently; they would perturb each other.

### Warm-up and repetition counts

- Ordinary targets: 7 unrecorded pairs, then 50 recorded pairs.
- Targets whose representative path contains at least 500 ms of intentional sampling/waiting: 3 unrecorded pairs, then 20 recorded pairs. Their p95 is explicitly *exploratory*; use 50 pairs before publishing a stable p95 claim.
- Micro paths whose median is below 50 ms: 10 unrecorded pairs, then 100 recorded pairs, because scheduler quantization otherwise dominates.
- Any comparison with fewer than 20 successful pairs is a smoke result, not a benchmark.

Warm-up exercises both lanes and the exact same workload but is never mixed into measured samples. This is a process-cold, filesystem-cache-warm protocol: every invocation starts new processes, while normal OS file caching remains enabled. A genuinely cold-cache campaign requires a separate reboot-controlled run and must not be compared as if it belonged to the warm-cache sample.

### Timer and output rules

Use a monotonic high-resolution timer around process creation through process exit and complete stdout/stderr drain. Do not render per-sample output to the terminal. Capture both lanes to files on the same result volume. Include child output generation and capture in both lanes; exclude result parsing and artifact hashing.

Measure, when the harness can attribute the full process tree reliably:

- wall-clock nanoseconds;
- process-tree user and kernel CPU time;
- peak committed/working-set bytes;
- process-tree read/write bytes;
- exit code and timeout state.

If process-tree attribution is unavailable, omit those fields rather than reporting the Octopus parent as the whole tree.

## 5. Correctness gate before timing is accepted

Every recorded pair must first satisfy all applicable gates:

- both processes exit successfully and before the 30 s adapter timeout;
- Octopus output contains the expected `[bio-binaries] name=<target>` evidence line;
- output counters describe the same amount of work;
- required artifacts exist, remain inside the iteration root, and have the expected type/size/hash or parse successfully;
- dry-run workloads leave their protected target unchanged;
- direct and Octopus semantic results match after stripping the Octopus evidence line and normalizing only documented nondeterministic fields such as timestamp, absolute lane root and live system values;
- no unexpected outbound network, production-state write or child process remains.

Do not silently discard failures or outliers. Report success count and every failure classification. Compute latency distributions only over successful pairs and print the denominator. If either lane has less than 99% success, the target fails the benchmark gate even when its successful samples are fast.

## 6. Statistics and derived metrics

For each lane report `n`, median, nearest-rank p95, minimum, maximum, median absolute deviation and a 95% bootstrap confidence interval for the median. Use at least 10,000 deterministic bootstrap resamples and record the seed.

For paired sample `i`:

```text
delta_i = Octopus_i - Direct_i
ratio_i = Octopus_i / Direct_i
```

Report the median and 95% bootstrap confidence interval of `delta_i` and `ratio_i`. Do not subtract independently calculated medians and call that paired overhead. When `Direct_i` is below the timer's reliable resolution, report the absolute delta and suppress the ratio.

Where meaningful:

```text
throughput = validated_units / elapsed_seconds
Octopus overhead % = 100 * median(delta_i / Direct_i)
```

Valid units are bytes read/hashed/transformed, files visited, commits parsed, packets processed, completed child processes or verified operations. Never derive throughput from an advertised parameter when the implementation did not process that data.

## 7. Workload contract for all 33 targets

`R` is the iteration root. `F`, `S`, `T`, `G`, `W` and `Q` mean its fixture, sync source, sync target, Git, wave and queen-state directories. All arguments below are child arguments and are passed unchanged to both lanes. `dry` means the command is catalogued as write/control but its primary timed workload is deliberately non-mutating. An additional untimed/live correctness case is still required where noted.

| Target | Catalog effect | Representative safe functional workload | Primary measurement and interpretation |
|---|---:|---|---|
| `viral-infect` | write | `F --pattern alpha --replace beta --ext txt --dry-run` on `tree-M` | files and input bytes scanned/s; verify known match count and no changes. One live `tree-S` correctness case runs outside timing. |
| `hox-diff` | read | `F` on `tree-M` | files and bytes mapped/s; validate region/file totals. |
| `plasmid-dream` | control | `F --command "rustc --version" --predict 3` | end-to-end analyzer latency. Measure the same `rustc --version` child separately to expose nested-process cost. A pinned tiny crate plus `cargo check --offline --locked` is a separate integration profile, not mixed into this result. |
| `plasmid-inject` | write | `text-M --start 1000 --end 2000 --fix-file R/fix.txt --dry-run` | input bytes and lines/s; validate proposed edit and unchanged input. One live copy verifies exact splice outside timing. |
| `telepathy-sync` | write | `S T --dry-run` on `tree-M` | files and bytes compared/s; validate planned-copy count and unchanged target. A fresh live `tree-S` copy is an untimed artifact gate. |
| `telepathy-entangle` | write | `set bench-<pair> connected` with isolated `TEMP` | successful state operations/s and latency. Verify with an untimed `get`, then delete the iteration state. |
| `eqm-pulse` | read | `--duration 1 --interval 100` | raw latency and 10 validated samples/run. The approximately 1,000 ms sampling wait is intended work, not Octopus overhead. |
| `eqm-methy` | write | `F --depth 8` on a fresh `tree-M` | files and bytes BLAKE3-hashed/s; validate `.methy-index.bin`. |
| `aether-excite` | read | no arguments | raw system-snapshot latency. The source intentionally waits 300 ms before refresh; report host load and do not call that time hashing or orchestration. |
| `aether-fabric` | read | `--top 30` | process/network topology latency and observed process count. Includes an intentional 300 ms refresh wait and `netstat`; results are host-state dependent. |
| `borg-cube` | control | one argument `cmd /d /c exit 0`, then `--max-power 5` | 63 validated child completions/run, children/s and scaling by stage. Cap at power 6; never benchmark arbitrary or mutating commands. |
| `nexus-logic` | write | `F --ext txt --query alpha --limit 100` on fresh `tree-M` | files indexed/searched per second and validated hits; preserve any cache only within its iteration. |
| `collective-sync` | control | `--echo-x <loopback-responder> --topic bench --vote OK` | successful loopback request/response latency. The responder returns a fixed valid protocol message. The existing unreachable-endpoint path is a separate 2 s timeout-contract test and is not success throughput. |
| `brain-synapse` | read | `G --limit 512 --min-weight 1` on `git-M` | commits/relationships parsed per second; validate fixed commit and edge counts. |
| `brain-connectome` | read | `F --lang rust` on `tree-M` | Rust files and relationships/s; validate totals against the manifest. |
| `wave-encoder` | write | `payload-M --output R/wave.json` | input bytes/s and packets produced/s; parse output and validate its semantic payload. |
| `wave-sculptor` | write | `W/input.json --filter lowpass --cutoff 1000 --output R/sculpted.json` using pinned `wave-M` | input packets/s and input bytes/s; parse output and validate filter invariants. |
| `iron-resonate` | read | `--samples 5` | raw latency and five CPU samples/run. The 5 x 200 ms waits are intentional sampling time; host conditions dominate values. |
| `path-resonance` | read | `F --depth 8` on `tree-M` | files and bytes visited/s; validate path/count summary. |
| `grid-warp` | write | `--source F/sample.txt --target R/warp.txt --dry-run` | single planned mapping latency; do not invent byte throughput. One live isolated mapping is created, resolved and removed outside timing. |
| `magneto-geo` | read | `F --depth 8` on `tree-M` | files and bytes mapped/s; validate cluster/count summary. |
| `mycelium-spread` | read | `F --depth 8` on `tree-M` | files/directories traversed per second; validate discovered network totals. |
| `homeostasis` | control | `status` in isolated state | validated status operations/s and latency. Long-running regulation modes require a separate duration-bounded soak, not this process-exit suite. |
| `omega-master` | control | `--state-dir Q key-info`, after pre-initializing the isolated key | steady-state key/status latency. Report first-key creation separately as a cold initialization operation. Never start or signal production drones. |
| `omega-point` | read | `--duration 1 --interval 1` | raw convergence-monitor latency and measurements/run. The 100 ms CPU refresh plus interval sleep are designed sampling behavior; do not subtract them from production latency. |
| `ribosome-synth` | control | `generate --name benchmark_drone --output-root R` without `--apply` | deterministic template rendering, validation and generation-plan latency. Compilation/publication is a separate artifact profile using a fresh output root; real replication uses `replicate --count N --output-root R --apply` and validates every copy hash. |
| `wave-cryo-tx` | write | `test --duration-ms 1` | real CryoFrame capture -> BFSK WAV encode -> decode -> CRC/frame-hash verification. Report payload bytes, samples and end-to-end latency; the WAV is temporary and removed after verification. |
| `wave-cryo-rx` | read | `monitor --duration-ms 1000` for the standard process matrix; separate pinned `.cryo` -> WAV -> `.cryo` artifact profile for codec throughput | monitor latency remains timer-only because no live audio-capture backend is configured. Decode throughput claims require the real pinned roundtrip profile and validated output hash. |
| `mutation-sentinel` | control | `hash R/payload.bin` using `payload-M` | BLAKE3 input bytes/s and hash equality. Watch-mode event latency is a separate, duration-bounded test that touches only an isolated `.rs` file and terminates the watcher explicitly. |
| `magneto-acoustic` | write | `F --output R/health.wav --tone-ms 10 --depth 8` on `tree-S` | files analyzed/s and valid WAV output bytes/s; validate WAV header, duration and channel/rate fields. |
| `wave-field` | write | `snapshot` against a byte-identical pre-seeded `wave-M` store | packets merged/evaluated per second and snapshot latency; validate persisted store and energy totals. A separate restart profile triggers events, reloads the versioned sidecar, checks newest-1000 trimming and queries `events --limit N`. |
| `vagus-nerve` | write | `--snapshot` with an isolated WaveStore | raw sensor/injection latency and exactly validated emitted packets. It intentionally waits 500 ms and writes the WaveStore inbox. |
| `microscope-mem` | write | `status` | wrapper dispatch latency only. Current `store`, `recall`, `status` and `build` return formatted placeholder strings and do not exercise the persistent Microscope engine, so no memory storage/recall throughput claim is permitted. |

## 8. Intentional-delay reporting

Always report observed end-to-end latency. A second diagnostic column may show a declared nominal wait, but it must be labeled and never replace the real latency.

| Target/path | Source-declared wait in the representative workload |
|---|---:|
| `eqm-pulse --duration 1 --interval 100` | 10 x 100 ms |
| `aether-excite` | 300 ms |
| `aether-fabric` | 300 ms |
| `iron-resonate --samples 5` | 5 x 200 ms |
| `omega-point --duration 1 --interval 1` | 100 ms refresh per point plus interval behavior |
| `ribosome-synth generate` plan | no declared sleep; render/validation only |
| `wave-cryo-tx test --duration-ms N` | N ms spectral capture plus a 200 ms system refresh and real codec work |
| `wave-cryo-rx monitor --duration-ms 1000` | 500 ms poll sleeps until duration is reached |
| `vagus-nerve --snapshot` | 500 ms |

`collective-sync` has a 2 s receive timeout. That duration belongs only to the negative timeout-contract profile. A successful benchmark requires a loopback response and must not wait for timeout.

## 9. Parallel scaling

Run scaling only after the single-job correctness and paired-latency gates pass.

Use concurrency `K = 1, 2, 4, ...` up to the smaller of 16 and the logical processor count. For each `K`, prepare `K` independent fixture/state copies, perform 2 warm-up windows and 15 recorded windows, and start jobs behind one barrier. A window ends when all jobs exit and their artifacts validate.

Report:

```text
window throughput(K) = total validated units / window seconds
speedup(K) = throughput(K) / throughput(1)
efficiency(K) = speedup(K) / K
```

Run two distinct scaling surfaces:

1. Direct `K`-process baseline versus `K` simultaneous `bio external` Octopus invocations. This measures aggregate production-boundary capacity.
2. Native Octopus pipeline scaling for read-only targets whose exact arguments can be represented identically in independent arms. This measures the head/arm scheduler as a system, not just several wrapper processes.

Good scaling candidates are `hox-diff`, `viral-infect --dry-run`, `brain-connectome`, `path-resonance`, `magneto-geo`, `mycelium-spread`, `mutation-sentinel hash`, `wave-encoder` and `wave-sculptor`, each on independent inputs. Do not use host sensors, loopback consensus, shared WaveStore/state, `borg-cube`, watchers, monitors or long-running controls for general parallel scaling. They measure shared-resource contention or nested fan-out, not independent module scaling.

Record CPU utilization, memory pressure and fixture-volume throughput for each window. Stop increasing `K` after two consecutive points reduce throughput or any point exceeds 90% memory commitment, begins paging, fails correctness, or triggers thermal throttling. Do not present oversubscribed degradation as an Octopus failure without the resource evidence.

## 10. Result presentation

Produce one row per target and workload profile with:

- artifact and fixture IDs;
- catalog/effective effect;
- successful pairs / attempted pairs;
- direct median and p95;
- Octopus median and p95;
- paired delta and ratio with confidence intervals;
- primary unit count and lane throughput;
- intentional wait, when present;
- correctness evidence;
- warnings and exclusions.

Keep raw sample data in CSV or TSV and the summary human-readable. Never aggregate the 33 target latencies into one “average Bio speed”: they perform different work and several contain deliberate waits. A suite-level result may state pass count and total wall time, but performance conclusions remain per target and workload.

## 11. Claim limits

The strongest valid conclusion from this protocol is of the form:

> On the recorded machine, binary hashes and fixture profile, target X completed Y validated units at the reported median/p95; the production Octopus path added the reported paired cost, and concurrency scaled to K with the reported efficiency.

The protocol does not justify any of these statements by itself:

- “fastest in the world” or faster than an unmeasured competitor;
- production performance on other hardware, storage, power plans or security software;
- algorithmic correctness from exit code alone;
- code quality, evolutionary quality or “code breeding” effectiveness from latency;
- BFSK encoding/decoding throughput from the current Cryo CLI surfaces;
- Microscope memory throughput from the current wrapper;
- p99 latency from 20–100 samples;
- cold-cache performance from this cache-warm protocol.

A comparison against another tool requires a separate pre-registered study with the same input, semantic output, safety constraints, cache condition and success gate. Results that violate this document remain diagnostics or demonstrations, not benchmark evidence.

## 12. Known pre-benchmark issues from source audit

These are not reasons to hide targets; they define what can honestly be measured today:

1. `vagus-nerve` was catalogued `read`, but `--snapshot` injects packets into the WaveStore inbox. The Octopus catalog now classifies it as `write`; benchmark state remains isolated.
2. Bio v0.3.0 connects real Wave-Cryo encode/decode and verifies a command-level `.cryo` -> WAV -> `.cryo` roundtrip. The standard 33-target latency row still uses TX self-test and RX monitor; only the separate pinned artifact profile supports codec-throughput claims.
3. `microscope-mem` remains a formatted compatibility wrapper, not a call into persistent Microscope storage.
4. Bio v0.3.0 connects `ribosome-synth generate` and bounded local `replicate`; benchmark reports must distinguish plan-only render latency from applied rustc/artifact latency.
5. The 33-target functional smoke still uses an unreachable `collective-sync` endpoint. That proves bounded negative-path behavior, not successful multi-peer consensus throughput.
6. Bio v0.3.0 persists WaveField emergent events in a bounded versioned sidecar; concurrent writers retain last-atomic-writer-wins semantics and are not a transactional multi-writer benchmark.

Every published benchmark report must identify its Bio release and executable hashes. Pre-v0.3.0 rows remain historical evidence and must not be relabeled as current generation, codec or event-persistence performance.
