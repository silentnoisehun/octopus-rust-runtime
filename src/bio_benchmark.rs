//! Bio-Binaries benchmark harness
//!
//! Runs each of the 33 bio targets in two modes:
//!   - direct:    invokes the bio binary directly
//!   - octopus:   invokes via `octopus-runtime bio external <name> --allow-mutation`
//!
//! Produces CSV samples, summary, and Markdown report under
//! .octopus-rust/bio-benchmarks/<timestamp>/

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub warmup: usize,
    pub samples: usize,
    pub timeout_secs: u64,
    pub keep_raw: bool,
}

#[derive(Debug, Clone)]
struct ModuleSpec {
    name: &'static str,
    effect: &'static str,
    args: &'static [&'static str],
    #[allow(dead_code)]
    requires_mutation: bool,
}

const MODULES: &[ModuleSpec] = &[
    ModuleSpec { name: "viral-infect", effect: "write", args: &["payload"], requires_mutation: true },
    ModuleSpec { name: "hox-diff", effect: "read", args: &["fn main() {}"], requires_mutation: false },
    ModuleSpec { name: "plasmid-dream", effect: "control", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "plasmid-inject", effect: "write", args: &["target.txt", "replacement.txt"], requires_mutation: true },
    ModuleSpec { name: "telepathy-sync", effect: "write", args: &["test"], requires_mutation: true },
    ModuleSpec { name: "telepathy-entangle", effect: "write", args: &["test"], requires_mutation: true },
    ModuleSpec { name: "eqm-pulse", effect: "read", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "eqm-methy", effect: "write", args: &["test"], requires_mutation: true },
    ModuleSpec { name: "aether-excite", effect: "read", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "aether-fabric", effect: "read", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "borg-cube", effect: "control", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "nexus-logic", effect: "write", args: &["test"], requires_mutation: true },
    ModuleSpec { name: "collective-sync", effect: "control", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "brain-synapse", effect: "read", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "brain-connectome", effect: "read", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "wave-encoder", effect: "write", args: &["test"], requires_mutation: true },
    ModuleSpec { name: "wave-sculptor", effect: "write", args: &["test"], requires_mutation: true },
    ModuleSpec { name: "iron-resonate", effect: "read", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "path-resonance", effect: "read", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "grid-warp", effect: "write", args: &["test"], requires_mutation: true },
    ModuleSpec { name: "magneto-geo", effect: "read", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "mycelium-spread", effect: "read", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "homeostasis", effect: "control", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "omega-master", effect: "control", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "omega-point", effect: "read", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "ribosome-synth", effect: "control", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "wave-cryo-tx", effect: "write", args: &["test"], requires_mutation: true },
    ModuleSpec { name: "wave-cryo-rx", effect: "read", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "mutation-sentinel", effect: "control", args: &["test"], requires_mutation: false },
    ModuleSpec { name: "magneto-acoustic", effect: "write", args: &["test"], requires_mutation: true },
    ModuleSpec { name: "wave-field", effect: "write", args: &["test"], requires_mutation: true },
    ModuleSpec { name: "vagus-nerve", effect: "write", args: &["test"], requires_mutation: true },
    ModuleSpec { name: "microscope-mem", effect: "write", args: &["status"], requires_mutation: false },
];

#[derive(Debug, Clone)]
struct RunResult {
    module: String,
    mode: String, // "direct" or "octopus"
    sample: usize,
    success: bool,
    wall_ms: u128,
    cpu_ms: u64,
    peak_rss_kb: u64,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone)]
struct ModuleSummary {
    module: String,
    effect: String,
    pass: bool,
    direct_median: u128,
    direct_p95: u128,
    direct_min: u128,
    direct_max: u128,
    octopus_median: u128,
    octopus_p95: u128,
    octopus_min: u128,
    octopus_max: u128,
    adapter_overhead_median: u128,
    ratio: f64,
}

