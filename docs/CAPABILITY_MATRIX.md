# Octopus Capability Matrix — v2.9.0 verified state

Total public capabilities: 225 (the existing 192-entry Octopus surface plus 33 separately bundled Bio-Binaries process targets)

Canonical runtime axes:

- **status:** `real`, `unavailable`, `unsupported`, or `deprecated`;
- **execution class:** `advisory`, `local-operation`, `external-integration`, or `control-plane`;
- **verification grade:** `declared`, `tested`, or `observed`.

`real` means a route exists. It does not by itself claim an external side effect. `windows-offline` requires `real`, rejects external integrations, and requires at least `tested`.

## Current verified state

- Registry: 225 unique entries
- Status: 168 `real`, 55 `unavailable`, 2 `unsupported`
- Windows/offline profile: 164 entries, no external integration, no `declared` route
- Tests: 357 Octopus tests plus 62 Bio-Binaries tests, 0 failed
- Native functional smoke: 33/33 targets plus 7/7 artifact checks
- Current v0.3 diagnostic pilot: 33/33 direct and Octopus module cases with three measured samples per lane; concurrency disabled
- Historical pre-v0.3 latency: 660/660 direct/Octopus pairs; typical paired boundary cost 24.217 ms for the recorded executables
- Historical pre-v0.3 scaling: 48/48 jobs; 16.998 to 40.930 jobs/s from concurrency 1 to 8 (2.408x)
- Integrity: 33/33 release executables SHA-256 pinned and checked before process launch
- Clippy: clean with `-D warnings`
- Release hashes belong in the checksum file shipped with each future GitHub Release; Git does not contain generated executables.

## Bundled native Bio subsystem

These capabilities remain executables in the independent `bio-binaries` Cargo crate. Octopus supplies policy, exact argument forwarding, time/output limits, private runtime state, audit snapshots and executable integrity validation.

| Effect | Count | Targets |
|---|---:|---|
| read | 12 | hox-diff, eqm-pulse, aether-excite, aether-fabric, brain-synapse, brain-connectome, iron-resonate, path-resonance, magneto-geo, mycelium-spread, omega-point, wave-cryo-rx |
| write | 13 | viral-infect, plasmid-inject, telepathy-sync, telepathy-entangle, eqm-methy, nexus-logic, wave-encoder, wave-sculptor, grid-warp, magneto-acoustic, wave-field, vagus-nerve, microscope-mem |
| control | 8 | plasmid-dream, borg-cube, collective-sync, homeostasis, omega-master, ribosome-synth, wave-cryo-tx, mutation-sentinel |

Bio v0.3.0 qualifies `ribosome-synth generate/replicate`, Wave-Cryo encode/decode/self-test, and `wave-field events` with real artifact or persistence tests. `microscope-mem` remains a compatibility-only wrapper and the benchmarked `collective-sync` case remains a negative/unreachable-endpoint path; neither is represented as a completed persistent-memory or successful-consensus implementation.

Legacy v2.5 row status key (non-canonical):
- **real** — adapter performs a real action, verified by test
- **unavailable** — adapter is implemented, probe returns typed unavailable/auth_required
- **unsupported** — capability does not apply to this environment
- **deprecated** — retained for backward compatibility, will be removed

## Legacy v2.5 status summary (non-canonical)

- **real**: 9 (code-reader, code-writer, diagnostics, git-nexus, github, github-manager, pipeline-architect, rust-surgeon, summarize, sag)
- **unavailable**: All LocalProcess and ExternalRead/ExternalWrite blades (no CLI tools or credentials)
- **unsupported**: apple-notes, bear-notes (macOS only)

## Legacy v2.5 verification state

- Tests: 272 (253 unit + 19 integration)
- Clippy: clean (strict -D warnings)
- Build: clean
- All unavailable/unsupported blades return typed failures, not Completed-wrapped strings
- Snapshots: Result-based API with tested typed failure paths

## Legacy detailed contract inventory

The rows below retain historical per-blade contract fields and status spelling. They are not the current routing source of truth. Use `octopus-runtime capabilities` or `capabilities --profile windows-offline` for canonical classification.

