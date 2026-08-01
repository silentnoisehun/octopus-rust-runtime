[CmdletBinding()]
param(
    [string]$Octopus = $(
        $installed = "C:\Users\mater\.agents\skills\octopus\bin\octopus-runtime.exe"
        if (Test-Path -LiteralPath $installed -PathType Leaf) {
            $installed
        }
        else {
            Join-Path $PSScriptRoot "..\target\release\octopus-runtime.exe"
        }
    ),
    [string]$BioBinaryDir = $(
        $installed = "C:\Users\mater\.agents\skills\octopus\bin\bio-binaries"
        if (Test-Path -LiteralPath $installed -PathType Container) {
            $installed
        }
        else {
            Join-Path $PSScriptRoot "..\bio-binaries\target\release"
        }
    ),
    [string]$ArtifactRoot = (Join-Path $PSScriptRoot "..\..\.octopus-rust\bio-benchmarks"),
    [ValidateRange(0, 20)]
    [int]$Warmup = 1,
    [ValidateRange(3, 100)]
    [int]$Samples = 5,
    [ValidateRange(1, 300)]
    [int]$TimeoutSeconds = 45,
    [ValidateRange(1, 32)]
    [int[]]$Parallelism = @(1, 2, 4, 8),
    [ValidateRange(1, 20)]
    [int]$ParallelRepeats = 2,
    [switch]$SkipParallel,
    [switch]$KeepRawOutput,
    [switch]$ValidateOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Octopus = [System.IO.Path]::GetFullPath($Octopus)
$BioBinaryDir = [System.IO.Path]::GetFullPath($BioBinaryDir)
$ArtifactRoot = [System.IO.Path]::GetFullPath($ArtifactRoot)
$TimeoutMilliseconds = $TimeoutSeconds * 1000

$Catalog = @(
    [pscustomobject]@{ Name = "viral-infect";       Effect = "write" },
    [pscustomobject]@{ Name = "hox-diff";           Effect = "read" },
    [pscustomobject]@{ Name = "plasmid-dream";      Effect = "control" },
    [pscustomobject]@{ Name = "plasmid-inject";     Effect = "write" },
    [pscustomobject]@{ Name = "telepathy-sync";     Effect = "write" },
    [pscustomobject]@{ Name = "telepathy-entangle"; Effect = "write" },
    [pscustomobject]@{ Name = "eqm-pulse";          Effect = "read" },
    [pscustomobject]@{ Name = "eqm-methy";          Effect = "write" },
    [pscustomobject]@{ Name = "aether-excite";      Effect = "read" },
    [pscustomobject]@{ Name = "aether-fabric";      Effect = "read" },
    [pscustomobject]@{ Name = "borg-cube";          Effect = "control" },
    [pscustomobject]@{ Name = "nexus-logic";        Effect = "write" },
    [pscustomobject]@{ Name = "collective-sync";    Effect = "control" },
    [pscustomobject]@{ Name = "brain-synapse";      Effect = "read" },
    [pscustomobject]@{ Name = "brain-connectome";   Effect = "read" },
    [pscustomobject]@{ Name = "wave-encoder";       Effect = "write" },
    [pscustomobject]@{ Name = "wave-sculptor";      Effect = "write" },
    [pscustomobject]@{ Name = "iron-resonate";      Effect = "read" },
    [pscustomobject]@{ Name = "path-resonance";     Effect = "read" },
    [pscustomobject]@{ Name = "grid-warp";          Effect = "write" },
    [pscustomobject]@{ Name = "magneto-geo";        Effect = "read" },
    [pscustomobject]@{ Name = "mycelium-spread";    Effect = "read" },
    [pscustomobject]@{ Name = "homeostasis";        Effect = "control" },
    [pscustomobject]@{ Name = "omega-master";       Effect = "control" },
    [pscustomobject]@{ Name = "omega-point";        Effect = "read" },
    [pscustomobject]@{ Name = "ribosome-synth";     Effect = "control" },
    [pscustomobject]@{ Name = "wave-cryo-tx";       Effect = "write" },
    [pscustomobject]@{ Name = "wave-cryo-rx";       Effect = "read" },
    [pscustomobject]@{ Name = "mutation-sentinel";  Effect = "control" },
    [pscustomobject]@{ Name = "magneto-acoustic";   Effect = "write" },
    [pscustomobject]@{ Name = "wave-field";         Effect = "write" },
    [pscustomobject]@{ Name = "vagus-nerve";        Effect = "write" },
    [pscustomobject]@{ Name = "microscope-mem";     Effect = "write" }
)

function Assert-Preconditions {
    if (-not (Test-Path -LiteralPath $Octopus -PathType Leaf)) {
        throw "Octopus executable not found: $Octopus"
    }
    if (-not (Test-Path -LiteralPath $BioBinaryDir -PathType Container)) {
        throw "Bio-Binaries directory not found: $BioBinaryDir"
    }
    if ($Catalog.Count -ne 33) {
        throw "Internal catalog must contain exactly 33 entries; observed $($Catalog.Count)"
    }

    $duplicates = @($Catalog | Group-Object Name | Where-Object Count -ne 1)
    if ($duplicates.Count -gt 0) {
        throw "Duplicate Bio catalog entries: $($duplicates.Name -join ', ')"
    }

    $badEffects = @($Catalog | Where-Object Effect -notin @("read", "write", "control"))
    if ($badEffects.Count -gt 0) {
        throw "Invalid effects in Bio catalog: $($badEffects.Name -join ', ')"
    }

    $missing = @($Catalog | Where-Object {
        -not (Test-Path -LiteralPath (Join-Path $BioBinaryDir "$($_.Name).exe") -PathType Leaf)
    })
    if ($missing.Count -gt 0) {
        throw "Missing Bio-Binaries executables: $($missing.Name -join ', ')"
    }

    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        throw "git is required to construct the isolated brain-synapse fixture"
    }
}