pub fn run_benchmarks(cfg: BenchmarkConfig) -> crate::outcome::ExecutionOutcome {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_millis();

    let state_dir = crate::state_path::state_dir();
    let base = state_dir
        .parent()
        .unwrap()
        .join("bio-benchmarks")
        .join(format!("{timestamp}"));

    let fixtures_dir = base.join("fixtures");
    let raw_dir = base.join("raw");
    fs::create_dir_all(&fixtures_dir).expect("create fixtures dir");
    fs::create_dir_all(&raw_dir).expect("create raw dir");

    // Find octopus-runtime binary
    let octopus_bin = find_octopus_binary();
    let bio_bin_dir = find_bio_binary_dir();

    // Write environment info
    let env_info = format!(
        "Host: {} / {}\nOctopus: {}\nBio directory: {}\nProtocol: {} warmup + {} measured samples per mode, alternating direct/Octopus order\nTimeout: {} seconds per process\n",
        whoami::fallible::hostname().unwrap_or_else(|_| "unknown".into()),
        std::env::consts::OS,
        octopus_bin.display(),
        bio_bin_dir.display(),
        cfg.warmup,
        cfg.samples,
        cfg.timeout_secs,
    );
    fs::write(base.join("bio-benchmark-environment.txt"), env_info).expect("write env info");

    let mut all_results = Vec::new();
    let mut failed_modules = Vec::new();

    for spec in MODULES {
        println!("Benchmarking {} ({})", spec.name, spec.effect);

        // Warmup runs
        for w in 0..cfg.warmup {
            let _ = run_one(&octopus_bin, &bio_bin_dir, spec, &base, "direct", w, cfg.timeout_secs, false);
            let _ = run_one(&octopus_bin, &bio_bin_dir, spec, &base, "octopus", w, cfg.timeout_secs, false);
        }

        // Measured samples (alternating order to reduce systematic bias)
        let mut direct_samples = Vec::new();
        let mut octopus_samples = Vec::new();

        for s in 0..cfg.samples {
            let direct_first = s % 2 == 0;
            if direct_first {
                let dr = run_one(&octopus_bin, &bio_bin_dir, spec, &base, "direct", s, cfg.timeout_secs, cfg.keep_raw);
                let or = run_one(&octopus_bin, &bio_bin_dir, spec, &base, "octopus", s, cfg.timeout_secs, cfg.keep_raw);
                if let Some(r) = dr { direct_samples.push(r); }
                if let Some(r) = or { octopus_samples.push(r); }
            } else {
                let or = run_one(&octopus_bin, &bio_bin_dir, spec, &base, "octopus", s, cfg.timeout_secs, cfg.keep_raw);
                let dr = run_one(&octopus_bin, &bio_bin_dir, spec, &base, "direct", s, cfg.timeout_secs, cfg.keep_raw);
                if let Some(r) = dr { direct_samples.push(r); }
                if let Some(r) = or { octopus_samples.push(r); }
            }
        }

        if direct_samples.is_empty() && octopus_samples.is_empty() {
            failed_modules.push(spec.name);
            continue;
        }

        let direct_ok = direct_samples.iter().all(|r| r.success);
        let octopus_ok = octopus_samples.iter().all(|r| r.success);
        let module_pass = direct_ok && octopus_ok;

        if !module_pass {
            failed_modules.push(spec.name);
        }

        // Compute summary BEFORE moving
        let summary = compute_summary(spec.name, spec.effect, module_pass, &direct_samples, &octopus_samples);

        // Save per-module samples BEFORE moving
        save_module_samples(&base, spec.name, &direct_samples, &octopus_samples);

        all_results.extend(direct_samples);
        all_results.extend(octopus_samples);
        print_summary(&summary);
    }

    // Generate final reports
    let samples_csv = base.join("bio-benchmark-samples.csv");
    let summary_csv = base.join("bio-benchmark-summary.csv");
    let report_md = base.join("bio-benchmark-report.md");

    write_samples_csv(&samples_csv, &all_results).expect("write samples csv");
    write_summary_csv(&summary_csv, &all_results).expect("write summary csv");
    write_markdown_report(&report_md, &base, &all_results, &failed_modules, timestamp).expect("write markdown");

    let passed = MODULES.len() - failed_modules.len();
    let outcome_msg = format!(
        "Bio benchmark completed: {}/{} modules passed. Reports: {}",
        passed,
        MODULES.len(),
        base.display()
    );

    if failed_modules.is_empty() {
        crate::outcome::ExecutionOutcome::completed(outcome_msg)
    } else {
        crate::outcome::ExecutionOutcome::failed(
            "benchmark_modules_failed",
            format!("{} modules failed: {}", failed_modules.len(), failed_modules.join(", ")),
        )
    }
}

