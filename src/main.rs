use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "octopus-runtime",
    version,
    about = "Standalone native Rust blade runtime"
)]
struct Cli {
    /// Disable symbiosis visual language output (plain text mode)
    #[arg(long, global = true)]
    plain: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    List,
    Capabilities {
        /// Filter the registry to a named execution profile.
        #[arg(long, value_enum, default_value = "all")]
        profile: CapabilityProfileArg,
    },
    Run {
        blade: String,
        #[arg(trailing_var_arg = true)]
        prompt: Vec<String>,
    },
    Arm {
        spec: String,
        #[arg(trailing_var_arg = true)]
        prompt: Vec<String>,
    },
    Pipeline {
        spec: String,
        #[arg(trailing_var_arg = true)]
        prompt: Vec<String>,
    },
    /// Execute an evidence-bound per-arm JSON manifest.
    Manifest {
        /// Path to the manifest JSON file, or '-' to read exact JSON from stdin.
        path: String,
        /// Permit manifest arms whose declared or inferred effect can write.
        #[arg(long)]
        allow_write: bool,
    },
    /// Inspect and verify the append-only resonance hash chain.
    Resonance {
        /// Verify every link and content hash before rendering the report.
        #[arg(long)]
        verify: bool,
        /// Number of newest entries to render.
        #[arg(long, default_value_t = 10)]
        tail: usize,
    },
    /// Plan or apply guarded biological actuator operations.
    Bio {
        #[command(subcommand)]
        command: BioCommand,
    },
    /// Select a safe Octopus topology through a minimal-token, psi-weighted Marshal.
    Marshal {
        /// Dispatch the selected topology instead of returning only the plan.
        #[arg(long)]
        execute: bool,
        /// Permit a selected topology that can write to the workspace.
        #[arg(long, requires = "execute")]
        allow_write: bool,
        #[arg(trailing_var_arg = true)]
        task: Vec<String>,
    },
    Mcp,
    Status {
        root_id: String,
    },
    Resume {
        root_id: String,
    },
    Retry {
        arm_id: String,
    },
    /// Cancel a root by marking its running arms as cancelled in state.
    /// Does NOT interrupt in-flight processes (blades run synchronously in this process).
    /// Use to mark stuck/orphaned arms for audit purposes.
    Cancel {
        root_id: String,
    },
    /// Run end-to-end benchmarks on all 33 Bio-Binaries targets (direct + Octopus).
    /// Outputs CSV + Markdown report to .octopus-rust/bio-benchmarks/<timestamp>/
    Benchmark {
        /// Warmup runs per module before measured samples.
        #[arg(long, default_value_t = 1)]
        warmup: usize,
        /// Measured samples per module per mode.
        #[arg(long, default_value_t = 3)]
        samples: usize,
        /// Timeout per process in seconds.
        #[arg(long, default_value_t = 60)]
        timeout: u64,
        /// Keep raw output for every run (not just failures).
        #[arg(long)]
        keep_raw: bool,
    },
    Orphans,
    StateAudit {
        #[arg(long, default_value_t = 24)]
        stale_hours: u64,
        #[arg(long)]
        stale_minutes: Option<u64>,
    },
    StateRepair {
        #[arg(long, default_value_t = 24)]
        stale_hours: u64,
        #[arg(long)]
        stale_minutes: Option<u64>,
    },
    StateBackup {
        #[command(subcommand)]
        command: StateBackupCommand,
    },
    StateRestore {
        #[command(subcommand)]
        command: StateRestoreCommand,
    },
}

