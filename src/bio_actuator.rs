use crate::process::{self, ProcessSpec};
use crate::state_path::{sidecar_path, state_dir};
use crate::ExecutionOutcome;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_PATCH_BYTES: u64 = 64 * 1024 * 1024;
const MICROSCOPE_FILES: &[&str] = &[
    "meta.bin",
    "microscope.bin",
    "data.bin",
    "activations.bin",
    "coactivations.bin",
    "fingerprints.bin",
    "thought_graph.bin",
    "thought_patterns.bin",
    "predictive_cache.bin",
    "resonance.bin",
    "dream_log.bin",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessIdentity {
    pid: u32,
    name: String,
    start_ticks: u64,
    path: PathBuf,
    parent_pid: u32,
    executable_hash: String,
}

#[derive(Debug, Clone)]
struct SynapticPlan {
    executable: PathBuf,
    config: PathBuf,
    data_dir: PathBuf,
    archive_dir: PathBuf,
    fingerprint: String,
    executable_hash: String,
    config_hash: String,
    confirmation: String,
}

#[derive(Debug, Clone)]
struct CrisprPlan {
    target: PathBuf,
    replacement: PathBuf,
    target_hash: String,
    replacement_hash: String,
    backup: PathBuf,
    health_args: Vec<String>,
    confirmation: String,
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn sha256(value: impl AsRef<[u8]>) -> String {
    hex::encode(Sha256::digest(value.as_ref()))
}

fn hash_file(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(sha256)
        .map_err(|error| format!("cannot hash {}: {error}", path.display()))
}

fn confirmation(prefix: &str, payload: &str) -> String {
    format!("{prefix}-{}", &sha256(payload)[..32])
}

fn run_endurance_guard() -> Result<(), String> {
    let configured = env::var_os("OCTOPUS_ENDURANCE_GUARD")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"D:\codex\octopus-runtime-ui\manage-endurance-soak.ps1"));
    run_endurance_guard_at(&configured)
}

fn run_endurance_guard_at(configured: &Path) -> Result<(), String> {
    if !configured.is_file() {
        return Err(format!(
            "endurance guard is missing or is not a regular file: {}",
            configured.display()
        ));
    }
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(configured)
        .arg("Guard")
        .output()
        .map_err(|error| format!("cannot run endurance guard: {error}"))?;
    match output.status.code() {
        Some(0) => Ok(()),
        Some(3) => Err("active endurance lease blocks the actuator".to_string()),
        code => Err(format!(
            "endurance guard failed with exit code {}: {}",
            code.unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr).trim()
        )),
    }
}