| # | Name | Effect | Adapter | Legacy status | Input | Side effect | Tool | Credential | Timeout ms | Limit | Typed failures | Tested | Owner |
|---:|---|---|---|---|---|---|---|---|---:|---|---|---|---|
| 2 | code-writer | local-write | adapter | real | path\|expected_sha256_or_NEW\|content | file write/backup | none | none | 10000 | 1 MiB | invalid_write_contract, stale_write, new_file_requires_new, file_too_large, file_read_failed, path_denied, temporary_create_failed, temporary_write_failed, backup_failed, write_commit_failed | yes | capability.rs |
| 3 | summarize | pure-algorithm | real-algorithm | real-algorithm | text to summarize | none | none | none | 10000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch1.rs |
| 4 | web-research | external-read | real-algorithm | unavailable | query text | network | curl/wget | none | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch1.rs |
| 5 | sag | pure-algorithm | real-algorithm | real-algorithm | text | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch1.rs |
| 6 | code-analysis | pure-algorithm | real-algorithm | real-algorithm | code text | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch1.rs |
| 7 | diagnostics | local-read | adapter | real | file path (no newlines) | none | none | none | 5000 | 1 MiB | path_not_found, file_metadata_failed, file_too_large, file_read_failed, path_denied | yes | capability.rs |
| 8 | audio-diagnostics | external-read | real-algorithm | unavailable | audio file path | file read + network | ffmpeg, whisper | none | 30000 | 1 MiB | blade_unavailable, blade_panicked | no | batch1.rs |
| 9 | openai-image-gen | external-write | real-algorithm | unavailable | prompt text | network + file write | curl | OPENAI_API_KEY | 30000 | 1 MiB | blade_unavailable, blade_panicked | no | batch1.rs |
| 10 | openai-whisper | external-read | real-algorithm | unavailable | audio file path | network | curl | OPENAI_API_KEY | 60000 | 1 MiB | blade_unavailable, blade_panicked | no | batch1.rs |
| 11 | sherpa-onnx-tts | local-process | real-algorithm | unavailable | text | file write | sherpa-onnx-tts | none | 30000 | 1 MiB | blade_unavailable, blade_panicked | no | batch1.rs |
| 12 | tts-voice | local-process | real-algorithm | unavailable | text | file write | sherpa-onnx-tts | none | 30000 | 1 MiB | blade_unavailable, blade_panicked | no | batch1.rs |
| 13 | stt-ear | external-read | real-algorithm | unavailable | audio stream | network | whisper | none | 30000 | 1 MiB | blade_unavailable, blade_panicked | no | batch1.rs |
| 14 | mermaid-agent | pure-algorithm | real-algorithm | real-algorithm | text | none | none | none | 10000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch1.rs |
| 15 | github | external-read | adapter | real | gh CLI args | network | gh CLI | GITHUB_TOKEN | 15000 | 1 MiB | blade_unavailable, blade_panicked | yes | capability.rs |
| 16 | github-manager | external-read | adapter | real | gh CLI args | network | gh CLI | GITHUB_TOKEN | 15000 | 1 MiB | blade_unavailable, blade_panicked | yes | capability.rs |
| 17 | git-nexus | local-read | adapter | real | directory path | none | git | none | 15000 | 1 MiB | path_denied, not_a_directory, git_unavailable, git_status_failed, current_dir_failed | yes | capability.rs |
| 18 | notion | external-read | real-algorithm | unavailable | API query | network | curl | NOTION_API_KEY | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch1.rs |
| 19 | discord | external-read | real-algorithm | unavailable | message query | network | curl | DISCORD_TOKEN | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch1.rs |
| 20 | himalaya | external-read | real-algorithm | unavailable | email query | network | himalaya CLI | email credentials | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch1.rs |
| 21 | 1password | external-read | real-algorithm | unavailable | item query | network | op CLI | 1password credentials | 10000 | 1 MiB | blade_unavailable, blade_panicked | no | batch1.rs |
| 22 | canvas | pure-algorithm | real-algorithm | real-algorithm | HTML content | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 23 | canvas-design | pure-algorithm | real-algorithm | real-algorithm | design spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 24 | frontend-design | pure-algorithm | real-algorithm | real-algorithm | design spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 25 | ui-design-system | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 26 | ui-ux-pro | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 27 | theme-factory | pure-algorithm | real-algorithm | real-algorithm | theme spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 28 | brand-guidelines | pure-algorithm | real-algorithm | real-algorithm | brand spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 29 | brand-voice | pure-algorithm | real-algorithm | real-algorithm | voice spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 30 | brand-writer | pure-algorithm | real-algorithm | real-algorithm | writer spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 31 | prose | pure-algorithm | real-algorithm | real-algorithm | prose spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 32 | writing-rules | pure-algorithm | real-algorithm | real-algorithm | rules spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 33 | doc-scribe | pure-algorithm | real-algorithm | real-algorithm | doc spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 34 | document-agent | pure-algorithm | real-algorithm | real-algorithm | doc spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 35 | agent-development | pure-algorithm | real-algorithm | real-algorithm | agent spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 36 | hook-development | pure-algorithm | real-algorithm | real-algorithm | hook spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 37 | plugin-structure | pure-algorithm | real-algorithm | real-algorithm | plugin spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 38 | command-development | pure-algorithm | real-algorithm | real-algorithm | command spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 39 | testing-codegen | pure-algorithm | real-algorithm | real-algorithm | test spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 40 | test-tui | local-process | real-algorithm | unavailable | test command | process run | cargo/bun/npm | none | 60000 | 1 MiB | blade_unavailable, blade_panicked | no | batch2.rs |
| 41 | memory-skills | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 42 | memory-skills-v2 | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 43 | microscope-memory | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 44 | emoti-mem | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 45 | claude-logic | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 46 | claude-psi | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 47 | psi-logic | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 48 | psi-quantum | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 49 | psi | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 50 | architect-mind | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 51 | senior-architect | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 52 | senior-prompt-engineer | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 53 | planner | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 54 | memory-bank | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 55 | rust-surgeon | composite | adapter | real | rust_file\|exact_boundary\|replacement | file write (transactional) | none | none | 10000 | 1 MiB | architect_refused, surgeon_refused | yes | capability.rs + composite.rs |
| 56 | omni-surgeon | pure-algorithm | real-algorithm | real-algorithm | AST spec | none | none | none | 10000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 57 | file-surgeon | pure-algorithm | real-algorithm | real-algorithm | file spec | none | none | none | 10000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 58 | formatter | pure-algorithm | real-algorithm | real-algorithm | code to format | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 59 | stem-core | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 60 | omni-connector | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch3.rs |
| 61 | mintlify | pure-algorithm | real-algorithm | real-algorithm | doc spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch2.rs |
| 62 | parser | pure-algorithm | real-algorithm | real-algorithm | code to parse | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 63 | type-inference | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 64 | lint-rules | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 65 | crispr-hotfix | pure-algorithm | real-algorithm | real-algorithm | code spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 66 | crispr-hotfix-v2 | pure-algorithm | real-algorithm | real-algorithm | code spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 67 | synaptic-pruning | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 68 | synaptic-pruning-v2 | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 69 | viral-transduction | pure-algorithm | real-algorithm | real-algorithm | code spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 70 | hox-architecture | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 71 | ai-synapse | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 72 | hive-orchestrator | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 73 | maestro | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 74 | swarm | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 75 | colony-swarm | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 76 | quality-bun | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 77 | react-practices | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 78 | stem-cell-manager | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 79 | mitosis-agent | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 80 | blogwatcher | pure-algorithm | real-algorithm | real-algorithm | url | none | none | none | 10000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 81 | peekaboo | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch4.rs |
| 82 | merge-pr | pure-algorithm | real-algorithm | real-algorithm | PR spec | none | gh CLI | GITHUB_TOKEN | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 83 | merge-pr-v1 | pure-algorithm | real-algorithm | real-algorithm | PR spec | none | gh CLI | GITHUB_TOKEN | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 84 | review-pr | pure-algorithm | real-algorithm | real-algorithm | PR spec | none | gh CLI | GITHUB_TOKEN | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 85 | still-archive | external-read | real-algorithm | unavailable | query | network | none | none | 10000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 86 | eightctl | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch5.rs |
| 87 | clawhub | external-read | real-algorithm | unavailable | query | network | clawhub CLI | none | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 88 | wacli | external-read | real-algorithm | unavailable | query | network | wacli CLI | whatsapp credentials | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 89 | goplaces | external-read | real-algorithm | unavailable | query | network | curl | GOOGLE_API_KEY | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 90 | local-places | external-read | real-algorithm | unavailable | query | network | curl | none | 10000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 91 | weather | external-read | real-algorithm | unavailable | location query | network | curl | none | 10000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 92 | web-extractor | external-read | real-algorithm | unavailable | URL | network | curl | none | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 93 | lobster-scraper | external-read | real-algorithm | unavailable | URL | network | curl | none | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 94 | nano-pdf | local-process | real-algorithm | unavailable | PDF instructions | file write | nano-pdf CLI | none | 30000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 95 | pptx | local-process | real-algorithm | unavailable | PPTX instructions | file write | pptx CLI | none | 30000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 96 | gog | external-read | real-algorithm | unavailable | query | network | gog CLI | GOOGLE_CREDENTIALS | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 97 | tmux | local-process | real-algorithm | unavailable | command | process run | tmux | none | 10000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 98 | turborepo | local-process | real-algorithm | unavailable | command | process run | turbo | none | 30000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 99 | brainstorming | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch5.rs |
| 100 | voice-call | external-write | real-algorithm | unavailable | call spec | network | none | VOICE_API_KEY | 30000 | 1 MiB | blade_unavailable, blade_panicked | no | batch5.rs |
| 101 | incubator | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch5.rs |
| 102 | video-frames | local-process | real-algorithm | unavailable | video file | file write | ffmpeg | none | 30000 | 1 MiB | blade_unavailable, blade_panicked | no | batch6.rs |
| 103 | bench-meter | pure-algorithm | real-algorithm | real-algorithm | benchmark spec | none | none | none | 10000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch6.rs |
| 104 | forge-blade | pure-algorithm | real-algorithm | real-algorithm | blade spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch6.rs |
| 105 | mcporter | external-read | real-algorithm | unavailable | MCP query | network | mcporter CLI | none | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch6.rs |
| 106 | apple-notes | external-read | real-algorithm | unavailable | query | network | none | APPLE_ID | 10000 | 1 MiB | blade_unavailable, blade_panicked | unsupported | batch6.rs |
| 107 | bear-notes | external-read | real-algorithm | unavailable | query | network | none | none | 10000 | 1 MiB | blade_unavailable, blade_panicked | unsupported | batch6.rs |
| 108 | hello-mate | pure-algorithm | real-algorithm | real-algorithm | message | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch6.rs |
| 109 | omega-striker | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch6.rs |
| 110 | sigma | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch6.rs |
| 111 | data-master | pure-algorithm | real-algorithm | real-algorithm | data spec | none | none | none | 10000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch6.rs |
| 112 | model-usage | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch6.rs |
| 113 | claude-migration | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch6.rs |
| 114 | ast-refactor | pure-algorithm | real-algorithm | real-algorithm | AST spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch7.rs |
| 115 | code-quality | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch7.rs |
| 116 | connectome | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch7.rs |
| 117 | connectome-rs | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch7.rs |
| 118 | connectome-py | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch7.rs |
| 119 | connectome-js | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch7.rs |
| 120 | duplicate-detector | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch7.rs |
| 121 | safety-check | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch7.rs |
| 122 | safety-check-py | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch7.rs |
| 123 | safety-check-js | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch7.rs |
| 124 | polyglot | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch8.rs |
| 125 | polyglot-metrics | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch8.rs |
| 126 | polyglot-convert | pure-algorithm | real-algorithm | real-algorithm | lang\nfrom\ncode | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch8.rs |
| 127 | circuit-breaker | pure-algorithm | real-algorithm | real-algorithm | state query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch8.rs |
| 128 | retry-policy | pure-algorithm | real-algorithm | real-algorithm | max retry_ms | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch8.rs |
| 129 | graceful-shutdown | pure-algorithm | real-algorithm | real-algorithm | timeout_ms | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch8.rs |
| 130 | immune-status | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch8.rs |
| 131 | immune-antibody | pure-algorithm | real-algorithm | real-algorithm | threat spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch8.rs |
| 132 | immune-log | pure-algorithm | real-algorithm | real-algorithm | count | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch8.rs |
| 133 | plugin-list | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch8.rs |
| 134 | plugin-install | local-write | real-algorithm | unavailable | name\nsource | file write | none | none | 10000 | 1 MiB | blade_unavailable, blade_panicked | no | batch8.rs |
| 135 | plugin-remove | local-write | real-algorithm | unavailable | plugin name | file delete | none | none | 5000 | 1 MiB | blade_unavailable, blade_panicked | no | batch8.rs |
| 136 | dreamer-loop | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 137 | auto-evolve | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 138 | adaptive-evolve | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 139 | self-evolve | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 140 | mitosis | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 141 | bio-mitosis | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 142 | metamorphic-trigger | pure-algorithm | real-algorithm | real-algorithm | generations | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 143 | omnicoder | pure-algorithm | real-algorithm | real-algorithm | mode\ncode | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 144 | code-review | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 145 | agent-factory | pure-algorithm | real-algorithm | real-algorithm | type\ncaps | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 146 | commander | pure-algorithm | real-algorithm | real-algorithm | cmd args | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 147 | swarm-queen | pure-algorithm | real-algorithm | real-algorithm | count | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 148 | replicator | pure-algorithm | real-algorithm | real-algorithm | target\ncode | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch9.rs |
| 149 | vision-analyze | external-read | real-algorithm | unavailable | image path | network | curl | OPENAI_API_KEY | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch10.rs |
| 150 | vision-compare | external-read | real-algorithm | unavailable | img1 img2 | network | curl | OPENAI_API_KEY | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch10.rs |
| 151 | vision-ocr | external-read | real-algorithm | unavailable | image path | network | curl | OPENAI_API_KEY | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch10.rs |
| 152 | geolocation-lookup | external-read | real-algorithm | unavailable | query | network | curl | none | 10000 | 1 MiB | blade_unavailable, blade_panicked | no | batch10.rs |
| 153 | geolocation-distance | pure-algorithm | real-algorithm | real-algorithm | coords | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 154 | geolocation-memory-map | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 155 | navigation-route | pure-algorithm | real-algorithm | real-algorithm | route spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 156 | navigation-poi | pure-algorithm | real-algorithm | real-algorithm | query location | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 157 | collective-decision | pure-algorithm | real-algorithm | real-algorithm | choices | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 158 | collective-consciousness | pure-algorithm | real-algorithm | real-algorithm | count | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 159 | distributed-raft | pure-algorithm | real-algorithm | real-algorithm | nodes id | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 160 | distributed-lock | pure-algorithm | real-algorithm | real-algorithm | resource timeout | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 161 | alan-self-code | pure-algorithm | real-algorithm | real-algorithm | code\ninstruction | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 162 | alan-learn | pure-algorithm | real-algorithm | real-algorithm | pattern hours | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 163 | templates-refactor | pure-algorithm | real-algorithm | real-algorithm | template\ncode | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 164 | templates-list | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 165 | pollinations-generate | external-read | real-algorithm | unavailable | prompt | network | curl | none | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch10.rs |
| 166 | pollinations-memory-viz | external-read | real-algorithm | unavailable | prompt | network | curl | none | 15000 | 1 MiB | blade_unavailable, blade_panicked | no | batch10.rs |
| 167 | qr-generate | pure-algorithm | real-algorithm | real-algorithm | text | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 168 | qr-spine | pure-algorithm | real-algorithm | real-algorithm | text | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 169 | qr-scan | local-process | real-algorithm | unavailable | image path | file read | zbarimg | none | 10000 | 1 MiB | blade_unavailable, blade_panicked | no | batch10.rs |
| 170 | cryo-snap | pure-algorithm | real-algorithm | real-algorithm | data | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch10.rs |
| 171 | dna-extract | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 172 | dna-mutate | pure-algorithm | real-algorithm | real-algorithm | code type | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 173 | dna-mutate-point | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 174 | dna-mutate-insert | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 175 | dna-mutate-delete | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 176 | dna-mutate-optimize | pure-algorithm | real-algorithm | real-algorithm | code | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 177 | dna-crossover | pure-algorithm | real-algorithm | real-algorithm | code1\ncode2 | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 178 | dna-select | pure-algorithm | real-algorithm | real-algorithm | population | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 179 | dna-evolve | pure-algorithm | real-algorithm | real-algorithm | code\ngens | none | none | none | 10000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 180 | dna-teach | pure-algorithm | real-algorithm | real-algorithm | pattern | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 181 | dna-hebbian | pure-algorithm | real-algorithm | real-algorithm | data | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 182 | dna-stats | pure-algorithm | real-algorithm | real-algorithm | data | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 183 | brain | pure-algorithm | real-algorithm | real-algorithm | mode\ncode | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 184 | brain-compare | pure-algorithm | real-algorithm | real-algorithm | query | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch11.rs |
| 185 | dual-generate | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch12.rs |
| 186 | dual-cache | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch12.rs |
| 187 | dual-learn | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch12.rs |
| 188 | dual-record | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch12.rs |
| 189 | dual-status | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch12.rs |
| 190 | dual-teach | pure-algorithm | real-algorithm | real-algorithm | spec | none | none | none | 5000 | 1 MiB | empty_blade_output, blade_panicked | yes (batch) | batch12.rs |
| 191 | pipeline-architect | composite | adapter | real | rust_file\|boundary\|replacement | file write (transactional) | none | none | 10000 | 1 MiB | architect_refused, surgeon_refused | yes | composite.rs |
| 192 | macrophage | pure-algorithm | advisory | real | incident evidence text | none (advisory only) | none | none | 5000 | 1 MiB | bio_input_missing | yes | bio.rs |

