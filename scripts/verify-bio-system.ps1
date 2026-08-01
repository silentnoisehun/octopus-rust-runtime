param(
    [string]$Octopus = (Join-Path $PSScriptRoot "..\target\debug\octopus-runtime.exe"),
    [string]$BioBinaryDir = (Join-Path $PSScriptRoot "..\bio-binaries\target\release"),
    [string]$ArtifactRoot = (Join-Path $PSScriptRoot "..\..\.octopus-rust\bio-functional-smoke")
)

$ErrorActionPreference = "Stop"

$Octopus = [System.IO.Path]::GetFullPath($Octopus)
$BioBinaryDir = [System.IO.Path]::GetFullPath($BioBinaryDir)
$ArtifactRoot = [System.IO.Path]::GetFullPath($ArtifactRoot)

if (-not (Test-Path -LiteralPath $Octopus -PathType Leaf)) {
    throw "Octopus executable not found: $Octopus"
}

$expected = @(
    "viral-infect", "hox-diff", "plasmid-dream", "plasmid-inject",
    "telepathy-sync", "telepathy-entangle", "eqm-pulse", "eqm-methy",
    "aether-excite", "aether-fabric", "borg-cube", "nexus-logic",
    "collective-sync", "brain-synapse", "brain-connectome", "wave-encoder",
    "wave-sculptor", "iron-resonate", "path-resonance", "grid-warp",
    "magneto-geo", "mycelium-spread", "homeostasis", "omega-master",
    "omega-point", "ribosome-synth", "wave-cryo-tx", "wave-cryo-rx",
    "mutation-sentinel", "magneto-acoustic", "wave-field", "vagus-nerve",
    "microscope-mem"
)

$missing = @($expected | Where-Object {
    -not (Test-Path -LiteralPath (Join-Path $BioBinaryDir "$_.exe") -PathType Leaf)
})
if ($missing.Count -gt 0) {
    throw "Missing Bio-Binaries executables: $($missing -join ', ')"
}

$runId = Get-Date -Format "yyyyMMdd-HHmmss-fff"
$root = Join-Path $ArtifactRoot $runId
$fixture = Join-Path $root "fixture"
$source = Join-Path $root "sync-source"
$target = Join-Path $root "sync-target"
$gitFixture = Join-Path $root "git-fixture"
$temp = Join-Path $root "temp"
$state = Join-Path $root "octopus-state"
$queen = Join-Path $root "queen-state"