#[derive(Subcommand)]
enum BioCommand {
    /// Show the separately bundled 33-binary Bio subsystem and availability.
    Status,
    /// Execute one exact bundled Bio-Binaries target through the safe process boundary.
    External {
        name: String,
        /// Permit Bio targets classified as write or control effects.
        #[arg(long)]
        allow_mutation: bool,
        /// Exact child arguments. Each CLI argument is forwarded without shell parsing.
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Inspect or terminate one exact, revalidated non-protected PID.
    Macrophage {
        #[command(subcommand)]
        command: MacrophageCommand,
    },
    /// Archive and run Microscope dream consolidation with integrity gates.
    Synaptic {
        #[command(subcommand)]
        command: SynapticCommand,
    },
    /// Transactionally replace one bounded file with rollback and health checks.
    Crispr {
        #[command(subcommand)]
        command: CrisprCommand,
    },
}

#[derive(Subcommand)]
enum MacrophageCommand {
    Plan {
        pid: u32,
    },
    Apply {
        pid: u32,
        #[arg(long)]
        confirm: String,
        #[arg(long)]
        allow_kill: bool,
    },
}

#[derive(Subcommand)]
enum SynapticCommand {
    Plan {
        #[arg(
            long,
            default_value = "D:\\codex\\microscope-memory\\target\\release\\microscope-mem.exe"
        )]
        executable: PathBuf,
        #[arg(long, default_value = "D:\\codex\\microscope-memory\\config.toml")]
        config: PathBuf,
    },
    Apply {
        #[arg(
            long,
            default_value = "D:\\codex\\microscope-memory\\target\\release\\microscope-mem.exe"
        )]
        executable: PathBuf,
        #[arg(long, default_value = "D:\\codex\\microscope-memory\\config.toml")]
        config: PathBuf,
        #[arg(long)]
        confirm: String,
        #[arg(long)]
        allow_write: bool,
    },
}

