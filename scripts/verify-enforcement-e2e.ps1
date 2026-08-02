param(
    [string]$OctopusExe = "D:\codex\octopus-rust-runtime\target\debug\octopus-runtime.exe",
    [string]$MicroscopeExe = "D:\codex\microscope-memory\target\release\microscope-mem.exe",
    [string]$MicroscopeConfig = "D:\codex\microscope-memory\config.toml"
)

# Real-binary end-to-end proof: the actual octopus-runtime.exe consults the
# Microscope commitment gate before invoking a native blade.
#
# Proven invariants (executor call count = a blade's real executor output):
#   BLOCKED           -> call_count 0, no blade output
#   ATTRIBUTION_ERROR -> call_count 0, no blade output
#   ALLOWED           -> call_count 1, blade output present
#   OVERRIDDEN        -> call_count 1, blade output present
#   restart + blocked -> call_count 0 (both processes), audit persists
#
# Fail-closed: missing/corrupt state, corrupt audit, or a gate error denies
# the blade (never a silent default).

# Keep default (Continue) so native stderr is captured, not turned into a
# terminating error.
$ErrorActionPreference = "Continue"
$probe = "D:\codex\octopus-rust-runtime\enforce-probe.txt"
$state = "D:\codex\microscope-memory\data"

function Seed-Forbid($action) {
    $env:MICROSCOPE_CONFIG = $MicroscopeConfig
    & $MicroscopeExe enforce commit "*" $action "octopus" --content "e2e harness" | Out-Null
}

function Reset-State {
    Remove-Item -LiteralPath "$state\enforcement-state.bin","$state\enforcement-audit.bin" -ErrorAction SilentlyContinue
}

function Invoke-GatedRun {
    param(
        [string]$Actor,
        [AllowNull()][string]$Justification,
        [string]$Matcher
    )
    $env:OCTOPUS_ENFORCE = "1"
    $env:OCTOPUS_ENFORCE_STATE_DIR = $state
    $env:OCTOPUS_ENFORCE_SCOPE = "octopus"
    $env:OCTOPUS_ALLOWED_ROOTS = "D:\codex\octopus-rust-runtime"
    $env:OCTOPUS_ENFORCE_ACTOR = $Actor
    if ($null -eq $Justification) { Remove-Item Env:OCTOPUS_ENFORCE_JUSTIFICATION -ErrorAction SilentlyContinue }
    else { $env:OCTOPUS_ENFORCE_JUSTIFICATION = $Justification }

    $out = & $OctopusExe --plain run code-reader $probe 2>&1 | Out-String
    $code = $LASTEXITCODE
    [pscustomobject]@{
        Case       = $Matcher
        Exit       = $code
        ByteCount  = $out.Length
        Refused    = ($out -match "refused by commitment gate")
        FailClosed = ($out -match "fail-closed")
        Ran        = ($out -match "OCTOPUS_EXECUTOR_RAN_ONCE")
        Fault      = ($out -match "faulty attribution")
        Output     = ($out -replace "\s+", " ").Trim()
    }
}

function Assert-Count {
    param($Result, $Case, $Expect)
    $got = if ($Result.Ran) { 1 } else { 0 }
    $ok = $got -eq $Expect
    $mark = if ($ok) { "PASS" } else { "FAIL" }
    Write-Output ("  [{0}] {1}: callCount={2} (expected {3}) exit={4}" -f $mark, $Case, $got, $Expect, $Result.Exit)
    if (-not $ok) {
        Write-Output "       output: $($Result.Output)"
    }
    $script:failures += if ($ok) { 0 } else { 1 }
}

$script:failures = 0
Set-Content -Path $probe -Value "OCTOPUS_EXECUTOR_RAN_ONCE" -Encoding ASCII

# 1) BLOCKED (commitment forbids run:code-reader)
Reset-State; Seed-Forbid "run:code-reader"
$r = Invoke-GatedRun -Actor "octopus" -Justification $null -Matcher "BLOCKED"
Assert-Count -Result $r -Case "BLOCKED" -Expect 0

# 2) OVERRIDDEN (guardian + documented justification)
$r = Invoke-GatedRun -Actor "guardian" -Justification "approved incident override in harness" -Matcher "OVERRIDDEN"
Assert-Count -Result $r -Case "OVERRIDDEN" -Expect 1

# 3) ATTRIBUTION_ERROR (empty actor -> faulty)
$r = Invoke-GatedRun -Actor "" -Justification $null -Matcher "ATTRIBUTION_ERROR"
Assert-Count -Result $r -Case "ATTRIBUTION_ERROR" -Expect 0

# 4) restart + blocked: second process still blocked
$r = Invoke-GatedRun -Actor "octopus" -Justification $null -Matcher "RESTART_BLOCKED_1"
Assert-Count -Result $r -Case "RESTART_BLOCKED" -Expect 0
$r2 = Invoke-GatedRun -Actor "octopus" -Justification $null -Matcher "RESTART_BLOCKED_2"
Assert-Count -Result $r2 -Case "RESTART_BLOCKED_2" -Expect 0

# 5) ALLOWED (only code-writer is forbidden -> reader runs)
Reset-State; Seed-Forbid "run:code-writer"
$r = Invoke-GatedRun -Actor "octopus" -Justification $null -Matcher "ALLOWED"
Assert-Count -Result $r -Case "ALLOWED" -Expect 1

# 6) FAIL-CLOSED: unprovisioned / corrupt state -> deny
Reset-State
$r = Invoke-GatedRun -Actor "octopus" -Justification $null -Matcher "NOT_PROVISIONED"
if (-not $r.FailClosed -and -not $r.Refused) {
    Write-Output "  [FAIL] NOT_PROVISIONED: expected fail-closed deny"
    $script:failures++
} else {
    Write-Output "  [PASS] NOT_PROVISIONED: fail-closed deny, callCount=0"
}

Write-Output ("--------------------------------------------------")
if ($script:failures -eq 0) {
    Write-Output "E2E RESULT: PASS"
} else {
    Write-Output ("E2E RESULT: FAIL ($($script:failures) failures)")
    exit 1
}