#[cfg(windows)]
fn query_process(pid: u32) -> Result<ProcessIdentity, String> {
    let script = format!(
        "$ErrorActionPreference='Stop'; $id=[uint32]{pid}; \
         $w=Get-CimInstance Win32_Process -Filter ('ProcessId = ' + $id); \
         if($null -eq $w){{exit 3}}; $p=Get-Process -Id $id; \
         $ticks=$p.StartTime.ToUniversalTime().Ticks; \
         $fields=@($p.Id.ToString(),$p.ProcessName,$ticks.ToString(),$p.Path,$w.ParentProcessId.ToString()); \
         [Console]::Out.WriteLine(($fields -join \"`t\"))"
    );
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .map_err(|error| format!("cannot inspect PID {pid}: {error}"))?;
    if output.status.code() == Some(3) {
        return Err(format!("PID {pid} does not exist"));
    }
    if !output.status.success() {
        return Err(format!(
            "cannot inspect PID {pid}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let fields: Vec<_> = stdout.trim().split('\t').collect();
    if fields.len() != 5 {
        return Err(format!("PID {pid} returned an invalid identity receipt"));
    }
    let path = fs::canonicalize(fields[3])
        .map_err(|error| format!("cannot canonicalize process executable: {error}"))?;
    Ok(ProcessIdentity {
        pid: fields[0]
            .parse()
            .map_err(|_| "invalid process PID receipt".to_string())?,
        name: fields[1].to_string(),
        start_ticks: fields[2]
            .parse()
            .map_err(|_| "invalid process start time receipt".to_string())?,
        executable_hash: hash_file(&path)?,
        path,
        parent_pid: fields[4]
            .parse()
            .map_err(|_| "invalid parent PID receipt".to_string())?,
    })
}

#[cfg(not(windows))]
fn query_process(_pid: u32) -> Result<ProcessIdentity, String> {
    Err("macrophage process actuator is currently Windows-only".to_string())
}

fn protected_process(identity: &ProcessIdentity) -> Result<(), String> {
    if identity.pid == std::process::id() || identity.pid <= 4 {
        return Err("refusing to target the actuator or a kernel PID".to_string());
    }
    let protected = [
        "system",
        "registry",
        "smss",
        "csrss",
        "wininit",
        "services",
        "lsass",
        "winlogon",
        "dwm",
        "octopus-runtime",
        "octopus-runtime-core",
        "microscope-mem",
    ];
    if protected
        .iter()
        .any(|name| identity.name.eq_ignore_ascii_case(name))
    {
        return Err(format!("protected process name: {}", identity.name));
    }
    if let Some(system_root) = env::var_os("SystemRoot") {
        let system_root = system_root.to_string_lossy().to_lowercase();
        if identity
            .path
            .to_string_lossy()
            .to_lowercase()
            .starts_with(&system_root)
        {
            return Err(format!(
                "process executable is inside the protected Windows root: {}",
                identity.path.display()
            ));
        }
    }
    Ok(())
}

fn macrophage_token(identity: &ProcessIdentity) -> String {
    confirmation(
        "MAC",
        &format!(
            "v1|{}|{}|{}|{}|{}|{}",
            identity.pid,
            identity.name,
            identity.start_ticks,
            identity.path.display(),
            identity.parent_pid,
            identity.executable_hash
        ),
    )
}

pub fn macrophage_plan(pid: u32) -> ExecutionOutcome {
    let identity = match query_process(pid) {
        Ok(identity) => identity,
        Err(error) => return ExecutionOutcome::failed("macrophage_inspection_failed", error),
    };
    if let Err(error) = protected_process(&identity) {
        return ExecutionOutcome::failed("macrophage_target_protected", error);
    }
    let token = macrophage_token(&identity);
    ExecutionOutcome::completed(format!(
        "MACROPHAGE PLAN\nmode: dry-run\npid: {}\nname: {}\nparent-pid: {}\nstart-ticks: {}\npath: {}\nsha256: {}\naction: terminate exact PID only\nconfirm: {}",
        identity.pid,
        identity.name,
        identity.parent_pid,
        identity.start_ticks,
        identity.path.display(),
        identity.executable_hash,
        token
    ))
}

#[cfg(windows)]
fn terminate_exact_process(identity: &ProcessIdentity) -> Result<(), String> {
    // Keep one System.Diagnostics.Process object alive from identity
    // revalidation through termination. This binds the destructive action to
    // the inspected process handle instead of issuing a second PID-only kill.
    let script = "$ErrorActionPreference='Stop'; \
         $id=[uint32]$env:OCTOPUS_EXPECTED_PID; \
         $p=Get-Process -Id $id -ErrorAction Stop; \
         $start=$p.StartTime.ToUniversalTime().Ticks.ToString(); \
         $path=[IO.Path]::GetFullPath($p.Path); \
         $hash=(Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant(); \
         if($p.ProcessName -cne $env:OCTOPUS_EXPECTED_NAME -or \
            $start -cne $env:OCTOPUS_EXPECTED_START -or \
            $path -ine $env:OCTOPUS_EXPECTED_PATH -or \
            $hash -cne $env:OCTOPUS_EXPECTED_HASH){exit 5}; \
         $p.Kill(); \
         if(-not $p.WaitForExit(5000)){exit 4}";
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("OCTOPUS_EXPECTED_PID", identity.pid.to_string())
        .env("OCTOPUS_EXPECTED_NAME", &identity.name)
        .env("OCTOPUS_EXPECTED_START", identity.start_ticks.to_string())
        .env("OCTOPUS_EXPECTED_PATH", &identity.path)
        .env("OCTOPUS_EXPECTED_HASH", &identity.executable_hash)
        .output()
        .map_err(|error| format!("cannot terminate PID {}: {error}", identity.pid))?;
    if output.status.success() {
        Ok(())
    } else if output.status.code() == Some(5) {
        Err("process identity changed after confirmation; refusing termination".to_string())
    } else {
        Err(format!(
            "PID {} termination failed: {}",
            identity.pid,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(not(windows))]
fn terminate_exact_process(_identity: &ProcessIdentity) -> Result<(), String> {
    Err("macrophage process actuator is currently Windows-only".to_string())
}

fn append_antigen(identity: &ProcessIdentity, token: &str, state: &str) -> Result<PathBuf, String> {
    let path = sidecar_path(&state_dir(), "antigen.log")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create antigen directory: {error}"))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("cannot open antigen log: {error}"))?;
    writeln!(
        file,
        "{}\tstate={}\tpid={}\tname={}\tstart={}\tpath-sha256={}\tconfirmation-sha256={}",
        now_millis(),
        state,
        identity.pid,
        identity.name,
        identity.start_ticks,
        identity.executable_hash,
        sha256(token)
    )
    .map_err(|error| format!("cannot append antigen log: {error}"))?;
    file.sync_data()
        .map_err(|error| format!("cannot sync antigen log: {error}"))?;
    Ok(path)
}

pub fn macrophage_apply(pid: u32, confirm: &str, allow_kill: bool) -> ExecutionOutcome {
    if !allow_kill {
        return ExecutionOutcome::failed(
            "macrophage_permission_required",
            "macrophage apply requires --allow-kill",
        );
    }
    if let Err(error) = run_endurance_guard() {
        return ExecutionOutcome::failed("endurance_lease_active", error);
    }
    let identity = match query_process(pid) {
        Ok(identity) => identity,
        Err(error) => return ExecutionOutcome::failed("macrophage_inspection_failed", error),
    };
    if let Err(error) = protected_process(&identity) {
        return ExecutionOutcome::failed("macrophage_target_protected", error);
    }
    let expected = macrophage_token(&identity);
    if confirm != expected {
        return ExecutionOutcome::failed(
            "macrophage_confirmation_mismatch",
            "process identity changed or confirmation token is invalid; run plan again",
        );
    }
    let antigen = match append_antigen(&identity, confirm, "prepared") {
        Ok(path) => path,
        Err(error) => return ExecutionOutcome::failed("macrophage_antigen_log_failed", error),
    };
    if let Err(error) = terminate_exact_process(&identity) {
        let _ = append_antigen(&identity, confirm, "termination-failed");
        return ExecutionOutcome::failed("macrophage_termination_failed", error);
    }
    let audit = match append_antigen(&identity, confirm, "terminated") {
        Ok(_) => "sealed".to_string(),
        Err(error) => format!("finalize-warning: {error}"),
    };
    ExecutionOutcome::completed(format!(
        "MACROPHAGE APPLY\nstatus: terminated\npid: {}\nname: {}\nstart-ticks: {}\npath-sha256: {}\nantigen-log: {}\nantigen-audit: {}\nidentity-revalidated: true",
        identity.pid,
        identity.name,
        identity.start_ticks,
        identity.executable_hash,
        antigen.display(),
        audit
    ))
}

fn parse_output_dir(config: &Path) -> Result<PathBuf, String> {
    let content = fs::read_to_string(config)
        .map_err(|error| format!("cannot read microscope config: {error}"))?;
    let mut in_paths = false;
    for raw in content.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_paths = line == "[paths]";
            continue;
        }
        if in_paths {
            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == "output_dir" {
                    let value = value.trim().trim_matches('"');
                    let path = PathBuf::from(value);
                    return if path.is_absolute() {
                        Ok(path)
                    } else {
                        Ok(config.parent().unwrap_or_else(|| Path::new(".")).join(path))
                    };
                }
            }
        }
    }
    Err("microscope config has no [paths].output_dir".to_string())
}

fn data_fingerprint(data_dir: &Path) -> Result<String, String> {
    let mut payload = String::new();
    for name in MICROSCOPE_FILES {
        let path = data_dir.join(name);
        if path.is_file() {
            let metadata = fs::metadata(&path)
                .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
            payload.push_str(&format!(
                "{name}|{}|{}\n",
                metadata.len(),
                hash_file(&path)?
            ));
        }
    }
    if payload.is_empty() {
        return Err(format!(
            "no supported Microscope state files found in {}",
            data_dir.display()
        ));
    }
    Ok(sha256(payload))
}

fn build_synaptic_plan(executable: &Path, config: &Path) -> Result<SynapticPlan, String> {
    let executable = fs::canonicalize(executable)
        .map_err(|error| format!("invalid microscope executable: {error}"))?;
    let config =
        fs::canonicalize(config).map_err(|error| format!("invalid microscope config: {error}"))?;
    let data_dir = fs::canonicalize(parse_output_dir(&config)?)
        .map_err(|error| format!("invalid microscope data directory: {error}"))?;
    let fingerprint = data_fingerprint(&data_dir)?;
    let executable_hash = hash_file(&executable)?;
    let config_hash = hash_file(&config)?;
    let archive_parent = data_dir
        .parent()
        .ok_or_else(|| "microscope data directory has no parent".to_string())?
        .join("archives");
    let archive_dir = archive_parent.join(format!("synaptic-{}", &fingerprint[..16]));
    let confirmation = confirmation(
        "SYN",
        &format!(
            "v1|{}|{}|{}|{}|{}",
            executable.display(),
            executable_hash,
            config.display(),
            config_hash,
            fingerprint
        ),
    );
    Ok(SynapticPlan {
        executable,
        config,
        data_dir,
        archive_dir,
        fingerprint,
        executable_hash,
        config_hash,
        confirmation,
    })
}

pub fn synaptic_plan(executable: &Path, config: &Path) -> ExecutionOutcome {
    match build_synaptic_plan(executable, config) {
        Ok(plan) => ExecutionOutcome::completed(format!(
            "SYNAPTIC PLAN\nmode: dry-run\nexecutable: {}\nexecutable-sha256: {}\nconfig: {}\nconfig-sha256: {}\ndata: {}\nstate-fingerprint: {}\narchive: {}\naction: archive -> dream -> CRC -> Merkle\nconfirm: {}",
            plan.executable.display(),
            plan.executable_hash,
            plan.config.display(),
            plan.config_hash,
            plan.data_dir.display(),
            plan.fingerprint,
            plan.archive_dir.display(),
            plan.confirmation
        )),
        Err(error) => ExecutionOutcome::failed("synaptic_plan_failed", error),
    }
}

fn write_synaptic_archive(plan: &SynapticPlan) -> Result<(), String> {
    if plan.archive_dir.exists() {
        return verify_synaptic_archive(plan).map(|_| ());
    }
    let parent = plan
        .archive_dir
        .parent()
        .ok_or_else(|| "synaptic archive has no parent".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create synaptic archive parent: {error}"))?;
    let partial = parent.join(format!(
        ".synaptic-{}-{}.partial",
        &plan.fingerprint[..16],
        std::process::id()
    ));
    if partial.exists() {
        return Err(format!(
            "stale synaptic partial archive: {}",
            partial.display()
        ));
    }
    fs::create_dir(&partial)
        .map_err(|error| format!("cannot create synaptic partial archive: {error}"))?;
    let mut manifest = format!("SYNAPTIC ARCHIVE\nfingerprint: {}\n", plan.fingerprint);
    for name in MICROSCOPE_FILES {
        let source = plan.data_dir.join(name);
        if source.is_file() {
            let destination = partial.join(name);
            fs::copy(&source, &destination)
                .map_err(|error| format!("cannot archive {name}: {error}"))?;
            let size = fs::metadata(&destination)
                .map_err(|error| format!("cannot inspect archived {name}: {error}"))?
                .len();
            manifest.push_str(&format!(
                "file: {name}\t{size}\t{}\n",
                hash_file(&destination)?
            ));
        }
    }
    fs::write(partial.join("manifest.txt"), manifest)
        .map_err(|error| format!("cannot seal synaptic archive: {error}"))?;
    fs::rename(&partial, &plan.archive_dir)
        .map_err(|error| format!("cannot publish synaptic archive: {error}"))?;
    verify_synaptic_archive(plan).map(|_| ())
}

fn verify_synaptic_archive(plan: &SynapticPlan) -> Result<HashSet<&'static str>, String> {
    let manifest = fs::read_to_string(plan.archive_dir.join("manifest.txt"))
        .map_err(|error| format!("cannot read synaptic archive manifest: {error}"))?;
    if !manifest.contains(&format!("fingerprint: {}", plan.fingerprint)) {
        return Err("synaptic archive fingerprint mismatch".to_string());
    }
    let expected: HashSet<_> = MICROSCOPE_FILES
        .iter()
        .copied()
        .filter(|name| plan.archive_dir.join(name).is_file())
        .collect();
    let mut listed = HashSet::new();
    for line in manifest.lines().filter(|line| line.starts_with("file: ")) {
        let fields: Vec<_> = line[6..].split('\t').collect();
        if fields.len() != 3 || !MICROSCOPE_FILES.contains(&fields[0]) {
            return Err("invalid synaptic archive manifest entry".to_string());
        }
        let name = MICROSCOPE_FILES
            .iter()
            .copied()
            .find(|name| *name == fields[0])
            .ok_or_else(|| "invalid synaptic archive manifest entry".to_string())?;
        if !listed.insert(name) {
            return Err(format!("duplicate synaptic archive entry: {name}"));
        }
        let path = plan.archive_dir.join(fields[0]);
        let size: u64 = fields[1]
            .parse()
            .map_err(|_| "invalid synaptic archive size".to_string())?;
        if fs::metadata(&path)
            .map_err(|error| format!("missing archived file: {error}"))?
            .len()
            != size
            || hash_file(&path)? != fields[2]
        {
            return Err(format!(
                "synaptic archive verification failed: {}",
                fields[0]
            ));
        }
    }
    if listed != expected {
        return Err("synaptic archive manifest does not match the sealed file set".to_string());
    }
    let fingerprint = data_fingerprint(&plan.archive_dir)?;
    if fingerprint != plan.fingerprint {
        return Err("synaptic archive fingerprint mismatch".to_string());
    }
    Ok(listed)
}

fn restore_synaptic_archive(plan: &SynapticPlan) -> Result<(), String> {
    let archived = verify_synaptic_archive(plan)?;
    for name in &archived {
        let source = plan.archive_dir.join(name);
        let target = plan.data_dir.join(name);
        let temporary = plan.data_dir.join(format!("{name}.synaptic-restore.tmp"));
        if temporary.exists() {
            fs::remove_file(&temporary)
                .map_err(|error| format!("cannot clear restore temp: {error}"))?;
        }
        fs::copy(&source, &temporary)
            .map_err(|error| format!("cannot stage restore for {name}: {error}"))?;
        if target.exists() {
            fs::remove_file(&target).map_err(|error| format!("cannot replace {name}: {error}"))?;
        }
        fs::rename(&temporary, &target)
            .map_err(|error| format!("cannot commit restore for {name}: {error}"))?;
    }
    for name in MICROSCOPE_FILES {
        let target = plan.data_dir.join(name);
        if !archived.contains(name) && target.is_file() {
            fs::remove_file(&target)
                .map_err(|error| format!("cannot remove post-dream file {name}: {error}"))?;
        }
    }
    Ok(())
}

fn microscope_command(plan: &SynapticPlan, args: &[&str]) -> Result<String, String> {
    let spec = ProcessSpec::new(plan.executable.to_string_lossy())
        .args(args.iter().copied())
        .timeout_ms(120_000)
        .max_output_bytes(1024 * 1024)
        .env("MICROSCOPE_CONFIG", plan.config.to_string_lossy());
    let result = process::run_process(&spec).map_err(|outcome| outcome.output)?;
    let stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let stderr = String::from_utf8_lossy(&result.stderr).to_string();
    if result.exit_code != 0
        || stdout.to_lowercase().contains("error:")
        || stderr.to_lowercase().contains("error:")
    {
        return Err(format!(
            "microscope command '{}' failed: {} {}",
            args.join(" "),
            stdout.trim(),
            stderr.trim()
        ));
    }
    Ok(stdout)
}

fn microscope_process_running() -> Result<bool, String> {
    #[cfg(windows)]
    {
        let output = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "@(Get-Process -Name 'microscope-mem' -ErrorAction SilentlyContinue).Count",
            ])
            .output()
            .map_err(|error| format!("cannot inspect microscope processes: {error}"))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim() != "0")
    }
    #[cfg(not(windows))]
    Ok(false)
}