function Set-Utf8File {
    param(
        [Parameter(Mandatory)] [string]$Path,
        [Parameter(Mandatory)] [string]$Value
    )

    [System.IO.File]::WriteAllText($Path, $Value, [System.Text.UTF8Encoding]::new($false))
}

function New-BenchmarkFixture {
    param(
        [Parameter(Mandatory)] [string]$Root,
        [Parameter(Mandatory)] [string]$Name,
        [Parameter(Mandatory)] [string]$Token
    )

    $fixture = Join-Path $Root "fixture"
    $source = Join-Path $Root "sync-source"
    $target = Join-Path $Root "sync-target"
    $gitFixture = Join-Path $Root "git-fixture"
    $temp = Join-Path $Root "temp"
    $state = Join-Path $Root "octopus-state"
    $bioState = Join-Path $Root "bio-state"
    $queen = Join-Path $Root "queen-state"
    $appData = Join-Path $Root "appdata"
    $localAppData = Join-Path $Root "localappdata"
    $homeDirectory = Join-Path $Root "home"
    $integrity = Join-Path $bioState "direct-integrity"

    $directories = @(
        $Root, $fixture, (Join-Path $fixture "src"), (Join-Path $fixture "tests"),
        $source, $target, $gitFixture, $temp, $state, $bioState, $queen,
        $appData, $localAppData, $homeDirectory, $integrity
    )
    [System.IO.Directory]::CreateDirectory($Root) | Out-Null
    foreach ($directory in $directories) {
        [System.IO.Directory]::CreateDirectory($directory) | Out-Null
    }

    $sample = Join-Path $fixture "sample.txt"
    Set-Utf8File -Path $sample -Value "alpha beta alpha`n"
    Set-Utf8File -Path (Join-Path $fixture "src\lib.rs") -Value "pub fn alpha() -> &'static str { `"alpha`" }`n"
    Set-Utf8File -Path (Join-Path $fixture "tests\smoke.rs") -Value "#[test] fn smoke() { assert_eq!(2 + 2, 4); }`n"
    Set-Utf8File -Path (Join-Path $source "payload.txt") -Value "bio-sync`n"
    Set-Utf8File -Path (Join-Path $gitFixture "alpha.txt") -Value "alpha`n"

    if ($Name -eq "brain-synapse") {
        & git -C $gitFixture init --quiet
        & git -C $gitFixture config user.name "Octopus Bio Benchmark"
        & git -C $gitFixture config user.email "octopus-bio-benchmark@localhost"
        & git -C $gitFixture config core.autocrlf false
        & git -C $gitFixture add alpha.txt
        & git -C $gitFixture commit --quiet -m "isolated benchmark fixture"
        if ($LASTEXITCODE -ne 0) {
            throw "Could not create isolated git fixture at $gitFixture"
        }
    }

    $wavePacket = Join-Path $Root "wave.json"
    $wave = [ordered]@{
        timestamp = "2026-01-01T00:00:00Z"
        source_file = $sample
        source_size_bytes = 17
        base_frequency_hz = 432.0
        blake3_hash = ("0" * 64)
        bins = @(
            [ordered]@{ frequency_hz = 432.0; amplitude = 0.5; phase_rad = 0.0 },
            [ordered]@{ frequency_hz = 1296.0; amplitude = 0.25; phase_rad = 1.0 }
        )
        total_spectral_energy = 0.3125
        dominant_frequency_hz = 432.0
        encoding_fidelity = 1.0
    }
    Set-Utf8File -Path $wavePacket -Value ($wave | ConvertTo-Json -Depth 6)

    [pscustomobject]@{
        Root = $Root
        Fixture = $fixture
        Source = $source
        Target = $target
        GitFixture = $gitFixture
        Temp = $temp
        State = $state
        BioState = $bioState
        Queen = $queen
        AppData = $appData
        LocalAppData = $localAppData
        Home = $homeDirectory
        Integrity = $integrity
        Sample = $sample
        WavePacket = $wavePacket
        EncoderOutput = (Join-Path $Root "encoded-wave.json")
        Sculpted = (Join-Path $Root "sculpted.json")
        WaveAudio = (Join-Path $Root "health.wav")
        WarpTarget = (Join-Path $Root "warp-target.txt")
        Token = $Token
    }
}