#[derive(Subcommand)]
enum CrisprCommand {
    Plan {
        target: PathBuf,
        replacement: PathBuf,
        #[arg(long = "health-arg")]
        health_args: Vec<String>,
    },
    Apply {
        target: PathBuf,
        replacement: PathBuf,
        #[arg(long = "health-arg")]
        health_args: Vec<String>,
        #[arg(long)]
        confirm: String,
        #[arg(long)]
        allow_write: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum CapabilityProfileArg {
    All,
    WindowsOffline,
}

impl From<CapabilityProfileArg> for octopus_runtime::CapabilityProfile {
    fn from(value: CapabilityProfileArg) -> Self {
        match value {
            CapabilityProfileArg::All => Self::All,
            CapabilityProfileArg::WindowsOffline => Self::WindowsOffline,
        }
    }
}

#[derive(Subcommand)]
enum StateBackupCommand {
    /// Create and verify a sealed state backup.
    Create,
    /// Verify a sealed or legacy state backup by direct state-* identifier.
    Verify { backup_id: String },
}

#[derive(Subcommand)]
enum StateRestoreCommand {
    /// Validate a sealed backup and show the exact non-mutating restore plan.
    Plan { backup_id: String },
    /// Restore a sealed backup after an exact backup-id confirmation.
    Apply {
        backup_id: String,
        #[arg(long)]
        confirm: String,
    },
    /// Recover or finish an interrupted journaled restore transaction.
    Recover,
}

fn prompt(parts: Vec<String>, preserve_stdin: bool) -> Result<String, String> {
    if !parts.is_empty() {
        return Ok(parts.join(" "));
    }
    if io::stdin().is_terminal() {
        return Ok(String::new());
    }
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("stdin read failed: {error}"))?;
    if preserve_stdin {
        // stdin is the exact transport for code-writer payloads. Do not trim
        // trailing newlines or spaces from file content.
        Ok(input)
    } else {
        Ok(input.trim_end().to_string())
    }
}

fn main() {
    let cli = Cli::parse();
    let explicit_recovery = matches!(
        &cli.command,
        Command::StateRestore {
            command: StateRestoreCommand::Recover
        }
    );
    if !explicit_recovery {
        let recovery = octopus_runtime::state_restore_auto_recover();
        if recovery.is_failed() {
            exit_with_error(recovery.output);
        }
    }

    let internally_exclusive = matches!(
        &cli.command,
        Command::StateRestore {
            command: StateRestoreCommand::Apply { .. } | StateRestoreCommand::Recover
        }
    );
    let maintenance_exclusive = matches!(
        &cli.command,
        Command::StateRepair { .. }
            | Command::StateBackup {
                command: StateBackupCommand::Create
            }
    );
    let bio_mutation_exclusive = matches!(
        &cli.command,
        Command::Bio {
            command: BioCommand::Macrophage {
                command: MacrophageCommand::Apply { .. }
            } | BioCommand::Synaptic {
                command: SynapticCommand::Apply { .. }
            } | BioCommand::Crispr {
                command: CrisprCommand::Apply { .. }
            }
        }
    ) || matches!(
        &cli.command,
        Command::Bio {
            command: BioCommand::External {
                allow_mutation: true,
                ..
            }
        }
    );
    let _state_guard = if internally_exclusive {
        None
    } else {
        let guard = if maintenance_exclusive || bio_mutation_exclusive {
            octopus_runtime::state_exclusive_command_guard()
        } else {
            octopus_runtime::state_command_guard()
        };
        match guard {
            Ok(guard) => Some(guard),
            Err(error) => exit_with_error(format!("state lock failed: {error}")),
        }
    };
    if !internally_exclusive {
        octopus_runtime::orch_init();
    }

    match cli.command {
        Command::Mcp => octopus_runtime::run_mcp(),
        Command::List => println!("{}", octopus_runtime::list().join("\n")),
        Command::Capabilities { profile } => println!(
            "{}",
            octopus_runtime::render_capabilities_for_profile(profile.into())
        ),
        Command::Run {
            blade,
            prompt: parts,
        } => match prompt(parts, blade == "code-writer") {
            Ok(input) => print_outcome(octopus_runtime::run_outcome(&blade, &input)),
            Err(error) => exit_with_error(error),
        },
        Command::Arm {
            spec,
            prompt: parts,
        } => match prompt(parts, spec == "code-writer") {
            Ok(input) => print_outcome(octopus_runtime::run_arm_outcome(&spec, &input)),
            Err(error) => exit_with_error(error),
        },
        Command::Pipeline {
            spec,
            prompt: parts,
        } => match prompt(parts, false) {
            Ok(input) => print_outcome(octopus_runtime::run_pipeline_outcome(&spec, &input)),
            Err(error) => exit_with_error(error),
        },
        Command::Manifest { path, allow_write } => {
            let source = if path == "-" {
                prompt(Vec::new(), true)
            } else {
                fs::read_to_string(&path)
                    .map_err(|error| format!("cannot read manifest '{path}': {error}"))
            };
            match source {
                Ok(source) => {
                    print_outcome(octopus_runtime::run_manifest_outcome(&source, allow_write))
                }
                Err(error) => exit_with_error(error),
            }
        }
        Command::Resonance { verify, tail } => {
            print_outcome(octopus_runtime::resonance_status(verify, tail));
        }
        Command::Bio { command } => match command {
            BioCommand::Status => {
                print_outcome(octopus_runtime::bio_system_status());
            }
            BioCommand::External {
                name,
                allow_mutation,
                args,
            } => {
                print_outcome(octopus_runtime::bio_external_run(
                    &name,
                    &args.join("\n"),
                    allow_mutation,
                ));
            }
            BioCommand::Macrophage { command } => match command {
                MacrophageCommand::Plan { pid } => {
                    print_outcome(octopus_runtime::bio_macrophage_plan(pid));
                }
                MacrophageCommand::Apply {
                    pid,
                    confirm,
                    allow_kill,
                } => {
                    print_outcome(octopus_runtime::bio_macrophage_apply(
                        pid, &confirm, allow_kill,
                    ));
                }
            },
            BioCommand::Synaptic { command } => match command {
                SynapticCommand::Plan { executable, config } => {
                    print_outcome(octopus_runtime::bio_synaptic_plan(&executable, &config));
                }
                SynapticCommand::Apply {
                    executable,
                    config,
                    confirm,
                    allow_write,
                } => {
                    print_outcome(octopus_runtime::bio_synaptic_apply(
                        &executable,
                        &config,
                        &confirm,
                        allow_write,
                    ));
                }
            },
            BioCommand::Crispr { command } => match command {
                CrisprCommand::Plan {
                    target,
                    replacement,
                    health_args,
                } => {
                    print_outcome(octopus_runtime::bio_crispr_plan(
                        &target,
                        &replacement,
                        &health_args,
                    ));
                }
                CrisprCommand::Apply {
                    target,
                    replacement,
                    health_args,
                    confirm,
                    allow_write,
                } => {
                    print_outcome(octopus_runtime::bio_crispr_apply(
                        &target,
                        &replacement,
                        &health_args,
                        &confirm,
                        allow_write,
                    ));
                }
            },
        },
        Command::Marshal {
            execute,
            allow_write,
            task,
        } => match prompt(task, false) {
            Ok(input) if execute => {
                print_outcome(octopus_runtime::marshal_dispatch(&input, allow_write));
            }
            Ok(input) => print_outcome(octopus_runtime::marshal_plan(&input)),
            Err(error) => exit_with_error(error),
        },
        Command::Status { root_id } => {
            print_outcome(octopus_runtime::orch_status(&root_id));
        }
        Command::Resume { root_id } => {
            print_outcome(octopus_runtime::orch_resume(&root_id));
        }
        Command::Retry { arm_id } => {
            print_outcome(octopus_runtime::orch_retry(&arm_id));
        }
        Command::Cancel { root_id } => {
            print_outcome(octopus_runtime::orch_cancel(&root_id));
        }
        Command::Benchmark {
            warmup,
            samples,
            timeout,
            keep_raw,
        } => {
            let cfg = octopus_runtime::bio_benchmark::BenchmarkConfig {
                warmup,
                samples,
                timeout_secs: timeout,
                keep_raw,
            };
            let outcome = octopus_runtime::bio_benchmark::run_benchmarks(cfg);
            print_outcome(outcome);
        }
        Command::Orphans => {
            print_outcome(octopus_runtime::orch_orphans());
        }
        Command::StateAudit {
            stale_hours,
            stale_minutes,
        } => {
            print_outcome(octopus_runtime::state_audit(
                stale_minutes.unwrap_or_else(|| stale_hours.saturating_mul(60)),
            ));
        }
        Command::StateRepair {
            stale_hours,
            stale_minutes,
        } => {
            print_outcome(octopus_runtime::state_repair(
                stale_minutes.unwrap_or_else(|| stale_hours.saturating_mul(60)),
            ));
        }
        Command::StateBackup { command } => match command {
            StateBackupCommand::Create => {
                print_outcome(octopus_runtime::state_backup_create());
            }
            StateBackupCommand::Verify { backup_id } => {
                print_outcome(octopus_runtime::state_backup_verify(&backup_id));
            }
        },
        Command::StateRestore { command } => match command {
            StateRestoreCommand::Plan { backup_id } => {
                print_outcome(octopus_runtime::state_restore_plan(&backup_id));
            }
            StateRestoreCommand::Apply { backup_id, confirm } => {
                print_outcome(octopus_runtime::state_restore_apply(&backup_id, &confirm));
            }
            StateRestoreCommand::Recover => {
                print_outcome(octopus_runtime::state_restore_recover());
            }
        },
    }
}

fn print_outcome(outcome: octopus_runtime::ExecutionOutcome) {
    if outcome.is_failed() {
        eprintln!("{}", outcome.output);
        std::process::exit(outcome.exit_code());
    }
    println!("{}", outcome.output);
}

fn exit_with_error(error: String) -> ! {
    eprintln!("octopus-runtime: {error}");
    std::process::exit(2)
}