pub fn synaptic_apply(
    executable: &Path,
    config: &Path,
    confirm: &str,
    allow_write: bool,
) -> ExecutionOutcome {
    if !allow_write {
        return ExecutionOutcome::failed(
            "synaptic_permission_required",
            "synaptic apply requires --allow-write",
        );
    }
    if let Err(error) = run_endurance_guard() {
        return ExecutionOutcome::failed("endurance_lease_active", error);
    }
    let plan = match build_synaptic_plan(executable, config) {
        Ok(plan) => plan,
        Err(error) => return ExecutionOutcome::failed("synaptic_plan_failed", error),
    };
    if confirm != plan.confirmation {
        return ExecutionOutcome::failed(
            "synaptic_confirmation_mismatch",
            "Microscope state changed or confirmation token is invalid; run plan again",
        );
    }
    match microscope_process_running() {
        Ok(true) => {
            return ExecutionOutcome::failed(
                "synaptic_runtime_busy",
                "a microscope-mem process is active; pruning requires an offline window",
            );
        }
        Err(error) => return ExecutionOutcome::failed("synaptic_process_probe_failed", error),
        Ok(false) => {}
    }
    if let Err(error) = write_synaptic_archive(&plan) {
        return ExecutionOutcome::failed("synaptic_archive_failed", error);
    }
    let operation = (|| -> Result<(String, String, String), String> {
        let dream = microscope_command(&plan, &["dream"])?;
        let crc = microscope_command(&plan, &["verify"])?;
        let merkle = microscope_command(&plan, &["verify-merkle"])?;
        Ok((dream, crc, merkle))
    })();
    match operation {
        Ok((dream, crc, merkle)) => {
            let after = match data_fingerprint(&plan.data_dir) {
                Ok(fingerprint) => fingerprint,
                Err(error) => {
                    let rollback = restore_synaptic_archive(&plan);
                    return ExecutionOutcome::failed(
                        "synaptic_post_state_failed",
                        format!("{error}; rollback={rollback:?}"),
                    );
                }
            };
            ExecutionOutcome::completed(format!(
                "SYNAPTIC APPLY\nstatus: completed\narchive: {}\nbefore: {}\nafter: {}\ncrc: passed\nmerkle: passed\ndream-receipt-sha256: {}\ncrc-receipt-sha256: {}\nmerkle-receipt-sha256: {}",
                plan.archive_dir.display(),
                plan.fingerprint,
                after,
                sha256(dream),
                sha256(crc),
                sha256(merkle)
            ))
        }
        Err(error) => match restore_synaptic_archive(&plan) {
            Ok(()) => ExecutionOutcome::failed(
                "synaptic_validation_failed",
                format!("{error}; rollback=completed"),
            ),
            Err(rollback) => ExecutionOutcome::failed(
                "synaptic_rollback_failed",
                format!("{error}; rollback={rollback}"),
            ),
        },
    }
}