New-Item -ItemType Directory -Force -Path $fixture, $source, $target, $gitFixture, $temp, $state, $queen | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $fixture "src"), (Join-Path $fixture "tests") | Out-Null
Set-Content -LiteralPath (Join-Path $fixture "sample.txt") -Value "alpha beta alpha" -Encoding utf8
Set-Content -LiteralPath (Join-Path $fixture "src\lib.rs") -Value "pub fn alpha() -> &'static str { `"alpha`" }" -Encoding utf8
Set-Content -LiteralPath (Join-Path $fixture "tests\smoke.rs") -Value "#[test] fn smoke() { assert_eq!(2 + 2, 4); }" -Encoding utf8
Set-Content -LiteralPath (Join-Path $source "payload.txt") -Value "bio-sync" -Encoding utf8
Set-Content -LiteralPath (Join-Path $gitFixture "alpha.txt") -Value "alpha" -Encoding utf8
& git -C $gitFixture init --quiet
& git -C $gitFixture config user.name "Octopus Bio Smoke"
& git -C $gitFixture config user.email "octopus-bio-smoke@localhost"
& git -C $gitFixture add alpha.txt
& git -C $gitFixture commit --quiet -m "bio smoke fixture"
if ($LASTEXITCODE -ne 0) {
    throw "Could not create isolated git fixture for brain-synapse"
}

$wavePacket = Join-Path $root "wave.json"
$sculpted = Join-Path $root "sculpted.json"
$wav = Join-Path $root "health.wav"
$warpTarget = Join-Path $root "warp-target.txt"
$mutationKey = "octopus-smoke-$runId"

$old = @{
    OCTOPUS_STATE_DIR = $env:OCTOPUS_STATE_DIR
    OCTOPUS_ALLOWED_ROOTS = $env:OCTOPUS_ALLOWED_ROOTS
    OCTOPUS_BIO_BIN_DIR = $env:OCTOPUS_BIO_BIN_DIR
    OCTOPUS_BIO_STATE_DIR = $env:OCTOPUS_BIO_STATE_DIR
    TEMP = $env:TEMP
    TMP = $env:TMP
}

$env:OCTOPUS_STATE_DIR = $state
$env:OCTOPUS_ALLOWED_ROOTS = "$([System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..')));$root"
$env:OCTOPUS_BIO_BIN_DIR = $BioBinaryDir
$env:OCTOPUS_BIO_STATE_DIR = Join-Path $root "bio-state"
$env:TEMP = $temp
$env:TMP = $temp

$cases = [ordered]@{
    "viral-infect"       = @($fixture, "--pattern", "alpha", "--replace", "beta", "--ext", "txt", "--dry-run")
    "hox-diff"           = @($fixture)
    "plasmid-dream"      = @($fixture, "--command", "rustc --version", "--predict", "1")
    "plasmid-inject"     = @((Join-Path $fixture "sample.txt"), "--start", "1", "--end", "1", "--fix", "beta", "--dry-run")
    "telepathy-sync"     = @($source, $target, "--dry-run")
    "telepathy-entangle" = @("set", $mutationKey, "connected")
    "eqm-pulse"          = @()
    "eqm-methy"          = @($fixture, "--depth", "2")
    "aether-excite"      = @()
    "aether-fabric"      = @("--top", "5")
    "borg-cube"          = @("cmd /c exit 0", "--max-power", "1")
    "nexus-logic"        = @($fixture, "--ext", "txt", "--query", "alpha", "--limit", "5")
    "collective-sync"    = @("--echo-x", "127.0.0.1:1", "--topic", "octopus-smoke", "--vote", "OK")
    "brain-synapse"      = @($gitFixture, "--limit", "1", "--min-weight", "1")
    "brain-connectome"   = @($fixture, "--lang", "rust")
    "wave-encoder"       = @((Join-Path $fixture "sample.txt"), "--output", $wavePacket)
    "wave-sculptor"      = @($wavePacket, "--filter", "lowpass", "--cutoff", "1000", "--output", $sculpted)
    "iron-resonate"      = @("--samples", "1")
    "path-resonance"     = @($fixture, "--depth", "2")
    "grid-warp"          = @("--source", (Join-Path $fixture "sample.txt"), "--target", $warpTarget, "--dry-run")
    "magneto-geo"        = @($fixture, "--depth", "2")
    "mycelium-spread"    = @($fixture, "--depth", "2")
    "homeostasis"        = @("status")
    "omega-master"       = @("--state-dir", $queen, "key-info")
    "omega-point"        = @("--duration", "1", "--interval", "1")
    "ribosome-synth"     = @("generate", "--name", "smoke_drone", "--output-root", $root, "--apply")
    "wave-cryo-tx"       = @("test", "--duration-ms", "1")
    "wave-cryo-rx"       = @("monitor", "--duration-ms", "1")
    "mutation-sentinel"  = @("hash", (Join-Path $fixture "sample.txt"))
    "magneto-acoustic"   = @($fixture, "--output", $wav, "--tone-ms", "10", "--depth", "2")
    "wave-field"         = @("snapshot")
    "vagus-nerve"        = @("--snapshot")
    "microscope-mem"     = @("status")
}

$results = [System.Collections.Generic.List[object]]::new()

try {
    foreach ($name in $expected) {
        $childArgs = @($cases[$name])
        $octopusArgs = @("bio", "external", $name, "--allow-mutation", "--") + $childArgs
        $timer = [System.Diagnostics.Stopwatch]::StartNew()
        $output = (& $Octopus @octopusArgs 2>&1 | Out-String).Trim()
        $exitCode = $LASTEXITCODE
        $timer.Stop()

        $adapterObserved = $output -match "\[bio-binaries\] name=$([regex]::Escape($name)) "
        $passed = ($exitCode -eq 0) -and $adapterObserved
        $results.Add([pscustomobject]@{
            Name = $name
            Passed = $passed
            Exit = $exitCode
            Milliseconds = $timer.ElapsedMilliseconds
            Evidence = if ($adapterObserved) { "native-process" } else { "missing-adapter-evidence" }
            Output = $output
        })
    }
}
finally {
    foreach ($key in $old.Keys) {
        [System.Environment]::SetEnvironmentVariable($key, $old[$key], "Process")
    }
}

$ribosomeSource = Join-Path $root "smoke_drone.drone.rs"
$ribosomeBinary = Join-Path $root "smoke_drone.drone.exe"
$ribosomeRuns = $false
if (Test-Path -LiteralPath $ribosomeBinary -PathType Leaf) {
    $ribosomeOutput = (& $ribosomeBinary 2>&1 | Out-String)
    $ribosomeRuns = ($LASTEXITCODE -eq 0) -and ($ribosomeOutput -match "mode=minimal-offline")
}

$artifactChecks = @(
    [pscustomobject]@{ Name = "wave-encoder-output"; Passed = (Test-Path -LiteralPath $wavePacket -PathType Leaf) },
    [pscustomobject]@{ Name = "wave-sculptor-output"; Passed = (Test-Path -LiteralPath $sculpted -PathType Leaf) },
    [pscustomobject]@{ Name = "magneto-acoustic-output"; Passed = (Test-Path -LiteralPath $wav -PathType Leaf) },
    [pscustomobject]@{ Name = "grid-warp-dry-run"; Passed = -not (Test-Path -LiteralPath $warpTarget) },
    [pscustomobject]@{ Name = "ribosome-source"; Passed = (Test-Path -LiteralPath $ribosomeSource -PathType Leaf) },
    [pscustomobject]@{ Name = "ribosome-binary"; Passed = (Test-Path -LiteralPath $ribosomeBinary -PathType Leaf) },
    [pscustomobject]@{ Name = "ribosome-binary-runs"; Passed = $ribosomeRuns }
)

$failed = @($results | Where-Object { -not $_.Passed })
$failedArtifacts = @($artifactChecks | Where-Object { -not $_.Passed })
$latencies = @($results.Milliseconds | Sort-Object)
$median = if ($latencies.Count -eq 0) { 0 } else { $latencies[[math]::Floor($latencies.Count / 2)] }

$results | Select-Object Name, Passed, Exit, Milliseconds, Evidence | Format-Table -AutoSize
$artifactChecks | Format-Table -AutoSize
Write-Host "Bio functional smoke: $($results.Count - $failed.Count)/$($results.Count) passed; artifact checks: $($artifactChecks.Count - $failedArtifacts.Count)/$($artifactChecks.Count); median=${median}ms"
Write-Host "Evidence directory: $root"

if ($failed.Count -gt 0) {
    foreach ($failure in $failed) {
        Write-Host "`n--- FAILED: $($failure.Name) (exit $($failure.Exit)) ---"
        Write-Host $failure.Output
    }
}

if ($failed.Count -gt 0 -or $failedArtifacts.Count -gt 0) {
    exit 1
}