function Get-CaseArguments {
    param(
        [Parameter(Mandatory)] [string]$Name,
        [Parameter(Mandatory)] [pscustomobject]$Context
    )

    switch ($Name) {
        "viral-infect"       { return @($Context.Fixture, "--pattern", "alpha", "--replace", "beta", "--ext", "txt", "--dry-run") }
        "hox-diff"           { return @($Context.Fixture) }
        "plasmid-dream"      { return @($Context.Fixture, "--command", "rustc --version", "--predict", "1") }
        "plasmid-inject"     { return @($Context.Sample, "--start", "1", "--end", "1", "--fix", "beta", "--dry-run") }
        "telepathy-sync"     { return @($Context.Source, $Context.Target, "--dry-run") }
        "telepathy-entangle" { return @("set", $Context.Token, "connected") }
        "eqm-pulse"          { return @() }
        "eqm-methy"          { return @($Context.Fixture, "--depth", "2") }
        "aether-excite"      { return @() }
        "aether-fabric"      { return @("--top", "5") }
        "borg-cube"          { return @("cmd /c exit 0", "--max-power", "1") }
        "nexus-logic"        { return @($Context.Fixture, "--ext", "txt", "--query", "alpha", "--limit", "5") }
        "collective-sync"    { return @("--echo-x", "127.0.0.1:1", "--topic", "octopus-benchmark", "--vote", "OK") }
        "brain-synapse"      { return @($Context.GitFixture, "--limit", "1", "--min-weight", "1") }
        "brain-connectome"   { return @($Context.Fixture, "--lang", "rust") }
        "wave-encoder"       { return @($Context.Sample, "--output", $Context.EncoderOutput) }
        "wave-sculptor"      { return @($Context.WavePacket, "--filter", "lowpass", "--cutoff", "1000", "--output", $Context.Sculpted) }
        "iron-resonate"      { return @("--samples", "1") }
        "path-resonance"     { return @($Context.Fixture, "--depth", "2") }
        "grid-warp"          { return @("--source", $Context.Sample, "--target", $Context.WarpTarget, "--dry-run") }
        "magneto-geo"        { return @($Context.Fixture, "--depth", "2") }
        "mycelium-spread"    { return @($Context.Fixture, "--depth", "2") }
        "homeostasis"        { return @("status") }
        "omega-master"       { return @("--state-dir", $Context.Queen, "key-info") }
        "omega-point"        { return @("--duration", "1", "--interval", "1") }
        "ribosome-synth"     { return @("generate", "--name", "benchmark_drone", "--output-root", $Context.Root) }
        "wave-cryo-tx"       { return @("test", "--duration-ms", "1") }
        "wave-cryo-rx"       { return @("monitor", "--duration-ms", "1") }
        "mutation-sentinel"  { return @("hash", $Context.Sample) }
        "magneto-acoustic"   { return @($Context.Fixture, "--output", $Context.WaveAudio, "--tone-ms", "10", "--depth", "2") }
        "wave-field"         { return @("snapshot") }
        "vagus-nerve"        { return @("--snapshot") }
        "microscope-mem"     { return @("status") }
        default { throw "No benchmark case is defined for '$Name'" }
    }
}

function Get-IsolatedEnvironment {
    param([Parameter(Mandatory)] [pscustomobject]$Context)

    $allowedRoots = @(
        [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..")),
        $Context.Root
    ) -join ";"

    @{
        "TEMP" = $Context.Temp
        "TMP" = $Context.Temp
        "APPDATA" = $Context.AppData
        "LOCALAPPDATA" = $Context.LocalAppData
        "USERPROFILE" = $Context.Home
        "HOME" = $Context.Home
        "OCTOPUS_STATE_DIR" = $Context.State
        "OCTOPUS_ALLOWED_ROOTS" = $allowedRoots
        "OCTOPUS_BIO_BIN_DIR" = $BioBinaryDir
        "OCTOPUS_BIO_STATE_DIR" = $Context.BioState
        "BIO_INTEGRITY_DIR" = $Context.Integrity
        "NO_COLOR" = "1"
        "RUST_BACKTRACE" = "0"
    }
}

function New-InvocationSpec {
    param(
        [Parameter(Mandatory)] [pscustomobject]$Entry,
        [Parameter(Mandatory)] [ValidateSet("direct", "octopus")] [string]$Mode,
        [Parameter(Mandatory)] [ValidateSet("warmup", "measured", "parallel")] [string]$Phase,
        [Parameter(Mandatory)] [int]$Iteration,
        [Parameter(Mandatory)] [int]$Order,
        [Parameter(Mandatory)] [string]$Root,
        [string]$SharedOctopusState
    )

    $safeName = $Entry.Name -replace "[^a-zA-Z0-9_-]", "_"
    $contextRoot = Join-Path $Root ("{0}-{1}-{2:D3}" -f $safeName, $Mode, $Iteration)
    $token = "bio-bench-$safeName-$Mode-$Phase-$Iteration"
    $context = New-BenchmarkFixture -Root $contextRoot -Name $Entry.Name -Token $token
    $environment = Get-IsolatedEnvironment -Context $context
    if ($SharedOctopusState) {
        [System.IO.Directory]::CreateDirectory($SharedOctopusState) | Out-Null
        $environment["OCTOPUS_STATE_DIR"] = $SharedOctopusState
    }
    $childArguments = @(Get-CaseArguments -Name $Entry.Name -Context $context)

    if ($Mode -eq "direct") {
        $fileName = Join-Path $BioBinaryDir "$($Entry.Name).exe"
        $arguments = $childArguments
    }
    else {
        $fileName = $Octopus
        $arguments = @("bio", "external", $Entry.Name)
        if ($Entry.Effect -ne "read") {
            $arguments += "--allow-mutation"
        }
        $arguments += "--"
        $arguments += $childArguments
    }

    [pscustomobject]@{
        Name = $Entry.Name
        Effect = $Entry.Effect
        Mode = $Mode
        Phase = $Phase
        Iteration = $Iteration
        Order = $Order
        Context = $context
        FileName = $fileName
        Arguments = @($arguments)
        Environment = $environment
    }
}

function Start-TimedProcess {
    param([Parameter(Mandatory)] [pscustomobject]$Spec)

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $Spec.FileName
    $startInfo.WorkingDirectory = $BioBinaryDir
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($argument in $Spec.Arguments) {
        $startInfo.ArgumentList.Add([string]$argument)
    }
    foreach ($key in $Spec.Environment.Keys) {
        $startInfo.Environment[$key] = [string]$Spec.Environment[$key]
    }

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        if (-not $process.Start()) {
            throw "Process.Start returned false"
        }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
    }
    catch {
        $watch.Stop()
        $process.Dispose()
        throw
    }

    [pscustomobject]@{
        Spec = $Spec
        Process = $process
        Watch = $watch
        StdoutTask = $stdoutTask
        StderrTask = $stderrTask
    }
}