fn canonical_allowed(path: &Path) -> Result<PathBuf, String> {
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("invalid path {}: {error}", path.display()))?;
    let mut roots = Vec::new();
    if let Ok(cwd) = env::current_dir().and_then(fs::canonicalize) {
        roots.push(cwd);
    }
    if let Some(configured) = env::var_os("OCTOPUS_ALLOWED_ROOTS") {
        roots.extend(env::split_paths(&configured).filter_map(|root| fs::canonicalize(root).ok()));
    }
    let candidate = canonical.to_string_lossy().to_lowercase();
    if roots.iter().any(|root| {
        let root = root.to_string_lossy().to_lowercase();
        candidate == root
            || candidate
                .strip_prefix(root.trim_end_matches(['\\', '/']))
                .is_some_and(|suffix| suffix.starts_with(['\\', '/']))
    }) {
        Ok(canonical)
    } else {
        Err(format!(
            "path is outside allowed roots: {}",
            canonical.display()
        ))
    }
}

fn build_crispr_plan(
    target: &Path,
    replacement: &Path,
    health_args: &[String],
) -> Result<CrisprPlan, String> {
    let target = canonical_allowed(target)?;
    let replacement = canonical_allowed(replacement)?;
    if target == replacement || !target.is_file() || !replacement.is_file() {
        return Err("CRISPR requires two distinct regular files".to_string());
    }
    for path in [&target, &replacement] {
        let size = fs::metadata(path)
            .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?
            .len();
        if size > MAX_PATCH_BYTES {
            return Err(format!("CRISPR file exceeds {MAX_PATCH_BYTES} bytes"));
        }
    }
    let target_hash = hash_file(&target)?;
    let replacement_hash = hash_file(&replacement)?;
    if target_hash == replacement_hash {
        return Err("replacement is byte-identical to the target".to_string());
    }
    let args = if health_args.is_empty()
        && target.extension().and_then(|extension| extension.to_str()) == Some("exe")
    {
        vec!["--version".to_string()]
    } else {
        health_args.to_vec()
    };
    let token = confirmation(
        "CRI",
        &format!(
            "v1|{}|{}|{}|{}|{}",
            target.display(),
            target_hash,
            replacement.display(),
            replacement_hash,
            args.join("\u{1f}")
        ),
    );
    let file_name = target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "target has no valid file name".to_string())?;
    let backup = target.with_file_name(format!("{file_name}.crispr-{}.bak", &token[4..16]));
    Ok(CrisprPlan {
        target,
        replacement,
        target_hash,
        replacement_hash,
        backup,
        health_args: args,
        confirmation: token,
    })
}