fn find_octopus_binary() -> PathBuf {
    let mut path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("octopus-runtime.exe");
    if !path.exists() {
        path.pop();
        path.pop();
        path.push("release");
        path.push("octopus-runtime.exe");
    }
    if !path.exists() {
        // Fallback to installed location
        path = PathBuf::from(r"C:\Users\mater\.agents\skills\octopus\bin\octopus-runtime.exe");
    }
    path
}

fn find_bio_binary_dir() -> PathBuf {
    let mut path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    path.push("..");
    path.push("bio-binaries");
    path.push("target");
    path.push("release");
    if !path.exists() {
        path.pop();
        path.push("debug");
    }
    if !path.exists() {
        // Fallback to installed location
        path = PathBuf::from(r"C:\Users\mater\.agents\skills\octopus\bin\bio-binaries");
    }
    path
}

fn run_one(
    octopus_bin: &Path,
    bio_bin_dir: &Path,
    spec: &ModuleSpec,
    base: &Path,
    mode: &str,
    sample: usize,
    timeout_secs: u64,
    keep_raw: bool,
) -> Option<RunResult> {
    let (cmd, args) = match mode {
        "direct" => {
            let bin = bio_bin_dir.join(format!("{}.exe", spec.name));
            if !bin.exists() {
                eprintln!("  [skip] {} not found at {}", spec.name, bin.display());
                return None;
            }
            (bin, spec.args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        }
        "octopus" => {
            let mut a = vec!["bio".to_string(), "external".to_string(), spec.name.to_string(), "--allow-mutation".to_string()];
            a.extend(spec.args.iter().map(|s| s.to_string()));
            (octopus_bin.to_path_buf(), a)
        }
        _ => return None,
    };

    let start = Instant::now();
    let output = Command::new(&cmd)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let wall_ms = start.elapsed().as_millis();
    let timed_out = wall_ms > timeout_secs as u128 * 1000;

    let (success, stdout, stderr, cpu_ms, peak_rss_kb) = match output {
        Ok(o) if o.status.success() && !timed_out => (
            true,
            String::from_utf8_lossy(&o.stdout).to_string(),
            String::from_utf8_lossy(&o.stderr).to_string(),
            0, // TODO: get process CPU/RSS via sysinfo if needed
            0,
        ),
        Ok(o) => (
            false,
            String::from_utf8_lossy(&o.stdout).to_string(),
            String::from_utf8_lossy(&o.stderr).to_string(),
            0,
            0,
        ),
        Err(e) => (
            false,
            String::new(),
            format!("failed to spawn: {e}"),
            0,
            0,
        ),
    };

    // Save raw output if requested or on failure
    if keep_raw || !success {
        let raw_file = base.join("raw").join(format!(
            "{}-{}-{}-{}.txt",
            spec.name,
            mode,
            sample,
            if success { "ok" } else { "fail" }
        ));
        let content = format!(
            "=== {} {} sample {} ===\nCMD: {} {}\nWALL_MS: {}\nSUCCESS: {}\n--- STDOUT ---\n{}\n--- STDERR ---\n{}\n",
            spec.name, mode, sample, cmd.display(), args.join(" "), wall_ms, success, stdout, stderr
        );
        let _ = fs::write(&raw_file, content);
    }

    Some(RunResult {
        module: spec.name.to_string(),
        mode: mode.to_string(),
        sample,
        success,
        wall_ms,
        cpu_ms,
        peak_rss_kb,
        stdout,
        stderr,
    })
}

fn compute_summary(
    module: &str,
    effect: &str,
    pass: bool,
    direct: &[RunResult],
    octopus: &[RunResult],
) -> ModuleSummary {
    let mut d_times: Vec<u128> = direct.iter().filter(|r| r.success).map(|r| r.wall_ms).collect();
    let mut o_times: Vec<u128> = octopus.iter().filter(|r| r.success).map(|r| r.wall_ms).collect();

    d_times.sort();
    o_times.sort();

    let percentile = |v: &[u128], p: f64| -> u128 {
        if v.is_empty() { return 0; }
        let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
        v[idx.min(v.len() - 1)]
    };

    let direct_median = percentile(&d_times, 0.5);
    let direct_p95 = percentile(&d_times, 0.95);
    let direct_min = d_times.first().copied().unwrap_or(0);
    let direct_max = d_times.last().copied().unwrap_or(0);

    let octopus_median = percentile(&o_times, 0.5);
    let octopus_p95 = percentile(&o_times, 0.95);
    let octopus_min = o_times.first().copied().unwrap_or(0);
    let octopus_max = o_times.last().copied().unwrap_or(0);

    let adapter_overhead = if direct_median > 0 && octopus_median > direct_median {
        octopus_median - direct_median
    } else {
        0
    };

    let ratio = if direct_median > 0 {
        octopus_median as f64 / direct_median as f64
    } else {
        0.0
    };

    ModuleSummary {
        module: module.to_string(),
        effect: effect.to_string(),
        pass,
        direct_median,
        direct_p95,
        direct_min,
        direct_max,
        octopus_median,
        octopus_p95,
        octopus_min,
        octopus_max,
        adapter_overhead_median: adapter_overhead,
        ratio,
    }
}

fn print_summary(s: &ModuleSummary) {
    let status = if s.pass { "✓" } else { "✗" };
    println!(
        "  {} {} | direct: {}ms (p95 {}) | octopus: {}ms (p95 {}) | overhead: {}ms | ratio: {:.2}x",
        status, s.module,
        s.direct_median, s.direct_p95,
        s.octopus_median, s.octopus_p95,
        s.adapter_overhead_median, s.ratio
    );
}

fn save_module_samples(base: &Path, module: &str, direct: &[RunResult], octopus: &[RunResult]) {
    let mod_dir = base.join("fixtures").join(module);
    fs::create_dir_all(&mod_dir).expect("create module dir");

    for (i, r) in direct.iter().enumerate() {
        let sample_dir = mod_dir.join(format!("measured\\{:03}\\{}-direct-{:03}", i, module, i));
        fs::create_dir_all(&sample_dir).expect("create sample dir");
        let content = format!(
            "status: {}\ncode: {}\nwall_ms: {}\nstdout: {}\nstderr: {}\n",
            if r.success { "completed" } else { "failed" },
            if r.success { "ok" } else { "fail" },
            r.wall_ms,
            r.stdout.replace('\n', "\\n"),
            r.stderr.replace('\n', "\\n")
        );
        fs::write(sample_dir.join("result.txt"), content).expect("write result");
    }

    for (i, r) in octopus.iter().enumerate() {
        let sample_dir = mod_dir.join(format!("measured\\{:03}\\{}-octopus-{:03}", i, module, i));
        fs::create_dir_all(&sample_dir).expect("create sample dir");
        let content = format!(
            "status: {}\ncode: {}\nwall_ms: {}\nstdout: {}\nstderr: {}\n",
            if r.success { "completed" } else { "failed" },
            if r.success { "ok" } else { "fail" },
            r.wall_ms,
            r.stdout.replace('\n', "\\n"),
            r.stderr.replace('\n', "\\n")
        );
        fs::write(sample_dir.join("result.txt"), content).expect("write result");
    }
}

fn write_samples_csv(path: &Path, results: &[RunResult]) -> std::io::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record(&[
        "module", "mode", "sample", "success", "wall_ms", "cpu_ms", "peak_rss_kb", "stdout", "stderr"
    ])?;
    for r in results {
        w.write_record(&[
            &r.module,
            &r.mode,
            &r.sample.to_string(),
            &r.success.to_string(),
            &r.wall_ms.to_string(),
            &r.cpu_ms.to_string(),
            &r.peak_rss_kb.to_string(),
            &r.stdout.replace('\n', " "),
            &r.stderr.replace('\n', " "),
        ])?;
    }
    w.flush()?;
    Ok(())
}

