# Bio-Binaries paired latency benchmark — 2026-08-01

> Historical evidence notice: this run predates Bio-Binaries v0.3.0. Its process-boundary and scaling measurements remain valid for the recorded executable hashes, but the `ribosome-synth`, `wave-cryo-tx`, `wave-cryo-rx`, and `wave-field` workload semantics must not be projected onto the v0.3.0 implementation. Rerun the harness for current per-module latency.

## Verdict

The measured architecture hypothesis is supported on this machine:

1. Native Bio processes have low startup-to-exit latency on the small isolated functional fixtures: 22 of 33 module medians are below 50 ms and 25 of 33 are below 100 ms.
2. The full Octopus production boundary has a bounded absolute cost: the median of the 33 per-module paired overhead medians is 24.217 ms; 27 of 33 are below 30 ms.
3. Independent Octopus Bio work scales: throughput rose from 16.998 jobs/s at concurrency 1 to 40.930 jobs/s at concurrency 8, a 2.408x speedup and 140.8% throughput increase.

This does not prove a world record, that process separation is faster than a hypothetical in-process port, large-fixture algorithm throughput, or code-breeding quality. It proves that the separate native subsystem is fast in absolute end-to-end terms, the safety boundary cost is measurable rather than dominant for substantial work, and independent work gains real throughput from concurrency.

## Protocol and evidence

- Machine: AMD Ryzen 5 7535HS, 6 cores / 12 logical processors, 15.245 GiB RAM, Windows 11 build 26200.
- Runtime SHA-256: `CBE3DD06BFB4C0597B8822188F4CABD87132D3B0101224D44ADA68EF117D4A49`.
- Protocol: 3 warm-up pairs plus 20 measured direct/Octopus pairs per module, alternating order.
- Measured pairs: 660/660 successful.
- Measured process executions: 1320/1320 successful.
- Parallel jobs: 48/48 successful.
- Main 33-module matrix duration: 558.87 seconds.
- Measurement scope: cache-warm, process-cold, small isolated functional fixtures.
- Nearest-rank p95 values are exploratory at 20 pairs; medians and paired deltas are primary.

Raw evidence:

- `D:\codex\.octopus-rust\bio-benchmarks\20260801-165403-029\bio-benchmark-samples.csv`
- `D:\codex\.octopus-rust\bio-benchmarks\20260801-165403-029\bio-benchmark-summary.csv`
- `D:\codex\.octopus-rust\bio-benchmarks\20260801-165403-029\bio-benchmark-parallel.csv`
- `D:\codex\.octopus-rust\bio-benchmarks\20260801-165403-029\bio-benchmark-environment.txt`
- `D:\codex\.octopus-rust\bio-benchmarks\20260801-165403-029\bio-benchmark-report.md`
- `D:\codex\.octopus-rust\bio-benchmarks\20260801-165403-029\SHA256SUMS`

## All 33 module medians

`Delta` and `ratio` are computed from matched direct/Octopus pairs, not by subtracting unrelated distributions.