pub fn crispr_plan(target: &Path, replacement: &Path, health_args: &[String]) -> ExecutionOutcome {
    match build_crispr_plan(target, replacement, health_args) {
        Ok(plan) => ExecutionOutcome::completed(format!(
            "CRISPR PLAN\nmode: dry-run\ntarget: {}\ntarget-sha256: {}\nreplacement: {}\nreplacement-sha256: {}\nbackup: {}\nhealth-args: {}\nconfirm: {}",
            plan.target.display(),
            plan.target_hash,
            plan.replacement.display(),
            plan.replacement_hash,
            plan.backup.display(),
            if plan.health_args.is_empty() { "hash-only".to_string() } else { plan.health_args.join(" ") },
            plan.confirmation
        )),
        Err(error) => ExecutionOutcome::failed("crispr_plan_failed", error),
    }
}

fn crispr_health(plan: &CrisprPlan) -> Result<String, String> {
    if plan.health_args.is_empty() {
        return Ok("hash-only".to_string());
    }
    let spec = ProcessSpec::new(plan.target.to_string_lossy())
        .args(plan.health_args.iter().cloned())
        .timeout_ms(15_000)
        .max_output_bytes(256 * 1024);
    let result = process::run_process(&spec).map_err(|outcome| outcome.output)?;
    if result.exit_code != 0 {
        return Err(format!("health command exited {}", result.exit_code));
    }
    Ok(sha256([result.stdout, result.stderr].concat()))
}

