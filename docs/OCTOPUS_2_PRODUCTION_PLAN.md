# Octopus Rust Runtime — V2.0 Production Plan

## Current State

- **Version**: 2.0.0
- **Tests**: 159/159 passing
- **Real adapters**: 6 (code-reader, code-writer, diagnostics, git-nexus, github, github-manager)
- **Copied-native blades**: ~158 (Hope batch file copies, not real implementations)
- **Modules**: 11 (lib, main, outcome, capability, contract, composite, process, external, approval, orchestration, snapshot, mcp, blade)

## Goal

Convert copied-native blades to real implementations where possible, and properly categorize the rest.

---

## Phase 1: Pure Algorithm Blades (Week 1-3)

These blades can be implemented purely in Rust with no external dependencies.

### 1.1 Text Analysis (~15 blades)

| Blade | Implementation | Complexity |
|-------|---------------|------------|
| `summarize` | Extractive summary (TF-IDF scoring) | Medium |
| `sag` | Text search with occurrence counting | Easy |
| `code-analysis` | Code metrics (lines, functions, complexity) | Easy |
| `diagnostics` | Already real | — |
| `mermaid-agent` | Mermaid diagram generation from text | Medium |
| `code-review` | Pattern-based code review | Medium |
| `duplicate-detector` | Exact + fuzzy duplicate detection | Medium |
| `code-quality` | Quality metrics (nesting, length, naming) | Medium |
| `ast-refactor` | AST-based refactoring suggestions | Hard |
| `connectome` | Code dependency graph | Hard |
| `safety-check` | Unsafe code detection | Medium |
| `polyglot` | Language detection (keyword frequency) | Easy |
| `polyglot-metrics` | Language-specific metrics | Medium |
| `brand-voice` | Writing style analysis | Medium |
| `prose` | Prose quality scoring | Medium |

### 1.2 Math/Science (~10 blades)

| Blade | Implementation | Complexity |
|-------|---------------|------------|
| `geolocation-distance` | Haversine formula | Easy |
| `dna-extract` | Function/struct extraction from code | Easy |
| `dual-generate` | Template-based generation | Medium |
| `circuit-breaker` | State machine (closed/open/half-open) | Easy |
| `retry-policy` | Configurable retry with backoff | Easy |
| `graceful-shutdown` | Shutdown coordinator | Easy |
| `bench-meter` | Simple benchmarking | Medium |
| `data-master` | Basic data analysis (mean, median, std) | Easy |
| `model-usage` | Token counting | Easy |

### 1.3 Code Transformation (~15 blades)

| Blade | Implementation | Complexity |
|-------|---------------|------------|
| `formatter` | Code formatting (simple rules) | Medium |
| `type-inference` | Basic type inference | Hard |
| `lint-rules` | Lint rule definitions | Medium |
| `crispr-hotfix` | Code patching | Medium |
| `synaptic-pruning` | Dead code detection | Medium |
| `omni-surgeon` | AST surgery | Hard |
| `file-surgeon` | File-based surgery | Medium |
| `stem-core` | Code generation from templates | Medium |
| `parser` | Simple parser generation | Hard |
| `crispr-hotfix-v2` | Enhanced patching | Medium |
| `synaptic-pruning-v2` | Enhanced pruning | Medium |
| `viral-transduction` | Code injection | Medium |
| `hox-architecture` | Architecture analysis | Medium |
| `ai-synapse` | Neural network code gen | Hard |
| `stem-cell-manager` | Template management | Medium |

---

## Phase 2: Process Wrapper Blades (Week 4-5)

These blades wrap external CLI tools.

### 2.1 Audio/Video (~5 blades)

| Blade | External Tool | Required |
|-------|--------------|----------|
| `sherpa-onnx-tts` | sherpa-onnx-tts | Install |
| `tts-voice` | sherpa-onnx-tts | Install |
| `stt-ear` | whisper | Install |
| `openai-whisper` | whisper | Install |
| `video-frames` | ffmpeg | Install |

### 2.2 Document Processing (~5 blades)

| Blade | External Tool | Required |
|-------|--------------|----------|
| `nano-pdf` | nano-pdf | Install |
| `pptx` | python-pptx | Python |
| `pptx` | python-pptx | Python |
| `web-extractor` | curl + parsing | — |
| `lobster-scraper` | curl + parsing | — |

### 2.3 System Tools (~10 blades)