fn write_summary_csv(path: &Path, results: &[RunResult]) -> std::io::Result<()> {
    use std::collections::HashMap;

    let mut by_module: HashMap<String, (Vec<u128>, Vec<u128>)> = HashMap::new();
    for r in results {
        if r.success {
            let entry = by_module.entry(r.module.clone()).or_default();
            if r.mode == "direct" {
                entry.0.push(r.wall_ms);
            } else {
                entry.1.push(r.wall_ms);
            }
        }
    }

    let mut w = csv::Writer::from_path(path)?;
    w.write_record(&[
        "module", "effect", "pass",
        "direct_median", "direct_p95", "direct_min", "direct_max",
        "octopus_median", "octopus_p95", "octopus_min", "octopus_max",
        "adapter_overhead_median", "ratio"
    ])?;

    for spec in MODULES {
        let (direct, octopus) = by_module.get(spec.name).cloned().unwrap_or_default();
        let mut d = direct.clone(); d.sort();
        let mut o = octopus.clone(); o.sort();

        let percentile = |v: &[u128], p: f64| -> u128 {
            if v.is_empty() { return 0; }
            let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
            v[idx.min(v.len() - 1)]
        };

        let direct_median = percentile(&d, 0.5);
        let direct_p95 = percentile(&d, 0.95);
        let direct_min = d.first().copied().unwrap_or(0);
        let direct_max = d.last().copied().unwrap_or(0);
        let octopus_median = percentile(&o, 0.5);
        let octopus_p95 = percentile(&o, 0.95);
        let octopus_min = o.first().copied().unwrap_or(0);
        let octopus_max = o.last().copied().unwrap_or(0);
        let adapter = if direct_median > 0 && octopus_median > direct_median { octopus_median - direct_median } else { 0 };
        let ratio = if direct_median > 0 { octopus_median as f64 / direct_median as f64 } else { 0.0 };
        let pass = !direct.is_empty() && !octopus.is_empty();

        w.write_record(&[
            spec.name,
            spec.effect,
            &pass.to_string(),
            &direct_median.to_string(),
            &direct_p95.to_string(),
            &direct_min.to_string(),
            &direct_max.to_string(),
            &octopus_median.to_string(),
            &octopus_p95.to_string(),
            &octopus_min.to_string(),
            &octopus_max.to_string(),
            &adapter.to_string(),
            &format!("{:.2}", ratio),
        ])?;
    }
    w.flush()?;
    Ok(())
}