fn rollback_crispr(plan: &CrisprPlan) -> Result<(), String> {
    if plan.target.exists() {
        let rejected = plan.target.with_file_name(format!(
            "{}.rejected-{}",
            plan.target
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            &plan.replacement_hash[..12]
        ));
        if rejected.exists() {
            fs::remove_file(&rejected)
                .map_err(|error| format!("cannot clear rejected patch: {error}"))?;
        }
        fs::rename(&plan.target, rejected)
            .map_err(|error| format!("cannot quarantine rejected patch: {error}"))?;
    }
    fs::rename(&plan.backup, &plan.target)
        .map_err(|error| format!("cannot restore CRISPR backup: {error}"))?;
    if hash_file(&plan.target)? != plan.target_hash {
        return Err("restored CRISPR target hash mismatch".to_string());
    }
    Ok(())
}

pub fn crispr_apply(
    target: &Path,
    replacement: &Path,
    health_args: &[String],
    confirm: &str,
    allow_write: bool,
) -> ExecutionOutcome {
    if !allow_write {
        return ExecutionOutcome::failed(
            "crispr_permission_required",
            "CRISPR apply requires --allow-write",
        );
    }
    if let Err(error) = run_endurance_guard() {
        return ExecutionOutcome::failed("endurance_lease_active", error);
    }
    let plan = match build_crispr_plan(target, replacement, health_args) {
        Ok(plan) => plan,
        Err(error) => return ExecutionOutcome::failed("crispr_plan_failed", error),
    };
    if confirm != plan.confirmation {
        return ExecutionOutcome::failed(
            "crispr_confirmation_mismatch",
            "target or replacement changed, or confirmation token is invalid; run plan again",
        );
    }
    if plan.backup.exists() {
        if hash_file(&plan.backup).ok().as_deref() != Some(plan.target_hash.as_str()) {
            return ExecutionOutcome::failed(
                "crispr_backup_conflict",
                format!(
                    "conflicting backup already exists: {}",
                    plan.backup.display()
                ),
            );
        }
        if let Err(error) = fs::remove_file(&plan.backup) {
            return ExecutionOutcome::failed("crispr_backup_refresh_failed", error.to_string());
        }
    }
    let temporary = plan.target.with_file_name(format!(
        ".{}.crispr-{}.tmp",
        plan.target
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        std::process::id()
    ));
    if temporary.exists() {
        return ExecutionOutcome::failed(
            "crispr_stale_temporary",
            format!("stale temporary exists: {}", temporary.display()),
        );
    }
    if let Err(error) = fs::copy(&plan.replacement, &temporary) {
        return ExecutionOutcome::failed("crispr_stage_failed", error.to_string());
    }
    if hash_file(&temporary).ok().as_deref() != Some(plan.replacement_hash.as_str()) {
        let _ = fs::remove_file(&temporary);
        return ExecutionOutcome::failed(
            "crispr_stage_hash_mismatch",
            "staged patch hash mismatch",
        );
    }
    if hash_file(&plan.target).ok().as_deref() != Some(plan.target_hash.as_str()) {
        let _ = fs::remove_file(&temporary);
        return ExecutionOutcome::failed(
            "crispr_target_changed",
            "target changed after confirmation; run plan again",
        );
    }
    if let Err(error) = fs::rename(&plan.target, &plan.backup) {
        let _ = fs::remove_file(&temporary);
        return ExecutionOutcome::failed(
            "crispr_target_in_use",
            format!("cannot lock target as backup: {error}"),
        );
    }
    if hash_file(&plan.backup).ok().as_deref() != Some(plan.target_hash.as_str()) {
        let restore = fs::rename(&plan.backup, &plan.target);
        let _ = fs::remove_file(&temporary);
        return match restore {
            Ok(()) => ExecutionOutcome::failed(
                "crispr_target_changed",
                "target changed during commit; rollback=completed",
            ),
            Err(error) => ExecutionOutcome::failed(
                "crispr_rollback_failed",
                format!("target changed during commit; rollback={error}"),
            ),
        };
    }
    if let Err(error) = fs::rename(&temporary, &plan.target) {
        let rollback = fs::rename(&plan.backup, &plan.target);
        let _ = fs::remove_file(&temporary);
        return match rollback {
            Ok(()) => ExecutionOutcome::failed(
                "crispr_commit_failed",
                format!("patch commit failed; rollback=completed: {error}"),
            ),
            Err(rollback_error) => ExecutionOutcome::failed(
                "crispr_rollback_failed",
                format!("patch commit failed: {error}; rollback={rollback_error}"),
            ),
        };
    }
    if hash_file(&plan.target).ok().as_deref() != Some(plan.replacement_hash.as_str()) {
        return match rollback_crispr(&plan) {
            Ok(()) => ExecutionOutcome::failed(
                "crispr_commit_hash_mismatch",
                "committed patch hash mismatch; rollback=completed",
            ),
            Err(error) => ExecutionOutcome::failed("crispr_rollback_failed", error),
        };
    }
    let health = match crispr_health(&plan) {
        Ok(receipt) => receipt,
        Err(error) => {
            return match rollback_crispr(&plan) {
                Ok(()) => ExecutionOutcome::failed(
                    "crispr_health_failed",
                    format!("{error}; rollback=completed"),
                ),
                Err(rollback) => ExecutionOutcome::failed(
                    "crispr_rollback_failed",
                    format!("{error}; rollback={rollback}"),
                ),
            };
        }
    };
    ExecutionOutcome::completed(format!(
        "CRISPR APPLY\nstatus: committed\ntarget: {}\nold-sha256: {}\nnew-sha256: {}\nbackup: {}\nhealth: passed\nhealth-receipt: {}\nrollback-ready: true",
        plan.target.display(),
        plan.target_hash,
        plan.replacement_hash,
        plan.backup.display(),
        health
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQUENCE: AtomicU64 = AtomicU64::new(1);

    fn fixture(label: &str) -> PathBuf {
        let root = env::current_dir().unwrap().join("target").join(format!(
            "bio-actuator-{label}-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn crispr_plan_is_hash_bound() {
        let root = fixture("crispr-plan");
        let target = root.join("target.txt");
        let replacement = root.join("replacement.txt");
        fs::write(&target, "old").unwrap();
        fs::write(&replacement, "new").unwrap();
        let first = build_crispr_plan(&target, &replacement, &[]).unwrap();
        fs::write(&replacement, "changed").unwrap();
        let second = build_crispr_plan(&target, &replacement, &[]).unwrap();
        assert_ne!(first.confirmation, second.confirmation);
    }

    #[test]
    fn crispr_text_apply_commits_and_preserves_backup() {
        let root = fixture("crispr-apply");
        let target = root.join("target.txt");
        let replacement = root.join("replacement.txt");
        fs::write(&target, "old").unwrap();
        fs::write(&replacement, "new").unwrap();
        let plan = build_crispr_plan(&target, &replacement, &[]).unwrap();
        let outcome = crispr_apply(&target, &replacement, &[], &plan.confirmation, true);
        assert!(!outcome.is_failed(), "{}", outcome.output);
        assert_eq!(fs::read_to_string(&target).unwrap(), "new");
        assert_eq!(fs::read_to_string(&plan.backup).unwrap(), "old");
    }

    #[test]
    fn crispr_apply_requires_explicit_write_permission() {
        let root = fixture("crispr-permission");
        let target = root.join("target.txt");
        let replacement = root.join("replacement.txt");
        fs::write(&target, "old").unwrap();
        fs::write(&replacement, "new").unwrap();
        let plan = build_crispr_plan(&target, &replacement, &[]).unwrap();
        let outcome = crispr_apply(&target, &replacement, &[], &plan.confirmation, false);
        assert_eq!(outcome.code.as_deref(), Some("crispr_permission_required"));
        assert_eq!(fs::read_to_string(&target).unwrap(), "old");
        assert!(!plan.backup.exists());
    }

    #[test]
    fn synaptic_archive_detects_tampering() {
        let root = fixture("synaptic-archive");
        let data = root.join("data");
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("activations.bin"), b"activation-state").unwrap();
        let plan = SynapticPlan {
            executable: root.join("microscope.exe"),
            config: root.join("config.toml"),
            data_dir: data,
            archive_dir: root.join("archives/synaptic-test"),
            fingerprint: data_fingerprint(&root.join("data")).unwrap(),
            executable_hash: "x".repeat(64),
            config_hash: "y".repeat(64),
            confirmation: "SYN-test".to_string(),
        };
        write_synaptic_archive(&plan).unwrap();
        fs::write(plan.archive_dir.join("activations.bin"), b"tampered").unwrap();
        assert!(verify_synaptic_archive(&plan).is_err());
    }

    #[test]
    fn synaptic_archive_rejects_an_incomplete_manifest() {
        let root = fixture("synaptic-incomplete-manifest");
        let data = root.join("data");
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("activations.bin"), b"activation-state").unwrap();
        fs::write(data.join("resonance.bin"), b"resonance-state").unwrap();
        let plan = SynapticPlan {
            executable: root.join("microscope.exe"),
            config: root.join("config.toml"),
            data_dir: data,
            archive_dir: root.join("archives/synaptic-test"),
            fingerprint: data_fingerprint(&root.join("data")).unwrap(),
            executable_hash: "x".repeat(64),
            config_hash: "y".repeat(64),
            confirmation: "SYN-test".to_string(),
        };
        write_synaptic_archive(&plan).unwrap();
        let manifest_path = plan.archive_dir.join("manifest.txt");
        let manifest = fs::read_to_string(&manifest_path).unwrap();
        let incomplete = manifest
            .lines()
            .filter(|line| !line.starts_with("file: resonance.bin\t"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&manifest_path, incomplete).unwrap();
        assert!(verify_synaptic_archive(&plan).is_err());
    }

    #[test]
    fn missing_endurance_guard_fails_closed() {
        let root = fixture("missing-guard");
        let error = run_endurance_guard_at(&root.join("missing.ps1")).unwrap_err();
        assert!(error.contains("endurance guard is missing"), "{error}");
    }
}
