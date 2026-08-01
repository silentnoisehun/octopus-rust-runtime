use crate::outcome::ExecutionOutcome;
use crate::process::{run_process_typed, ProcessSpec};
use sha2::{Digest, Sha256};
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};

pub const DEFAULT_BIO_BINARY_RELATIVE_DIR: &str = "bio-binaries/target/release";
pub const BIO_BINARY_COUNT: usize = 33;
const RELEASE_SHA256SUMS: &str = include_str!("../../bio-binaries/RELEASE_SHA256SUMS");

const MAX_ARGUMENTS: usize = 64;
const MAX_ARGUMENT_BYTES: usize = 4 * 1024;
const MAX_INPUT_BYTES: usize = 32 * 1024;
const EXECUTION_TIMEOUT_MS: u64 = 30_000;
const MAX_OUTPUT_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalEffect {
    Read,
    Write,
    Control,
}

impl ExternalEffect {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Control => "control",
        }
    }

    const fn requires_authorization(self) -> bool {
        matches!(self, Self::Write | Self::Control)
    }
}

impl fmt::Display for ExternalEffect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalBioBinary {
    pub name: &'static str,
    pub effect: ExternalEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalBioAvailability {
    pub binary: ExternalBioBinary,
    pub path: PathBuf,
    pub available: bool,
}

pub static BIO_BINARIES: [ExternalBioBinary; BIO_BINARY_COUNT] = [
    ExternalBioBinary {
        name: "viral-infect",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "hox-diff",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "plasmid-dream",
        effect: ExternalEffect::Control,
    },
    ExternalBioBinary {
        name: "plasmid-inject",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "telepathy-sync",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "telepathy-entangle",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "eqm-pulse",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "eqm-methy",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "aether-excite",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "aether-fabric",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "borg-cube",
        effect: ExternalEffect::Control,
    },
    ExternalBioBinary {
        name: "nexus-logic",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "collective-sync",
        effect: ExternalEffect::Control,
    },
    ExternalBioBinary {
        name: "brain-synapse",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "brain-connectome",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "wave-encoder",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "wave-sculptor",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "iron-resonate",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "path-resonance",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "grid-warp",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "magneto-geo",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "mycelium-spread",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "homeostasis",
        effect: ExternalEffect::Control,
    },
    ExternalBioBinary {
        name: "omega-master",
        effect: ExternalEffect::Control,
    },
    ExternalBioBinary {
        name: "omega-point",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "ribosome-synth",
        effect: ExternalEffect::Control,
    },
    ExternalBioBinary {
        name: "wave-cryo-tx",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "wave-cryo-rx",
        effect: ExternalEffect::Read,
    },
    ExternalBioBinary {
        name: "mutation-sentinel",
        effect: ExternalEffect::Control,
    },
    ExternalBioBinary {
        name: "magneto-acoustic",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "wave-field",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "vagus-nerve",
        effect: ExternalEffect::Write,
    },
    ExternalBioBinary {
        name: "microscope-mem",
        effect: ExternalEffect::Write,
    },
];

pub fn catalog() -> &'static [ExternalBioBinary] {
    &BIO_BINARIES
}

pub fn find(name: &str) -> Option<&'static ExternalBioBinary> {
    BIO_BINARIES.iter().find(|binary| binary.name == name)
}

pub fn binary_dir() -> PathBuf {
    binary_dir_from_override(env::var_os("OCTOPUS_BIO_BIN_DIR"))
}

fn runtime_state_dir() -> PathBuf {
    match env::var_os("OCTOPUS_BIO_STATE_DIR") {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir)
            .join("Octopus")
            .join("bio-runtime"),
    }
}

fn expected_release_sha256(name: &str) -> Option<&'static str> {
    let expected_file = format!("{name}{}", env::consts::EXE_SUFFIX);
    RELEASE_SHA256SUMS.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let hash = fields.next()?;
        let file = fields.next()?;
        (file == expected_file).then_some(hash)
    })
}

