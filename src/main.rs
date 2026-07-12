use clap::{Parser, Subcommand};
use std::io::{self, IsTerminal, Read};

#[derive(Parser)]
#[command(
    name = "octopus-runtime",
    version,
    about = "Standalone native Rust blade runtime"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    List,
    Capabilities,
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
    Cancel {
        root_id: String,
    },
    Orphans,
}

fn prompt(parts: Vec<String>) -> Result<String, String> {
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
    Ok(input.trim_end().to_string())
}

fn main() {
    octopus_runtime::orch_init();
    let cli = Cli::parse();
    match cli.command {
        Command::Mcp => octopus_runtime::run_mcp(),
        Command::List => println!("{}", octopus_runtime::list().join("\n")),
        Command::Capabilities => println!("{}", octopus_runtime::render_capabilities()),
        Command::Run {
            blade,
            prompt: parts,
        } => match prompt(parts) {
            Ok(input) => print_outcome(octopus_runtime::run_outcome(&blade, &input)),
            Err(error) => exit_with_error(error),
        },
        Command::Arm {
            spec,
            prompt: parts,
        } => match prompt(parts) {
            Ok(input) => print_outcome(octopus_runtime::run_arm_outcome(&spec, &input)),
            Err(error) => exit_with_error(error),
        },
        Command::Pipeline {
            spec,
            prompt: parts,
        } => match prompt(parts) {
            Ok(input) => print_outcome(octopus_runtime::run_pipeline_outcome(&spec, &input)),
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
        Command::Orphans => {
            print_outcome(octopus_runtime::orch_orphans());
        }
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