| Blade | External Tool | Required |
|-------|--------------|----------|
| `tmux` | tmux | Install |
| `turborepo` | turbo | Node.js |
| `mcporter` | mcporter | Install |
| `eightctl` | eightctl | Install |
| `clawhub` | clawhub | Install |
| `forge-blade` | forge | Install |

---

## Phase 3: External API Blades (Week 6-8)

These blades require API keys and authentication.

### 3.1 Communication (~5 blades)

| Blade | API | Auth Required |
|-------|-----|---------------|
| `discord` | Discord Bot API | Bot token |
| `himalaya` | IMAP/SMTP | Email credentials |
| `wacli` | WhatsApp | Session |
| `voice-call` | Twilio | API key |
| `notion` | Notion API | Integration token |

### 3.2 Productivity (~10 blades)

| Blade | API | Auth Required |
|-------|-----|---------------|
| `gog` | Google Workspace | OAuth2 |
| `1password` | 1Password CLI | Account |
| `goplaces` | Google Places | API key |
| `local-places` | Google Places | API key |
| `weather` | OpenWeatherMap | API key |
| `apple-notes` | AppleScript | macOS only |
| `bear-notes` | Bear URL scheme | macOS only |

### 3.3 AI/ML (~10 blades)

| Blade | API | Auth Required |
|-------|-----|---------------|
| `openai-image-gen` | OpenAI API | API key |
| `openai-whisper` | OpenAI API | API key |
| `audio-diagnostics` | Various | API key |
| `ai-synapse` | Local/Cloud | Config |
| `claude-migration` | Anthropic API | API key |

### 3.4 Development (~10 blades)

| Blade | API | Auth Required |
|-------|-----|---------------|
| `merge-pr` | GitHub API | Token |
| `merge-pr-v1` | GitHub API | Token |
| `review-pr` | GitHub API | Token |
| `still-archive` | Local | None |
| `incubator` | Local | None |
| `model-usage` | Various | API keys |

---

## Phase 4: Meta/Skill Blades (Week 9)

These are documentation/template blades, not real execution.

| Category | Count | Implementation |
|----------|-------|----------------|
| Agent development | ~10 | Documentation only |
| Hook development | ~5 | Documentation only |
| Command development | ~5 | Documentation only |
| Testing patterns | ~5 | Documentation only |
| Architecture patterns | ~10 | Documentation only |
| Prompt engineering | ~5 | Documentation only |
| Memory/learning | ~10 | Documentation only |

---

## Implementation Strategy

### For Each Blade

1. **Contract definition** in `contract.rs`:
   - Input type and validation
   - Output format
   - Required tools/APIs
   - Deprecation status

2. **Implementation** in one of:
   - `capability.rs` (real adapter)
   - `blade/batchN.rs` (algorithm)
   - Process wrapper (external tool)

3. **Tests**:
   - Unit test for the blade
   - Integration test with real input
   - Error case tests

4. **Documentation**:
   - Update `CAPABILITY_MATRIX.md`
   - Update README.md

### Quality Gates

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

---

## Priority Order

### Must Have (Week 1-3)
- [ ] All pure algorithm blades (~40)
- [ ] Contract definitions for all blades
- [ ] Tests for all pure algorithm blades

### Should Have (Week 4-5)
- [ ] Process wrapper blades (~30)
- [ ] Integration tests with real tools

### Nice to Have (Week 6-8)
- [ ] External API blades (~40)
- [ ] OAuth2 flow for Google/GitHub

### Documentation (Week 9)
- [ ] Meta/skill blade documentation
- [ ] Updated capability matrix
- [ ] Production deployment guide

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Real blades | >100 (from 6) |
| Tests | >300 (from 159) |
| Coverage | >80% for pure algorithm |
| Documentation | 100% for all blades |
| Build time | <60s release |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| API key management | Environment variables, never hardcoded |
| Tool availability | Graceful fallback to `tool_unavailable` |
| Platform dependency | Windows-first, cross-platform where possible |
| Test flakiness | Mock external services in CI |
| Performance | Benchmark critical paths |

---

## Estimated Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1 | 3 weeks | 40 real algorithm blades |
| Phase 2 | 2 weeks | 30 process wrapper blades |
| Phase 3 | 3 weeks | 40 external API blades |
| Phase 4 | 1 week | Documentation |
| **Total** | **9 weeks** | **~191 real blades** |

---

## Notes

- External API blades will remain `unavailable` without proper credentials — this is by design
- macOS-only blades (apple-notes, bear-notes) will remain `unsupported` on Windows
- Meta/skill blades are documentation, not execution — they don't need "real" implementations
- The contract system ensures consistent input validation across all blades