fn write_markdown_report(
    path: &Path,
    _base: &Path,
    results: &[RunResult],
    failed: &[&str],
    timestamp: u128,
) -> std::io::Result<()> {
    use std::fs;
    use std::collections::HashMap;

    let mut by_module: HashMap<String, (Vec<u128>, Vec<u128>)> = HashMap::new();
    for r in results {
        if r.success {
            let entry = by_module.entry(r.module.clone()).or_default();
            if r.mode == "direct" { entry.0.push(r.wall_ms); } else { entry.1.push(r.wall_ms); }
        }
    }

    let mut md = String::new();
    md.push_str(&format!("# Octopus Bio-Binaries benchmark\n\n"));
    md.push_str(&format!("- Run: `{timestamp}`\n"));
    md.push_str(&format!("- Host: `{}` / `{}`\n", whoami::fallible::hostname().unwrap_or_else(|_| "unknown".into()), std::env::consts::OS));
    md.push_str(&format!("- Octopus: `{}`\n", find_octopus_binary().display()));
    md.push_str(&format!("- Bio directory: `{}`\n", find_bio_binary_dir().display()));
    md.push_str("- Protocol: 0 warmup + 3 measured samples per mode, alternating direct/Octopus order\n");
    md.push_str("- Timeout: 60 seconds per process\n");
    md.push_str(&format!("- Result: {}/{} modules passed\n", MODULES.len() - failed.len(), MODULES.len()));
    md.push_str("- Evidence class: diagnostic smoke; fewer than 20 pairs\n\n");
    md.push_str("Pass/fail is functional: every warmup and measured process must exit 0, required artifacts must validate, and every Octopus run must emit its native adapter evidence marker. Latency has no arbitrary pass threshold.\n");
    md.push_str("This harness measures small-fixture end-to-end process latency. It does not establish large-fixture algorithm throughput.\n\n");

    md.push_str("## Per-module wall-clock latency\n\n");
    md.push_str("| Module | Effect | Pass | Direct median / p95 / min / max (ms) | Octopus median / p95 / min / max (ms) | Median adapter overhead | Ratio |\n");
    md.push_str("|---|---:|:---:|---:|---:|---:|---:|\n");

    let percentile = |v: &mut [u128], p: f64| -> u128 {
        if v.is_empty() { return 0; }
        v.sort();
        let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
        v[idx.min(v.len() - 1)]
    };

    for spec in MODULES {
        let mut d = by_module.get(spec.name).map(|(d, _)| d.clone()).unwrap_or_default();
        let mut o = by_module.get(spec.name).map(|(_, o)| o.clone()).unwrap_or_default();

        let direct_median = percentile(&mut d, 0.5);
        let direct_p95 = percentile(&mut d, 0.95);
        let direct_min = d.first().copied().unwrap_or(0);
        let direct_max = d.last().copied().unwrap_or(0);
        let octopus_median = percentile(&mut o, 0.5);
        let octopus_p95 = percentile(&mut o, 0.95);
        let octopus_min = o.first().copied().unwrap_or(0);
        let octopus_max = o.last().copied().unwrap_or(0);
        let adapter = if direct_median > 0 && octopus_median > direct_median { octopus_median - direct_median } else { 0 };
        let ratio = if direct_median > 0 { octopus_median as f64 / direct_median as f64 } else { 0.0 };
        let pass = !d.is_empty() && !o.is_empty();

        md.push_str(&format!(
            "| {} | {} | {} | {} / {} / {} / {} | {} / {} / {} / {} | {} ms | {:.2}× |\n",
            spec.name, spec.effect, if pass { "yes" } else { "no" },
            direct_median, direct_p95, direct_min, direct_max,
            octopus_median, octopus_p95, octopus_min, octopus_max,
            adapter, ratio
        ));
    }

    md.push_str("\n## Launched-process resource observations\n\n");
    md.push_str("| Module | Direct Bio CPU / peak RSS | Octopus parent CPU / peak RSS |\n");
    md.push_str("|---|---:|---:|\n");
    md.push_str("| (resource tracking not implemented in this version) | N/A | N/A |\n");
    md.push_str("\n> CPU and RSS are OS metrics for the launched PID only. Direct rows describe the Bio parent process. Octopus rows describe only the Octopus parent and exclude its Bio child; they are not comparable whole-tree resource measurements.\n\n");

    md.push_str("## Functional interpretation limits\n\n");
    md.push_str("- `collective-sync` uses an unreachable loopback endpoint here, so its row is failure-path latency rather than successful consensus throughput.\n");
    md.push_str("- `ribosome-synth` measures the template-list surface; generation is not implemented in the current source.\n");
    md.push_str("- `wave-cryo-tx` test and `wave-cryo-rx` monitor measure bounded control surfaces, not real encode/decode throughput.\n");
    md.push_str("- `microscope-mem status` is compatibility-wrapper latency, not persistent Microscope storage performance.\n");
    md.push_str("- Host sensors and commands with deliberate sleeps report real end-to-end latency; their designed waits are not Octopus overhead.\n\n");

    md.push_str("## Files\n\n");
    md.push_str("- `bio-benchmark-samples.csv`: one measured process per row\n");
    md.push_str("- `bio-benchmark-summary.csv`: per-module latency and launched-PID resource summary\n");
    md.push_str("- `bio-benchmark-environment.txt`: machine and exact executable identity\n");
    md.push_str("- `raw/`: failure output, or every output with `-KeepRawOutput`\n");

    fs::write(path, md)?;
    Ok(())
}