| Module | Effect | Direct median (ms) | Octopus median (ms) | Paired delta (ms) | Paired ratio |
|---|---:|---:|---:|---:|---:|
| viral-infect | write | 28.187 | 52.360 | 24.599 | 1.853x |
| hox-diff | read | 25.940 | 49.194 | 23.171 | 1.898x |
| plasmid-dream | control | 50.650 | 74.848 | 25.348 | 1.504x |
| plasmid-inject | write | 26.270 | 50.037 | 24.076 | 1.923x |
| telepathy-sync | write | 26.210 | 49.970 | 23.509 | 1.894x |
| telepathy-entangle | write | 25.585 | 49.471 | 24.367 | 1.946x |
| eqm-pulse | read | 5395.308 | 5463.218 | 73.039 | 1.014x |
| eqm-methy | write | 28.596 | 51.966 | 23.548 | 1.815x |
| aether-excite | read | 656.596 | 725.668 | 71.158 | 1.108x |
| aether-fabric | read | 701.118 | 770.330 | 69.830 | 1.100x |
| borg-cube | control | 52.552 | 76.667 | 24.232 | 1.456x |
| nexus-logic | write | 26.382 | 49.961 | 23.712 | 1.910x |
| collective-sync | control | 24.126 | 47.928 | 24.255 | 2.015x |
| brain-synapse | read | 57.564 | 81.318 | 24.084 | 1.421x |
| brain-connectome | read | 29.056 | 53.728 | 25.097 | 1.882x |
| wave-encoder | write | 27.538 | 51.646 | 24.217 | 1.892x |
| wave-sculptor | write | 35.263 | 58.394 | 22.948 | 1.653x |
| iron-resonate | read | 556.654 | 631.862 | 75.357 | 1.135x |
| path-resonance | read | 25.406 | 48.815 | 23.900 | 1.951x |
| grid-warp | write | 25.946 | 50.285 | 24.404 | 1.939x |
| magneto-geo | read | 30.272 | 56.009 | 26.000 | 1.854x |
| mycelium-spread | read | 26.066 | 49.820 | 23.502 | 1.907x |
| homeostasis | control | 20.610 | 47.276 | 26.344 | 2.253x |
| omega-master | control | 23.690 | 48.171 | 23.889 | 2.016x |
| omega-point | read | 1456.184 | 1524.674 | 71.194 | 1.049x |
| ribosome-synth | control | 21.580 | 44.913 | 23.446 | 2.072x |
| wave-cryo-tx | write | 123.122 | 145.767 | 22.720 | 1.184x |
| wave-cryo-rx | read | 522.838 | 545.775 | 22.889 | 1.044x |
| mutation-sentinel | control | 25.930 | 49.778 | 23.780 | 1.920x |
| magneto-acoustic | write | 31.019 | 55.558 | 24.322 | 1.781x |
| wave-field | write | 20.326 | 44.146 | 23.810 | 2.172x |
| vagus-nerve | write | 852.385 | 929.238 | 72.839 | 1.085x |
| microscope-mem | write | 21.910 | 44.907 | 23.268 | 2.065x |

The median of module medians is 28.187 ms direct and 51.966 ms through Octopus. This is an integration-tax summary, not an average of equivalent workloads. For the 25 modules below 100 ms direct, the typical paired cost is 24.076 ms and the typical ratio is 1.907x. For the eight longer modules, the typical ratio drops to 1.092x because useful or intentional module work dominates the boundary.

## Parallel scaling

Twelve isolated read jobs were executed at each concurrency level using `hox-diff`, `brain-connectome`, `path-resonance`, and `mycelium-spread`.

| Concurrency | Passed | Makespan (ms) | Throughput (jobs/s) | Speedup | Efficiency |
|---:|---:|---:|---:|---:|---:|
| 1 | 12/12 | 705.953 | 16.998 | 1.000x | 100.0% |
| 2 | 12/12 | 466.851 | 25.704 | 1.512x | 75.6% |
| 4 | 12/12 | 345.358 | 34.747 | 2.044x | 51.1% |
| 8 | 12/12 | 293.181 | 40.930 | 2.408x | 30.1% |

Scaling is real but sublinear. On this 6-core/12-thread machine, the strongest efficiency is at concurrency 2, while absolute throughput continues to increase through 8.

## Functional limits discovered by the benchmark audit

The 33/33 result means the selected representative workload completed on the recorded pre-v0.3.0 binaries; it does not mean every subcommand was fully implemented at that time.

- `collective-sync` measured the bounded unreachable-loopback path, not successful consensus throughput.
- At measurement time, `wave-cryo-tx encode` and `wave-cryo-rx decode` reported surfaces without real codec I/O; Bio v0.3.0 subsequently connected and roundtrip-tested those paths.
- `microscope-mem` is currently a compatibility wrapper, not persistent Microscope storage.
- At measurement time, `ribosome-synth generate` was not implemented and the row measured template listing; Bio v0.3.0 subsequently added deterministic plan/apply generation and verified local replication.
- At measurement time, `wave-field events` was not persisted; Bio v0.3.0 subsequently added a bounded versioned event sidecar and restart tests.
- `vagus-nerve` writes WaveStore data. The audit corrected its Octopus effect from read to write before the final benchmark.

These were implementation findings, not timing failures. Fixed v0.3.0 surfaces require a new benchmark run and new executable hashes before any current-performance claim.

## Claim boundary

The evidence supports: low native process latency, a bounded Octopus production-boundary cost, successful effect isolation, and real concurrent throughput growth on the recorded machine.

It does not yet support: fastest-in-the-world claims, cross-hardware generalization, a causal comparison against an in-process port, production-scale throughput, or the quality/effectiveness of future code-breeding workflows.