function Complete-TimedProcess {
    param(
        [Parameter(Mandatory)] [pscustomobject]$Running,
        [Parameter(Mandatory)] [int]$TimeoutMs
    )

    $process = $Running.Process
    $timedOut = $false
    $errorText = ""
    try {
        if (-not $process.WaitForExit($TimeoutMs)) {
            $timedOut = $true
            try { $process.Kill($true) } catch { $errorText = "kill failed: $($_.Exception.Message)" }
            $process.WaitForExit()
        }
        $Running.Watch.Stop()

        $stdout = $Running.StdoutTask.GetAwaiter().GetResult()
        $stderr = $Running.StderrTask.GetAwaiter().GetResult()
        $exitCode = $process.ExitCode
        try { $cpuMs = $process.TotalProcessorTime.TotalMilliseconds } catch { $cpuMs = [double]::NaN }
        try { $peakRssMiB = $process.PeakWorkingSet64 / 1MB } catch { $peakRssMiB = [double]::NaN }
    }
    catch {
        $Running.Watch.Stop()
        $stdout = ""
        $stderr = ""
        $exitCode = -1
        $cpuMs = [double]::NaN
        $peakRssMiB = [double]::NaN
        $errorText = $_.Exception.Message
    }
    finally {
        $process.Dispose()
    }

    $adapterObserved = $Running.Spec.Mode -eq "direct" -or
        $stdout -match "\[bio-binaries\] name=$([regex]::Escape($Running.Spec.Name)) "
    $artifactPassed = Test-InvocationArtifact -Name $Running.Spec.Name -Context $Running.Spec.Context
    $passed = (-not $timedOut) -and ($exitCode -eq 0) -and $adapterObserved -and $artifactPassed

    [pscustomobject]@{
        Name = $Running.Spec.Name
        Effect = $Running.Spec.Effect
        Mode = $Running.Spec.Mode
        Phase = $Running.Spec.Phase
        Iteration = $Running.Spec.Iteration
        Order = $Running.Spec.Order
        Passed = $passed
        ExitCode = $exitCode
        TimedOut = $timedOut
        WallMs = [math]::Round($Running.Watch.Elapsed.TotalMilliseconds, 3)
        CpuMs = [math]::Round($cpuMs, 3)
        PeakWorkingSetMiB = [math]::Round($peakRssMiB, 3)
        StdoutBytes = [System.Text.Encoding]::UTF8.GetByteCount($stdout)
        StderrBytes = [System.Text.Encoding]::UTF8.GetByteCount($stderr)
        AdapterObserved = $adapterObserved
        ArtifactPassed = $artifactPassed
        Error = $errorText
        Stdout = $stdout
        Stderr = $stderr
        ContextRoot = $Running.Spec.Context.Root
    }
}

function Test-InvocationArtifact {
    param(
        [Parameter(Mandatory)] [string]$Name,
        [Parameter(Mandatory)] [pscustomobject]$Context
    )

    switch ($Name) {
        "wave-encoder"     { return Test-Path -LiteralPath $Context.EncoderOutput -PathType Leaf }
        "wave-sculptor"    { return Test-Path -LiteralPath $Context.Sculpted -PathType Leaf }
        "magneto-acoustic" { return Test-Path -LiteralPath $Context.WaveAudio -PathType Leaf }
        "grid-warp"        { return -not (Test-Path -LiteralPath $Context.WarpTarget) }
        default             { return $true }
    }
}

function Invoke-One {
    param([Parameter(Mandatory)] [pscustomobject]$Spec)

    try {
        $running = Start-TimedProcess -Spec $Spec
        Complete-TimedProcess -Running $running -TimeoutMs $TimeoutMilliseconds
    }
    catch {
        [pscustomobject]@{
            Name = $Spec.Name; Effect = $Spec.Effect; Mode = $Spec.Mode; Phase = $Spec.Phase
            Iteration = $Spec.Iteration; Order = $Spec.Order; Passed = $false; ExitCode = -1
            TimedOut = $false; WallMs = [double]::NaN; CpuMs = [double]::NaN
            PeakWorkingSetMiB = [double]::NaN; StdoutBytes = 0; StderrBytes = 0
            AdapterObserved = $false; ArtifactPassed = $false; Error = $_.Exception.Message
            Stdout = ""; Stderr = ""; ContextRoot = $Spec.Context.Root
        }
    }
}

function Get-Percentile {
    param(
        [Parameter(Mandatory)] [double[]]$Values,
        [Parameter(Mandatory)] [ValidateRange(0.0, 1.0)] [double]$Percentile
    )

    $ordered = @($Values | Where-Object { -not [double]::IsNaN($_) } | Sort-Object)
    if ($ordered.Count -eq 0) { return [double]::NaN }
    $index = [math]::Max(0, [math]::Ceiling($Percentile * $ordered.Count) - 1)
    [double]$ordered[$index]
}

function Get-Median {
    param([Parameter(Mandatory)] [double[]]$Values)

    $ordered = @($Values | Where-Object { -not [double]::IsNaN($_) } | Sort-Object)
    if ($ordered.Count -eq 0) { return [double]::NaN }
    $middle = [math]::Floor($ordered.Count / 2)
    if (($ordered.Count % 2) -eq 1) { return [double]$ordered[$middle] }
    ([double]$ordered[$middle - 1] + [double]$ordered[$middle]) / 2.0
}

