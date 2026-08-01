use crate::mitosis::{GenerationRequest, Genome, Ribosome, TemplateId};
use crate::output;
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::net::SocketAddr;
use std::path::PathBuf;

fn parse_replication_count(value: &str) -> Result<usize, String> {
    let count = value
        .parse::<usize>()
        .map_err(|_| "replication count must be an unsigned integer".to_string())?;
    crate::mitosis::validate_replication_count(count)?;
    Ok(count)
}

#[derive(Debug, Parser)]
#[command(
    name = "ribosome-synth",
    about = "Bounded code generation and verified local binary replication"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Plan generation, or compile and atomically publish with --apply.
    Generate {
        /// Canonical implemented template identifier.
        #[arg(long, default_value = "minimal-drone")]
        template: String,

        /// Drone identifier: 1-64 ASCII letters, digits, '-' or '_'.
        #[arg(long)]
        name: String,

        /// Existing output root. Generated filenames are always contained here.
        #[arg(long)]
        output_root: PathBuf,

        /// Simple contained binary filename. Defaults from the drone name.
        #[arg(long)]
        output: Option<String>,

        /// Simple contained Rust source filename. Defaults from the drone name.
        #[arg(long)]
        source: Option<String>,

        /// Generated drone generation number.
        #[arg(long, default_value_t = 0)]
        generation: u32,

        /// Recorded Queen endpoint. The minimal template does not connect to it.
        #[arg(long, default_value = "127.0.0.1:9000")]
        queen_addr: SocketAddr,

        /// Perform compilation and filesystem publication. Without this flag, only plan.
        #[arg(long)]
        apply: bool,
    },
    /// List canonical templates that are actually implemented.
    Templates,
    /// Plan or create verified local copies of this exact binary; never starts them.
    Replicate {
        /// Existing directory that will contain all copies.
        #[arg(long)]
        output_root: PathBuf,

        /// Base name used for copy filenames.
        #[arg(long)]
        name: String,

        /// Number of copies, bounded to 1..=16.
        #[arg(long, default_value_t = 1, value_parser = parse_replication_count)]
        count: usize,

        /// Perform filesystem publication. Without this flag, only plan.
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Debug, Serialize)]
pub struct GenerationResult {
    pub timestamp: String,
    pub template: String,
    pub drone_name: String,
    pub generation: u32,
    pub source_path: String,
    pub binary_path: String,
    pub generated: bool,
    pub source_blake3: String,
    pub binary_blake3: Option<String>,
    pub size_bytes: u64,
    pub compile_time_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ReplicationCommandResult {
    timestamp: String,
    applied: bool,
    count: usize,
    source_blake3: String,
    size_bytes: usize,
    targets: Vec<String>,
}

pub async fn dispatch(args: &[String]) -> Result<String, String> {
    let cli = Cli::try_parse_from(args).map_err(|error| error.to_string())?;

    match cli.command {
        Commands::Generate {
            template,
            name,
            output_root,
            output,
            source,
            generation,
            queen_addr,
            apply,
        } => {
            let template = TemplateId::parse(&template)?;
            let parent_name = Genome::self_name();
            let parent_hash = Genome::self_hash().map_err(|error| error.to_string())?;
            let request = GenerationRequest {
                template,
                source_name: source.unwrap_or_else(|| Ribosome::default_source_name(&name)),
                binary_name: output.unwrap_or_else(|| Ribosome::default_binary_name(&name)),
                drone_name: name,
                generation,
                queen_addr,
                parent_name,
                parent_hash,
                output_root,
            };
            let rendered = Ribosome::render(&request)?;
            let (source_path, binary_path) = Ribosome::planned_paths(&request)?;

            output::banner("RIBOSOME-SYNTH", "Bounded Binary Generator", "◈");
            output::section(if apply {
                "Generation Apply"
            } else {
                "Generation Plan"
            });
            output::kv("Template", request.template.name());
            output::kv("Drone Name", &request.drone_name);
            output::kv("Output Root", &request.output_root.display().to_string());

            let result = if apply {
                let artifact = Ribosome::generate(&request)?;
                output::success("Generated source and binary committed atomically");
                GenerationResult {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    template: rendered.template,
                    drone_name: request.drone_name,
                    generation: request.generation,
                    source_path: artifact.source_path,
                    binary_path: artifact.binary_path,
                    generated: true,
                    source_blake3: artifact.source_blake3,
                    binary_blake3: Some(artifact.binary_blake3),
                    size_bytes: artifact.size_bytes,
                    compile_time_ms: Some(artifact.compile_time_ms),
                }
            } else {
                output::warn("Plan only: pass --apply to write and compile artifacts");
                GenerationResult {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    template: rendered.template,
                    drone_name: request.drone_name,
                    generation: request.generation,
                    source_path: source_path.to_string_lossy().to_string(),
                    binary_path: binary_path.to_string_lossy().to_string(),
                    generated: false,
                    source_blake3: rendered.blake3,
                    binary_blake3: None,
                    size_bytes: 0,
                    compile_time_ms: None,
                }
            };
            output::summary(
                "ribosome-synth",
                if result.generated {
                    "Generation committed"
                } else {
                    "Generation plan complete"
                },
            );
            serde_json::to_string_pretty(&result).map_err(|error| error.to_string())
        }
        Commands::Templates => {
            output::banner("RIBOSOME-SYNTH", "Implemented Templates", "◈");
            output::section("Code Templates");
            for template in Ribosome::templates() {
                println!("  ▸ {template}");
            }
            output::summary("ribosome-synth", "Canonical template list");
            serde_json::to_string_pretty(Ribosome::templates()).map_err(|error| error.to_string())
        }
        Commands::Replicate {
            output_root,
            name,
            count,
            apply,
        } => {
            let planned = Ribosome::planned_replication_paths(&output_root, &name, count)?;
            let genome = Genome::read_self().map_err(|error| error.to_string())?;
            let source_hash = blake3::hash(&genome).to_hex().to_string();

            output::banner("RIBOSOME-SYNTH", "Verified Local Replication", "◈");
            output::section(if apply {
                "Replication Apply"
            } else {
                "Replication Plan"
            });
            output::kv("Replication Count", &count.to_string());

            let targets: Vec<String> = if apply {
                let results = Ribosome::replicate_local_copies(&output_root, &name, count)?;
                output::success(&format!(
                    "Committed and verified {} local copies",
                    results.len()
                ));
                results
                    .into_iter()
                    .map(|result| result.target_path)
                    .collect()
            } else {
                output::warn("Plan only: pass --apply to write local copies");
                planned
                    .into_iter()
                    .map(|path| path.to_string_lossy().to_string())
                    .collect()
            };
            let result = ReplicationCommandResult {
                timestamp: chrono::Utc::now().to_rfc3339(),
                applied: apply,
                count: targets.len(),
                source_blake3: source_hash,
                size_bytes: genome.len(),
                targets,
            };
            output::summary(
                "ribosome-synth",
                if apply {
                    "Local replication committed"
                } else {
                    "Local replication plan complete"
                },
            );
            serde_json::to_string_pretty(&result).map_err(|error| error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_requires_explicit_apply_for_mutation() {
        let cli = Cli::try_parse_from([
            "ribosome-synth",
            "generate",
            "--name",
            "drone_1",
            "--output-root",
            ".",
        ])
        .unwrap();
        match cli.command {
            Commands::Generate { apply, .. } => assert!(!apply),
            _ => panic!("expected generate command"),
        }
    }

    #[test]
    fn replication_count_is_bounded_by_clap() {
        let error = Cli::try_parse_from([
            "ribosome-synth",
            "replicate",
            "--output-root",
            ".",
            "--name",
            "copy",
            "--count",
            "17",
        ])
        .unwrap_err();
        assert!(error.to_string().contains("16"));
    }

    #[test]
    fn only_the_implemented_template_is_advertised() {
        assert_eq!(Ribosome::templates(), &["minimal-drone"]);
        assert!(TemplateId::parse("bio-client").is_err());
    }
}