fn binary_dir_from_override(value: Option<OsString>) -> PathBuf {
    match value {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => default_binary_dir(),
    }
}

fn default_binary_dir() -> PathBuf {
    if let Ok(executable) = env::current_exe() {
        if let Some(parent) = executable.parent() {
            let adjacent = parent.join("bio-binaries");
            if adjacent.is_dir() {
                return adjacent;
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_BIO_BINARY_RELATIVE_DIR)
}

pub fn binary_path(name: &str) -> Result<PathBuf, ExecutionOutcome> {
    let binary = find(name).ok_or_else(|| unknown_binary(name))?;
    Ok(binary_path_in_dir(binary, &binary_dir()))
}

fn binary_path_in_dir(binary: &ExternalBioBinary, directory: &Path) -> PathBuf {
    directory.join(format!("{}{}", binary.name, env::consts::EXE_SUFFIX))
}

pub fn availability() -> Vec<ExternalBioAvailability> {
    availability_in_dir(&binary_dir())
}

fn availability_in_dir(directory: &Path) -> Vec<ExternalBioAvailability> {
    BIO_BINARIES
        .iter()
        .copied()
        .map(|binary| {
            let path = binary_path_in_dir(&binary, directory);
            let available = path.is_file();
            ExternalBioAvailability {
                binary,
                path,
                available,
            }
        })
        .collect()
}

pub fn render_status() -> String {
    let directory = binary_dir();
    let availability = availability_in_dir(&directory);
    let available = availability.iter().filter(|entry| entry.available).count();
    let mut lines = Vec::with_capacity(BIO_BINARY_COUNT + 4);

    lines.push(format!("Bio-Binaries directory: {}", directory.display()));
    lines.push(format!("runtime state: {}", runtime_state_dir().display()));
    lines.push(format!(
        "integrity: {BIO_BINARY_COUNT}/{BIO_BINARY_COUNT} SHA-256 release pins embedded"
    ));
    for entry in availability {
        lines.push(format!(
            "{} [{}] {}",
            entry.binary.name,
            entry.binary.effect,
            if entry.available {
                "available"
            } else {
                "missing"
            }
        ));
    }
    lines.push(format!(
        "availability: {available}/{BIO_BINARY_COUNT} binaries"
    ));
    lines.join("\n")
}

pub fn execute(name: &str, input: &str, allow_mutation: bool) -> ExecutionOutcome {
    execute_in_dir(name, input, allow_mutation, &binary_dir())
}

fn execute_in_dir(
    name: &str,
    input: &str,
    allow_mutation: bool,
    directory: &Path,
) -> ExecutionOutcome {
    let Some(binary) = find(name) else {
        return unknown_binary(name);
    };

    if binary.effect.requires_authorization() && !allow_mutation {
        return ExecutionOutcome::failed(
            "bio_external_mutation_permission_required",
            format!(
                "[bio-external] refusing {} effect for '{}' without explicit mutation authorization",
                binary.effect, binary.name
            ),
        );
    }

    let args = match parse_arguments(input) {
        Ok(args) => args,
        Err(outcome) => return outcome,
    };

    let executable = binary_path_in_dir(binary, directory);
    if !executable.is_file() {
        return ExecutionOutcome::failed(
            "bio_external_binary_missing",
            format!(
                "[bio-external] '{}' is not available at {}",
                binary.name,
                executable.display()
            ),
        );
    }

    let Some(expected_hash) = expected_release_sha256(binary.name) else {
        return ExecutionOutcome::failed(
            "bio_external_integrity_pin_missing",
            format!(
                "[bio-external] no release SHA-256 pin exists for '{}'",
                binary.name
            ),
        );
    };
    let executable_bytes = match std::fs::read(&executable) {
        Ok(bytes) => bytes,
        Err(error) => {
            return ExecutionOutcome::failed(
                "bio_external_integrity_read_failed",
                format!(
                    "[bio-external] cannot hash '{}': {error}",
                    executable.display()
                ),
            )
        }
    };
    let actual_hash = hex::encode(Sha256::digest(&executable_bytes));
    if actual_hash != expected_hash {
        return ExecutionOutcome::failed(
            "bio_external_integrity_mismatch",
            format!(
                "[bio-external] SHA-256 mismatch for '{}': expected {expected_hash}, observed {actual_hash}",
                binary.name
            ),
        );
    }

    let runtime_state = runtime_state_dir();
    let temp_directory = runtime_state.join("temp");
    let integrity_directory = runtime_state.join("integrity").join(expected_hash);
    for directory in [&temp_directory, &integrity_directory] {
        if let Err(error) = std::fs::create_dir_all(directory) {
            return ExecutionOutcome::failed(
                "bio_external_state_create_failed",
                format!(
                    "[bio-external] cannot create runtime state at {}: {error}",
                    directory.display()
                ),
            );
        }
    }

    let Some(executable) = executable.to_str() else {
        return ExecutionOutcome::failed(
            "bio_external_binary_path_invalid",
            format!(
                "[bio-external] executable path is not valid Unicode: {}",
                executable.display()
            ),
        );
    };

    let spec = ProcessSpec::new(executable)
        .args(args)
        .cwd(directory)
        .timeout_ms(EXECUTION_TIMEOUT_MS)
        .max_output_bytes(MAX_OUTPUT_BYTES)
        .env("TEMP", temp_directory.to_string_lossy())
        .env("TMP", temp_directory.to_string_lossy())
        .env("BIO_INTEGRITY_DIR", integrity_directory.to_string_lossy())
        .no_color(true);
    let outcome = run_process_typed(&spec);
    if outcome.is_failed() {
        return outcome;
    }
    ExecutionOutcome::completed(format!(
        "[bio-binaries] name={} effect={} executable={}\n{}",
        binary.name, binary.effect, executable, outcome.output
    ))
}

fn parse_arguments(input: &str) -> Result<Vec<String>, ExecutionOutcome> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(ExecutionOutcome::failed(
            "bio_external_arguments_too_large",
            format!("[bio-external] argument input exceeds the {MAX_INPUT_BYTES}-byte limit"),
        ));
    }

    if input.is_empty() {
        return Ok(Vec::new());
    }

    let mut args = Vec::new();
    for raw_argument in input.split_terminator('\n') {
        if args.len() >= MAX_ARGUMENTS {
            return Err(ExecutionOutcome::failed(
                "bio_external_too_many_arguments",
                format!(
                    "[bio-external] more than {MAX_ARGUMENTS} newline-separated arguments supplied"
                ),
            ));
        }

        let argument = raw_argument.strip_suffix('\r').unwrap_or(raw_argument);
        if argument.as_bytes().contains(&0) {
            return Err(ExecutionOutcome::failed(
                "bio_external_argument_contains_nul",
                "[bio-external] arguments must not contain NUL bytes",
            ));
        }
        if argument.len() > MAX_ARGUMENT_BYTES {
            return Err(ExecutionOutcome::failed(
                "bio_external_argument_too_large",
                format!("[bio-external] one argument exceeds the {MAX_ARGUMENT_BYTES}-byte limit"),
            ));
        }
        args.push(argument.to_string());
    }
    Ok(args)
}