function Get-Stats {
    param([Parameter(Mandatory)] [object[]]$Rows)

    $successful = @($Rows | Where-Object Passed)
    $wall = [double[]]@($successful.WallMs)
    $cpu = [double[]]@($successful.CpuMs)
    $rss = [double[]]@($successful.PeakWorkingSetMiB)
    [pscustomobject]@{
        Count = $Rows.Count
        Passed = $successful.Count
        MinMs = if ($wall.Count) { ($wall | Measure-Object -Minimum).Minimum } else { [double]::NaN }
        MedianMs = Get-Median -Values $wall
        P95Ms = Get-Percentile -Values $wall -Percentile 0.95
        MaxMs = if ($wall.Count) { ($wall | Measure-Object -Maximum).Maximum } else { [double]::NaN }
        MedianCpuMs = Get-Median -Values $cpu
        MedianPeakRssMiB = Get-Median -Values $rss
        MedianStdoutBytes = Get-Median -Values ([double[]]@($successful.StdoutBytes))
        MedianStderrBytes = Get-Median -Values ([double[]]@($successful.StderrBytes))
    }
}

function Format-Number {
    param(
        [double]$Value,
        [string]$Pattern = "0.00"
    )
    if ([double]::IsNaN($Value) -or [double]::IsInfinity($Value)) { return "n/a" }
    $Value.ToString($Pattern, [System.Globalization.CultureInfo]::InvariantCulture)
}

function Save-RawEvidence {
    param(
        [Parameter(Mandatory)] [object[]]$Rows,
        [Parameter(Mandatory)] [string]$Directory,
        [switch]$All
    )

    [System.IO.Directory]::CreateDirectory($Directory) | Out-Null
    foreach ($row in $Rows) {
        if (-not $All -and $row.Passed) { continue }
        $stem = "{0}-{1}-{2}-{3:D3}" -f $row.Name, $row.Mode, $row.Phase, $row.Iteration
        Set-Utf8File -Path (Join-Path $Directory "$stem.stdout.txt") -Value $row.Stdout
        Set-Utf8File -Path (Join-Path $Directory "$stem.stderr.txt") -Value ($row.Stderr + "`n" + $row.Error)
    }
}

function Invoke-ParallelScaling {
    param(
        [Parameter(Mandatory)] [string]$Root,
        [Parameter(Mandatory)] [object[]]$ReadCatalog
    )

    $degrees = @((@($Parallelism) + 1) | Sort-Object -Unique)
    $jobs = [System.Collections.Generic.List[object]]::new()
    for ($repeat = 0; $repeat -lt $ParallelRepeats; $repeat++) {
        foreach ($entry in $ReadCatalog) {
            $jobs.Add([pscustomobject]@{ Entry = $entry; Job = $jobs.Count })
        }
    }

    $rows = [System.Collections.Generic.List[object]]::new()
    foreach ($degree in $degrees) {
        $degreeRoot = Join-Path $Root "degree-$degree"
        $sharedState = Join-Path $degreeRoot "shared-octopus-state"
        $specs = [System.Collections.Generic.List[object]]::new()
        foreach ($job in $jobs) {
            $specRoot = Join-Path $degreeRoot ("job-{0:D3}" -f $job.Job)
            $specs.Add((New-InvocationSpec -Entry $job.Entry -Mode octopus -Phase parallel -Iteration $job.Job -Order $job.Job -Root $specRoot -SharedOctopusState $sharedState))
        }

        $batchWatch = [System.Diagnostics.Stopwatch]::StartNew()
        $degreeResults = [System.Collections.Generic.List[object]]::new()
        for ($offset = 0; $offset -lt $specs.Count; $offset += $degree) {
            $running = [System.Collections.Generic.List[object]]::new()
            $last = [math]::Min($offset + $degree - 1, $specs.Count - 1)
            for ($index = $offset; $index -le $last; $index++) {
                try {
                    $running.Add((Start-TimedProcess -Spec $specs[$index]))
                }
                catch {
                    $degreeResults.Add((Invoke-One -Spec $specs[$index]))
                }
            }
            foreach ($active in $running) {
                $degreeResults.Add((Complete-TimedProcess -Running $active -TimeoutMs $TimeoutMilliseconds))
            }
        }
        $batchWatch.Stop()

        $seconds = $batchWatch.Elapsed.TotalSeconds
        $passed = @($degreeResults | Where-Object Passed).Count
        $rows.Add([pscustomobject]@{
            Degree = $degree
            Jobs = $degreeResults.Count
            Passed = $passed
            MakespanMs = [math]::Round($batchWatch.Elapsed.TotalMilliseconds, 3)
            ThroughputPerSecond = if ($seconds -gt 0) { [math]::Round($degreeResults.Count / $seconds, 3) } else { [double]::NaN }
            Speedup = [double]::NaN
            EfficiencyPercent = [double]::NaN
        })
    }

    $baseline = @($rows | Where-Object Degree -eq 1 | Select-Object -First 1)
    if ($baseline.Count -eq 1 -and $baseline[0].MakespanMs -gt 0) {
        foreach ($row in $rows) {
            $row.Speedup = [math]::Round($baseline[0].MakespanMs / $row.MakespanMs, 3)
            $row.EfficiencyPercent = [math]::Round(($row.Speedup / $row.Degree) * 100.0, 2)
        }
    }
    @($rows)
}

Assert-Preconditions

if ($ValidateOnly) {
    Write-Host "Benchmark harness validation passed: 33/33 binaries, 33 unique catalog entries, valid effects."
    Write-Host "Octopus: $Octopus"
    Write-Host "Bio-Binaries: $BioBinaryDir"
    exit 0
}

$runId = Get-Date -Format "yyyyMMdd-HHmmss-fff"
$runRoot = Join-Path $ArtifactRoot $runId
$fixtureRoot = Join-Path $runRoot "fixtures"
$rawRoot = Join-Path $runRoot "raw"
[System.IO.Directory]::CreateDirectory($fixtureRoot) | Out-Null