## Status Summary

- **real**: 9 (code-reader, code-writer, diagnostics, git-nexus, rust-surgeon/pipeline-architect, summarize, sag, github, github-manager)
- **canonical measured status**: 168 `real` routes, including the dedicated `bio.rs` homeostasis adapters and 33 tested native Bio process targets
- **Bio actuator control plane**: `bio macrophage|synaptic|crispr plan|apply` is intentionally outside the 225-capability registry; applies require confirmation plus explicit effect permission
- **unavailable**: 24 (external services requiring credentials or CLI tools — github now real with auth probe)
- **unsupported**: 2 (apple-notes, bear-notes — macOS only)

## V1.4 Reclassification

After audit, the following blades were reclassified:

**Real pure-algorithm (deterministic, input-dependent, tested):**
- summarize: extractive summary by sentence frequency scoring
- sag: search-and-grep occurrence counting

**Still real-algorithm (real algorithms but need more integration tests):**
- All batch8-12 blades with existing tests (39 tests across 5 batch files)
- batch7 AST surgery, code quality, connectome, duplicate detector, safety check

**Marked unavailable (need external tools/credentials):**
- video-frames (needs ffmpeg)
- qr-scan (needs zbarimg)
- nano-pdf (needs nano-pdf CLI)
- pptx (needs pptx CLI)
- tmux (needs tmux)
- All external API adapters (github, notion, discord, etc.)

