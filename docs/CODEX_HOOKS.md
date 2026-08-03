# Octopus — Codex integration (the wiring a coding agent actually runs)

How an agent (Codex / opencode / Claude-Code-style CLI) connects to the
`octopus-runtime` native binary. This is the practical, Codex-side layer: the
`octopus` skill's blade/pipeline orchestration, the MCP tool surface, and the
fail-closed Microscope commitment gate.

> The model is the motor.
> Octopus orchestrates the arms.
> Microscope Memory is the memory.

## The two surfaces Codex uses

```text
Codex session
    |
    +-- skill:         load octopus skill (blade/pipeline/arm contracts)
    +-- native CLI:    octopus-runtime.exe run|arm|pipeline|manifest
    +-- MCP (optional):octopus-runtime.exe mcp  -> blade tools, state, integrity
    +-- enforcement:   OCTOPUS_ENFORCE=1 (fail-closed Microscope gate)
```

### 1. Native CLI (the default path)

Build and invoke:

```powershell
cd D:\codex\octopus-rust-runtime
cargo build --release
$exe = "D:\codex\octopus-rust-runtime\target\release\octopus-runtime.exe"

# single blade
& $exe --plain run code-reader "src/lib.rs"

# composite arm (sequential, +)
& $exe --plain arm             "pipeline-architect + rust-surgeon" "Repair boundary"

# parallel pipeline (two composite arms)
& $exe --plain pipeline        "code-reader + diagnostics || pipeline-architect + rust-surgeon" "audit"

# manifest (declared arms)
& $exe --plain manifest        examples/evidence-manifest.json --allow-write
```

Every run creates a durable arm snapshot under the Octopus state dir
(`OCTOPUS_STATE_DIR`, default `~/.octopus-rust/`).

### MCP registration (`mcpServers`)

```json
{
  "mcpServers": {
    "octopus-runtime": {
      "command": "target\\release\\octopus-runtime.exe",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

The same server is pre-registered for OpenCode in `opencode.json`.
`octopus-runtime mcp` outputs MCP-compatible tool list only (its MCP reply is the
tool roster + state/integrity surface; heavy execution goes through the CLI).

## The Microscope commitment gate (fail-closed)

The native dispatcher loads Microscope enforcement state and calls
`can_execute()` before the blade executor. Opt-in, fail-closed:

```powershell
$env:OCTOPUS_ENFORCE = "1"
$env:OCTOPUS_ENFORCE_STATE_DIR = "D:\codex\microscope-memory\data"
$env:OCTOPUS_ENFORCE_ACTOR = "octopus"
$env:OCTOPUS_ENFORCE_SCOPE = "octopus"
# optional documented override:
$env:OCTOPUS_ENFORCE_JUSTIFICATION = "incident approved by guardian"
```

Blocked / attribution-error / corrupt-state / invalid chain => the blade never
starts (see `scripts/verify-enforcement-e2e.ps1` and Microscope
`docs/VALIDATION_REPORT.md`).

## Ready AGENTS.md snippet

```markdown
# Octopus — Codex integration

## Load the skill
Open  C:\Users\mater\.agents\skills\octopus\SKILL.md (blade/arm contract).

## MCP
Register the `octopus-runtime` MCP server as in .mcp.json; then the agent can
call the octopus tools.

## Invoke (native):
$exe = "D:\codex\octopus-rust-runtime\target\release\octopus-runtime.exe"
& $exe run <blade> "<prompt>"          # single blade
& $exe arm  "<a + b>" "<prompt>"      # composite
& $exe pipeline "<a || b>" "<prompt>" # parallel
```

## Reference

- `README.md` command matrix and `docs/CAPABILITY_MATRIX.md`.
- Enforcement: `.github`/`microscope` three-role reference implementation,
  `v0.9.1-octopus-native-enforcement` tag, `v0.9.1-commitment-enforcement` tag.