$environmentPath = Join-Path $runRoot "bio-benchmark-environment.txt"
$cpuInfo = Get-CimInstance Win32_Processor | Select-Object -First 1
$osInfo = Get-CimInstance Win32_OperatingSystem
$computerInfo = Get-CimInstance Win32_ComputerSystem
$octopusHash = (Get-FileHash -LiteralPath $Octopus -Algorithm SHA256).Hash
$harnessHash = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash
$bioHashes = @($Catalog | ForEach-Object {
    $path = Join-Path $BioBinaryDir "$($_.Name).exe"
    "$($_.Name).exe $((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash) $((Get-Item -LiteralPath $path).Length)"
})
$gitState = (& git -C (Join-Path $PSScriptRoot "..") status --short 2>$null | Out-String).Trim()
$environmentLines = @(
    "run=$runId",
    "started_utc=$([DateTime]::UtcNow.ToString('O'))",
    "timezone=$([TimeZoneInfo]::Local.Id)",
    "host=$([Environment]::MachineName)",
    "os=$($osInfo.Caption) build=$($osInfo.BuildNumber)",
    "cpu=$($cpuInfo.Name)",
    "cores=$($cpuInfo.NumberOfCores) logical=$($cpuInfo.NumberOfLogicalProcessors)",
    "ram_gib=$([math]::Round($computerInfo.TotalPhysicalMemory / 1GB, 3))",
    "powershell=$($PSVersionTable.PSVersion)",
    "octopus=$Octopus",
    "octopus_sha256=$octopusHash",
    "harness_sha256=$harnessHash",
    "bio_directory=$BioBinaryDir",
    "samples=$Samples warmup=$Warmup timeout_seconds=$TimeoutSeconds",
    "worktree_dirty=$([bool]$gitState)",
    "measurement_scope=end-to-end process latency on small isolated functional fixtures",
    "bio_executables:",
    ($bioHashes -join "`n")
)
Set-Utf8File -Path $environmentPath -Value (($environmentLines -join "`n") + "`n")

$allRows = [System.Collections.Generic.List[object]]::new()
$benchmarkWatch = [System.Diagnostics.Stopwatch]::StartNew()

for ($moduleIndex = 0; $moduleIndex -lt $Catalog.Count; $moduleIndex++) {
    $entry = $Catalog[$moduleIndex]
    Write-Host ("[{0,2}/33] {1} [{2}]" -f ($moduleIndex + 1), $entry.Name, $entry.Effect)

    $totalIterations = $Warmup + $Samples
    for ($index = 0; $index -lt $totalIterations; $index++) {
        $phase = if ($index -lt $Warmup) { "warmup" } else { "measured" }
        $iteration = if ($phase -eq "warmup") { $index } else { $index - $Warmup }
        $modes = if ((($index + $moduleIndex) % 2) -eq 0) { @("direct", "octopus") } else { @("octopus", "direct") }

        for ($order = 0; $order -lt $modes.Count; $order++) {
            $mode = $modes[$order]
            $iterationRoot = Join-Path $fixtureRoot ("{0}\{1}\{2:D3}" -f $entry.Name, $phase, $iteration)
            $spec = New-InvocationSpec -Entry $entry -Mode $mode -Phase $phase -Iteration $iteration -Order $order -Root $iterationRoot
            $result = Invoke-One -Spec $spec
            $allRows.Add($result)
        }
    }
}

$benchmarkWatch.Stop()
[System.IO.File]::AppendAllText(
    $environmentPath,
    "ended_utc=$([DateTime]::UtcNow.ToString('O'))`n",
    [System.Text.UTF8Encoding]::new($false)
)
$measuredRows = @($allRows | Where-Object Phase -eq "measured")
$warmupFailures = @($allRows | Where-Object { $_.Phase -eq "warmup" -and -not $_.Passed })
$summary = [System.Collections.Generic.List[object]]::new()