fn unknown_binary(name: &str) -> ExecutionOutcome {
    ExecutionOutcome::failed(
        "bio_external_unknown_binary",
        format!("[bio-external] unknown Bio-Binaries target: '{name}'"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalog_contains_exactly_33_unique_binary_names() {
        const EXPECTED_NAMES: [&str; BIO_BINARY_COUNT] = [
            "viral-infect",
            "hox-diff",
            "plasmid-dream",
            "plasmid-inject",
            "telepathy-sync",
            "telepathy-entangle",
            "eqm-pulse",
            "eqm-methy",
            "aether-excite",
            "aether-fabric",
            "borg-cube",
            "nexus-logic",
            "collective-sync",
            "brain-synapse",
            "brain-connectome",
            "wave-encoder",
            "wave-sculptor",
            "iron-resonate",
            "path-resonance",
            "grid-warp",
            "magneto-geo",
            "mycelium-spread",
            "homeostasis",
            "omega-master",
            "omega-point",
            "ribosome-synth",
            "wave-cryo-tx",
            "wave-cryo-rx",
            "mutation-sentinel",
            "magneto-acoustic",
            "wave-field",
            "vagus-nerve",
            "microscope-mem",
        ];
        let names: HashSet<&str> = catalog().iter().map(|binary| binary.name).collect();
        let expected: HashSet<&str> = EXPECTED_NAMES.into_iter().collect();
        assert_eq!(catalog().len(), BIO_BINARY_COUNT);
        assert_eq!(names.len(), BIO_BINARY_COUNT);
        assert_eq!(names, expected);
        let pins: Vec<_> = catalog()
            .iter()
            .map(|binary| expected_release_sha256(binary.name).expect("release pin"))
            .collect();
        assert_eq!(pins.len(), BIO_BINARY_COUNT);
        assert!(pins
            .iter()
            .all(|pin| pin.len() == 64 && pin.bytes().all(|b| b.is_ascii_hexdigit())));
    }

    #[test]
    fn write_and_control_effects_require_explicit_authorization() {
        let missing_dir = Path::new(r"Z:\octopus-tests\bio-binaries-missing");

        for name in ["viral-infect", "omega-master", "vagus-nerve"] {
            let outcome = execute_in_dir(name, "", false, missing_dir);
            assert!(outcome.is_failed());
            assert_eq!(
                outcome.code.as_deref(),
                Some("bio_external_mutation_permission_required")
            );
        }

        let read = execute_in_dir("hox-diff", "", false, missing_dir);
        assert_eq!(read.code.as_deref(), Some("bio_external_binary_missing"));
    }

    #[test]
    fn path_resolution_uses_override_and_platform_executable_suffix() {
        let directory = binary_dir_from_override(Some(OsString::from(r"C:\bio-tools")));
        assert_eq!(directory, PathBuf::from(r"C:\bio-tools"));

        let binary = find("omega-master").expect("catalog entry");
        let expected = directory.join(format!("omega-master{}", env::consts::EXE_SUFFIX));
        assert_eq!(binary_path_in_dir(binary, &directory), expected);
    }

    #[test]
    fn empty_override_falls_back_to_default_directory() {
        assert_eq!(
            binary_dir_from_override(Some(OsString::new())),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_BIO_BINARY_RELATIVE_DIR)
        );
    }

    #[test]
    fn missing_read_binary_returns_typed_failure_without_starting_a_process() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();
        let directory = env::temp_dir().join(format!(
            "octopus-bio-external-missing-{}-{unique}",
            std::process::id(),
        ));
        let outcome = execute_in_dir("hox-diff", "status", false, &directory);
        assert!(outcome.is_failed());
        assert_eq!(outcome.code.as_deref(), Some("bio_external_binary_missing"));
    }

    #[test]
    fn tampered_binary_is_refused_before_process_start() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();
        let directory = env::temp_dir().join(format!(
            "octopus-bio-external-tampered-{}-{unique}",
            std::process::id(),
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join(format!("hox-diff{}", env::consts::EXE_SUFFIX));
        std::fs::write(&path, b"not the pinned release binary").unwrap();

        let outcome = execute_in_dir("hox-diff", "", false, &directory);
        assert_eq!(
            outcome.code.as_deref(),
            Some("bio_external_integrity_mismatch")
        );

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_dir(directory);
    }

    #[test]
    fn newline_arguments_are_preserved_without_shell_tokenization() {
        let args = parse_arguments("--path\nD:\\a folder\\file.rs\n--literal=a b").unwrap();
        assert_eq!(
            args,
            vec!["--path", r"D:\a folder\file.rs", "--literal=a b"]
        );
    }
}