**Marked unsupported (macOS only):**
- apple-notes
- bear-notes

## Phase Status

- v1.3 CAPABILITY INVENTORY: COMPLETE
- v1.3 SAFE PROCESS RUNNER: COMPLETE (src/process.rs)
- v1.3 GIT-NEXUS REFACTOR: COMPLETE (uses process runner)
- v1.3 GATES: No shell execution, typed outcomes, 125 tests pass, fmt clean, clippy clean
- v1.4 LOCAL CAPABILITIES AUDIT: COMPLETE
- v1.4 INTEGRATION TESTS: COMPLETE (12 new tests for pure-algorithm blades)
- v1.4 GATES: All local capabilities real or honestly unsupported, 125 tests pass
- v1.5 EXTERNAL ADAPTERS: COMPLETE (src/external.rs)
- v1.5 GITHUB ADAPTER: COMPLETE (real gh CLI with auth probe)
- v1.5 GATES: External adapters with availability probe, auth state, typed failures, 125 tests pass
- v2.0 TYPED CONTRACTS: COMPLETE (src/contract.rs)
- v2.0 GATES: 159 tests pass, capability contracts validated
- v2.1 PHASE 1: COMPLETE (15 pure algorithm blades)
- v2.1 GATES: 176 tests pass
- v2.2 PHASE 2: COMPLETE (4 process wrapper blades)
- v2.2 GATES: 182 tests pass
- v2.3 PHASE 3: COMPLETE (6 external API blades)
- v2.3 GATES: 191 tests pass
- v2.4 PHASE 4: COMPLETE (15 meta/documentation blades)
- v2.4 GATES: 207 tests pass
- v2.5 PHASE 5: COMPLETE (143 additional blade implementations)
- v2.5 GATES: 249 tests pass, fmt clean, clippy clean, release build successful