foreach ($entry in $Catalog) {
    $directRows = @($measuredRows | Where-Object { $_.Name -eq $entry.Name -and $_.Mode -eq "direct" })
    $octopusRows = @($measuredRows | Where-Object { $_.Name -eq $entry.Name -and $_.Mode -eq "octopus" })
    $direct = Get-Stats -Rows $directRows
    $octopusStats = Get-Stats -Rows $octopusRows
    $moduleWarmupFailures = @($warmupFailures | Where-Object Name -eq $entry.Name).Count
    $pairDeltas = [System.Collections.Generic.List[double]]::new()
    $pairRatios = [System.Collections.Generic.List[double]]::new()
    for ($pairIndex = 0; $pairIndex -lt $Samples; $pairIndex++) {
        $directPair = @($directRows | Where-Object { $_.Iteration -eq $pairIndex -and $_.Passed } | Select-Object -First 1)
        $octopusPair = @($octopusRows | Where-Object { $_.Iteration -eq $pairIndex -and $_.Passed } | Select-Object -First 1)
        if ($directPair.Count -eq 1 -and $octopusPair.Count -eq 1) {
            $pairDeltas.Add($octopusPair[0].WallMs - $directPair[0].WallMs)
            if ($directPair[0].WallMs -gt 0) {
                $pairRatios.Add($octopusPair[0].WallMs / $directPair[0].WallMs)
            }
        }
    }
    $overheadMs = Get-Median -Values ([double[]]$pairDeltas.ToArray())
    $overheadRatio = Get-Median -Values ([double[]]$pairRatios.ToArray())
    $passed = ($direct.Passed -eq $Samples) -and ($octopusStats.Passed -eq $Samples) -and
        ($pairDeltas.Count -eq $Samples) -and ($moduleWarmupFailures -eq 0)

    $summary.Add([pscustomobject]@{
        Name = $entry.Name
        Effect = $entry.Effect
        Passed = $passed
        SamplesPerMode = $Samples
        SuccessfulPairs = $pairDeltas.Count
        WarmupFailures = $moduleWarmupFailures
        DirectPassed = $direct.Passed
        DirectMinMs = [math]::Round($direct.MinMs, 3)
        DirectMedianMs = [math]::Round($direct.MedianMs, 3)
        DirectP95Ms = [math]::Round($direct.P95Ms, 3)
        DirectMaxMs = [math]::Round($direct.MaxMs, 3)
        OctopusPassed = $octopusStats.Passed
        OctopusMinMs = [math]::Round($octopusStats.MinMs, 3)
        OctopusMedianMs = [math]::Round($octopusStats.MedianMs, 3)
        OctopusP95Ms = [math]::Round($octopusStats.P95Ms, 3)
        OctopusMaxMs = [math]::Round($octopusStats.MaxMs, 3)
        AdapterMedianOverheadMs = [math]::Round($overheadMs, 3)
        AdapterMedianRatio = [math]::Round($overheadRatio, 3)
        DirectMedianCpuMs = [math]::Round($direct.MedianCpuMs, 3)
        DirectMedianPeakRssMiB = [math]::Round($direct.MedianPeakRssMiB, 3)
        OctopusParentMedianCpuMs = [math]::Round($octopusStats.MedianCpuMs, 3)
        OctopusParentMedianPeakRssMiB = [math]::Round($octopusStats.MedianPeakRssMiB, 3)
        DirectMedianStdoutBytes = [math]::Round($direct.MedianStdoutBytes, 0)
        OctopusMedianStdoutBytes = [math]::Round($octopusStats.MedianStdoutBytes, 0)
    })
}

$parallelRows = @()
if (-not $SkipParallel) {
    $parallelNames = @("hox-diff", "brain-connectome", "path-resonance", "mycelium-spread")
    $parallelCatalog = @($Catalog | Where-Object Name -in $parallelNames)
    $parallelRows = @(Invoke-ParallelScaling -Root (Join-Path $runRoot "parallel") -ReadCatalog $parallelCatalog)
}

$sampleCsv = Join-Path $runRoot "bio-benchmark-samples.csv"
$summaryCsv = Join-Path $runRoot "bio-benchmark-summary.csv"
$parallelCsv = Join-Path $runRoot "bio-benchmark-parallel.csv"
$reportPath = Join-Path $runRoot "bio-benchmark-report.md"

$measuredRows |
    Select-Object Name, Effect, Mode, Phase, Iteration, Order, Passed, ExitCode, TimedOut,
        WallMs, CpuMs, PeakWorkingSetMiB, StdoutBytes, StderrBytes, AdapterObserved,
        ArtifactPassed, Error, ContextRoot |
    Export-Csv -LiteralPath $sampleCsv -NoTypeInformation -Encoding utf8
$summary | Export-Csv -LiteralPath $summaryCsv -NoTypeInformation -Encoding utf8
if ($parallelRows.Count -gt 0) {
    $parallelRows | Export-Csv -LiteralPath $parallelCsv -NoTypeInformation -Encoding utf8
}

Save-RawEvidence -Rows @($allRows) -Directory $rawRoot -All:$KeepRawOutput

$failedModules = @($summary | Where-Object { -not $_.Passed })
$parallelFailures = @($parallelRows | Where-Object { $_.Passed -ne $_.Jobs })
$report = [System.Collections.Generic.List[string]]::new()
$report.Add("# Octopus Bio-Binaries benchmark")
$report.Add("")
$report.Add("- Run: ``$runId``")
$report.Add("- Host: ``$([System.Environment]::MachineName)`` / $([System.Environment]::OSVersion.VersionString)")
$report.Add("- Octopus: ``$Octopus``")
$report.Add("- Bio directory: ``$BioBinaryDir``")
$report.Add("- Protocol: $Warmup warmup + $Samples measured samples per mode, alternating direct/Octopus order")
$report.Add("- Timeout: $TimeoutSeconds seconds per process")
$report.Add("- Duration: $(Format-Number $benchmarkWatch.Elapsed.TotalSeconds) seconds (main 33-module matrix)")
$report.Add("- Result: $($summary.Count - $failedModules.Count)/$($summary.Count) modules passed")
$evidenceClass = if ($Samples -ge 20) { "paired benchmark" } else { "diagnostic smoke; fewer than 20 pairs" }
$report.Add("- Evidence class: $evidenceClass")
$report.Add("")
$report.Add("Pass/fail is functional: every warmup and measured process must exit 0, required artifacts must validate, and every Octopus run must emit its native adapter evidence marker. Latency has no arbitrary pass threshold.")
$report.Add("This harness measures small-fixture end-to-end process latency. It does not establish large-fixture algorithm throughput; use ``docs/BIO_BENCHMARK_METHODOLOGY.md`` for the full publication protocol.")
if ($Samples -lt 50) {
    $report.Add("Nearest-rank p95 values are exploratory because this run has fewer than 50 successful pairs per module; medians and paired deltas are the primary results.")
}
$report.Add("")
$report.Add("## Per-module wall-clock latency")
$report.Add("")
$report.Add("| Module | Effect | Pass | Direct median / p95 / min / max (ms) | Octopus median / p95 / min / max (ms) | Median adapter overhead | Ratio |")
$report.Add("|---|---:|:---:|---:|---:|---:|---:|")
foreach ($row in $summary) {
    $status = if ($row.Passed) { "yes" } else { "NO" }
    $directText = "$(Format-Number $row.DirectMedianMs) / $(Format-Number $row.DirectP95Ms) / $(Format-Number $row.DirectMinMs) / $(Format-Number $row.DirectMaxMs)"
    $octopusText = "$(Format-Number $row.OctopusMedianMs) / $(Format-Number $row.OctopusP95Ms) / $(Format-Number $row.OctopusMinMs) / $(Format-Number $row.OctopusMaxMs)"
    $overheadText = "$(Format-Number $row.AdapterMedianOverheadMs) ms"
    $ratioText = "$(Format-Number $row.AdapterMedianRatio)×"
    $report.Add("| $($row.Name) | $($row.Effect) | $status | $directText | $octopusText | $overheadText | $ratioText |")
}

$report.Add("")
$report.Add("## Launched-process resource observations")
$report.Add("")
$report.Add("| Module | Direct Bio CPU / peak RSS | Octopus parent CPU / peak RSS |")
$report.Add("|---|---:|---:|")
foreach ($row in $summary) {
    $report.Add("| $($row.Name) | $(Format-Number $row.DirectMedianCpuMs) ms / $(Format-Number $row.DirectMedianPeakRssMiB) MiB | $(Format-Number $row.OctopusParentMedianCpuMs) ms / $(Format-Number $row.OctopusParentMedianPeakRssMiB) MiB |")
}
$report.Add("")
$report.Add("> CPU and RSS are OS metrics for the launched PID only. Direct rows describe the Bio parent process. Octopus rows describe only the Octopus parent and exclude its Bio child; they are not comparable whole-tree resource measurements. Bio targets that launch their own descendants also exclude those descendants.")
$report.Add("")
$report.Add("## Functional interpretation limits")
$report.Add("")
$report.Add("- ``collective-sync`` uses an unreachable loopback endpoint here, so its row is failure-path latency rather than successful consensus throughput.")
$report.Add("- ``ribosome-synth`` measures deterministic generation planning/render validation; compilation and artifact publication are covered by the functional smoke, not this latency row.")
$report.Add("- ``wave-cryo-tx test`` performs a real CryoFrame -> BFSK WAV -> CryoFrame integrity roundtrip. ``wave-cryo-rx monitor`` remains a bounded timer-only surface because no live audio-capture backend is configured.")
$report.Add("- ``microscope-mem status`` is compatibility-wrapper latency, not persistent Microscope storage performance.")
$report.Add("- Host sensors and commands with deliberate sleeps report real end-to-end latency; their designed waits are not Octopus overhead.")

if ($parallelRows.Count -gt 0) {
    $report.Add("")
    $report.Add("## Compatible read-only Octopus parallel scaling")
    $report.Add("")
    $report.Add("Workload: hox-diff, brain-connectome, path-resonance, and mycelium-spread; $ParallelRepeats repetitions each. All jobs share one Octopus state root per degree and use isolated fixtures.")
    $report.Add("")
    $report.Add("| Parallelism | Passed jobs | Makespan (ms) | Throughput (jobs/s) | Speedup | Efficiency |")
    $report.Add("|---:|---:|---:|---:|---:|---:|")
    foreach ($row in $parallelRows) {
        $report.Add("| $($row.Degree) | $($row.Passed)/$($row.Jobs) | $(Format-Number $row.MakespanMs) | $(Format-Number $row.ThroughputPerSecond) | $(Format-Number $row.Speedup)× | $(Format-Number $row.EfficiencyPercent)% |")
    }
}

$failures = @($allRows | Where-Object { -not $_.Passed })
if ($failures.Count -gt 0 -or $parallelFailures.Count -gt 0) {
    $report.Add("")
    $report.Add("## Failures")
    $report.Add("")
    foreach ($failure in $failures) {
        $report.Add("- ``$($failure.Name)`` $($failure.Mode)/$($failure.Phase)/$($failure.Iteration): exit=$($failure.ExitCode), timeout=$($failure.TimedOut), adapter=$($failure.AdapterObserved), artifact=$($failure.ArtifactPassed), error=$($failure.Error)")
    }
    foreach ($failure in $parallelFailures) {
        $report.Add("- Parallel degree $($failure.Degree): $($failure.Passed)/$($failure.Jobs) jobs passed")
    }
}

$report.Add("")
$report.Add("## Files")
$report.Add("")
$report.Add("- ``bio-benchmark-samples.csv``: one measured process per row")
$report.Add("- ``bio-benchmark-summary.csv``: per-module latency and launched-PID resource summary")
$report.Add("- ``bio-benchmark-environment.txt``: machine and exact executable identity")
if ($parallelRows.Count -gt 0) { $report.Add("- ``bio-benchmark-parallel.csv``: compatible concurrency scaling") }
$report.Add("- ``raw/``: failure output, or every output with ``-KeepRawOutput``")
Set-Utf8File -Path $reportPath -Value (($report -join "`n") + "`n")

$summary | Select-Object Name, Effect, Passed, DirectMedianMs, OctopusMedianMs, AdapterMedianOverheadMs, AdapterMedianRatio | Format-Table -AutoSize
if ($parallelRows.Count -gt 0) { $parallelRows | Format-Table -AutoSize }
Write-Host "Bio benchmark: $($summary.Count - $failedModules.Count)/$($summary.Count) modules passed"
Write-Host "Report: $reportPath"
Write-Host "Samples: $sampleCsv"
Write-Host "Summary: $summaryCsv"
if ($parallelRows.Count -gt 0) { Write-Host "Parallel: $parallelCsv" }

if ($failedModules.Count -gt 0 -or $parallelFailures.Count -gt 0) {
    exit 1
}
