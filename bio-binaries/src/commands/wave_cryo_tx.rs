use crate::{acoustic, cryo, output};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "wave-cryo-tx",
    about = "Acoustic CryoFrame transmitter — BFSK modulated WAV output"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long = "echo-x")]
    pub echo_x: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Encode a compressed CryoFrame (.cryo) into a BFSK-modulated WAV
    Encode {
        /// Input compressed CryoFrame (.cryo)
        #[arg(long)]
        input: PathBuf,

        /// Output WAV file
        #[arg(long)]
        output: Option<PathBuf>,

        /// BFSK mark frequency for bit 1 (Hz)
        #[arg(long, default_value_t = acoustic::MARK_FREQ)]
        mark_hz: f64,

        /// BFSK space frequency for bit 0 (Hz)
        #[arg(long, default_value_t = acoustic::SPACE_FREQ)]
        space_hz: f64,

        /// Symbol rate; BFSK carries one bit per symbol
        #[arg(long, default_value_t = acoustic::SYMBOL_RATE)]
        baud: u32,

        /// PCM sample rate (Hz)
        #[arg(long, default_value_t = acoustic::SAMPLE_RATE)]
        sample_rate: u32,

        /// Replace an existing output after a staged encode succeeds
        #[arg(long)]
        force: bool,
    },
    /// Run a real in-process CryoFrame -> WAV -> CryoFrame verification cycle
    Test {
        /// Spectral capture duration used to build the self-test CryoFrame
        #[arg(long, default_value = "1000")]
        duration_ms: u64,
    },
}

#[derive(Debug, Serialize)]
pub struct TransmissionResult {
    pub timestamp: String,
    pub input_file: String,
    pub output_file: String,
    pub frame_hash: String,
    pub mark_hz: f64,
    pub space_hz: f64,
    pub baud: u32,
    pub sample_rate: u32,
    pub payload_bytes: usize,
    pub sample_count: usize,
    pub duration_secs: f64,
}

pub async fn dispatch(args: &[String]) -> Result<String, String> {
    let cli = Cli::try_parse_from(args).map_err(|error| error.to_string())?;
    if cli.echo_x.is_some() {
        return Err(
            "--echo-x reporting is not implemented for wave-cryo-tx; refusing to ignore it"
                .to_string(),
        );
    }

    match cli.command {
        Commands::Encode {
            input,
            output,
            mark_hz,
            space_hz,
            baud,
            sample_rate,
            force,
        } => {
            let config = validated_config(mark_hz, space_hz, baud, sample_rate)?;
            let output = output.unwrap_or_else(|| PathBuf::from("cryo_encoded.wav"));
            let (input, output, staged) = prepare_paths(&input, &output, force)?;

            let frame = cryo::load_frame_binary(&input)
                .map_err(|error| format!("Cannot load CryoFrame '{}': {error}", input.display()))?;
            if !cryo::verify_frame(&frame) {
                return Err(format!(
                    "CryoFrame integrity verification failed: {}",
                    input.display()
                ));
            }

            let encoded = match acoustic::encode_cryo_to_wav(&frame, &staged, &config) {
                Ok(encoded) => encoded,
                Err(error) => {
                    remove_staged(&staged);
                    return Err(format!("BFSK encoding failed: {error}"));
                }
            };

            let (samples, wav_rate) = match acoustic::read_wav(&staged) {
                Ok(wav) => wav,
                Err(error) => {
                    remove_staged(&staged);
                    return Err(format!("Staged WAV verification failed: {error}"));
                }
            };
            if wav_rate != config.sample_rate || samples.len() != encoded.sample_count {
                remove_staged(&staged);
                return Err(format!(
                    "Staged WAV metrics mismatch: rate={wav_rate}, samples={} (expected rate={}, samples={})",
                    samples.len(),
                    config.sample_rate,
                    encoded.sample_count
                ));
            }
            if let Err(error) = sync_file(&staged) {
                remove_staged(&staged);
                return Err(error);
            }
            commit_staged(&staged, &output, force)?;

            let result = TransmissionResult {
                timestamp: chrono::Utc::now().to_rfc3339(),
                input_file: input.to_string_lossy().into_owned(),
                output_file: output.to_string_lossy().into_owned(),
                frame_hash: encoded.frame_hash,
                mark_hz: config.mark_freq,
                space_hz: config.space_freq,
                baud: config.symbol_rate,
                sample_rate: config.sample_rate,
                payload_bytes: encoded.payload_bytes,
                sample_count: encoded.sample_count,
                duration_secs: encoded.duration_secs,
            };
            print_result(&result);
            Ok(String::new())
        }
        Commands::Test { duration_ms } => {
            if !(1..=5_000).contains(&duration_ms) {
                return Err("self-test duration must be between 1 and 5000 ms".to_string());
            }
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let wav_path = std::env::temp_dir().join(format!(
                "wave-cryo-self-test-{}-{nonce}.wav",
                std::process::id()
            ));
            let config = acoustic::AcousticConfig::default();
            let frame = cryo::freeze(
                0,
                vec!["wave-cryo-self-test".to_string()],
                duration_ms,
                duration_ms.max(1),
            );
            let encoded = acoustic::encode_cryo_to_wav(&frame, &wav_path, &config)
                .map_err(|error| format!("self-test encode failed: {error}"))?;
            let decoded = acoustic::decode_wav_to_cryo(&wav_path, &config);
            let _ = std::fs::remove_file(&wav_path);
            let (received, restored) =
                decoded.map_err(|error| format!("self-test decode failed: {error}"))?;
            if !received.crc_ok || !received.frame_valid || restored.frame_hash != frame.frame_hash
            {
                return Err("self-test roundtrip integrity mismatch".to_string());
            }

            output::banner("WAVE-CRYO-TX", "Verified Acoustic Self-Test", "◈");
            output::section("CryoFrame -> BFSK WAV -> CryoFrame");
            output::kv("Capture", &format!("{duration_ms}ms"));
            output::kv("Frame Hash", &frame.frame_hash);
            output::kv("Payload", &format!("{} bytes", encoded.payload_bytes));
            output::kv("Samples", &encoded.sample_count.to_string());
            output::kv("CRC", "verified");
            output::kv("Frame Integrity", "verified");
            output::success("Real acoustic roundtrip completed");
            output::summary("wave-cryo-tx", "Self-test verified");
            Ok(String::new())
        }
    }
}

fn validated_config(
    mark_hz: f64,
    space_hz: f64,
    baud: u32,
    sample_rate: u32,
) -> Result<acoustic::AcousticConfig, String> {
    if sample_rate == 0 {
        return Err("sample rate must be greater than zero".to_string());
    }
    if baud == 0 {
        return Err("baud/symbol rate must be greater than zero".to_string());
    }
    if !sample_rate.is_multiple_of(baud) {
        return Err(format!(
            "sample rate {sample_rate} must be an exact multiple of baud {baud}"
        ));
    }
    let samples_per_symbol = sample_rate / baud;
    if samples_per_symbol < 8 {
        return Err(format!(
            "sample rate/baud provides only {samples_per_symbol} samples per symbol; at least 8 are required"
        ));
    }

    let nyquist = sample_rate as f64 / 2.0;
    for (name, frequency) in [("mark", mark_hz), ("space", space_hz)] {
        if !frequency.is_finite() || frequency <= 0.0 {
            return Err(format!(
                "{name} frequency must be finite and greater than zero"
            ));
        }
        if frequency >= nyquist {
            return Err(format!(
                "{name} frequency {frequency}Hz must be below Nyquist ({nyquist}Hz)"
            ));
        }
    }
    if (mark_hz - space_hz).abs() < f64::EPSILON {
        return Err("mark and space frequencies must be distinct".to_string());
    }

    Ok(acoustic::AcousticConfig {
        sample_rate,
        mark_freq: mark_hz,
        space_freq: space_hz,
        symbol_rate: baud,
        amplitude: acoustic::DEFAULT_AMPLITUDE,
    })
}

fn prepare_paths(
    input: &Path,
    requested_output: &Path,
    force: bool,
) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    if !input.is_file() {
        return Err(format!("Input CryoFrame not found: {}", input.display()));
    }
    let input = std::fs::canonicalize(input)
        .map_err(|error| format!("Cannot resolve input '{}': {error}", input.display()))?;

    let requested_output = if requested_output.is_absolute() {
        requested_output.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("Cannot resolve current directory: {error}"))?
            .join(requested_output)
    };
    let file_name = requested_output.file_name().ok_or_else(|| {
        format!(
            "Output path has no file name: {}",
            requested_output.display()
        )
    })?;
    let parent = requested_output
        .parent()
        .ok_or_else(|| format!("Output path has no parent: {}", requested_output.display()))?;
    std::fs::create_dir_all(parent).map_err(|error| {
        format!(
            "Cannot create output directory '{}': {error}",
            parent.display()
        )
    })?;
    let parent = std::fs::canonicalize(parent).map_err(|error| {
        format!(
            "Cannot resolve output directory '{}': {error}",
            parent.display()
        )
    })?;
    let output = parent.join(file_name);

    if output.exists() {
        if !output.is_file() {
            return Err(format!(
                "Output exists but is not a file: {}",
                output.display()
            ));
        }
        let existing = std::fs::canonicalize(&output)
            .map_err(|error| format!("Cannot resolve output '{}': {error}", output.display()))?;
        if existing == input {
            return Err("input and output must be different files".to_string());
        }
        if !force {
            return Err(format!(
                "Output already exists: {} (use --force to replace it after a successful staged encode)",
                output.display()
            ));
        }
    } else if output == input {
        return Err("input and output must be different files".to_string());
    }

    let staged = unique_sibling(&output, "stage");
    Ok((input, output, staged))
}

fn unique_sibling(path: &Path, kind: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();
    path.with_file_name(format!(".{name}.{}.{stamp}.{kind}", std::process::id()))
}

fn sync_file(path: &Path) -> Result<(), String> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|error| format!("Cannot open staged output '{}': {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("Cannot flush staged output '{}': {error}", path.display()))
}

fn commit_staged(staged: &Path, output: &Path, force: bool) -> Result<(), String> {
    if output.exists() {
        if !force {
            remove_staged(staged);
            return Err(format!(
                "Output appeared during encoding: {}",
                output.display()
            ));
        }
        let backup = unique_sibling(output, "backup");
        std::fs::rename(output, &backup).map_err(|error| {
            remove_staged(staged);
            format!(
                "Cannot stage existing output '{}': {error}",
                output.display()
            )
        })?;
        if let Err(error) = std::fs::rename(staged, output) {
            let rollback = std::fs::rename(&backup, output);
            remove_staged(staged);
            return Err(match rollback {
                Ok(()) => format!("Cannot commit staged output; original restored: {error}"),
                Err(rollback_error) => format!(
                    "Cannot commit staged output ({error}) and cannot restore original ({rollback_error}); backup retained at {}",
                    backup.display()
                ),
            });
        }
        if let Err(error) = std::fs::remove_file(&backup) {
            eprintln!(
                "[WAVE-CRYO-TX] Warning: committed output, but backup cleanup failed at {}: {error}",
                backup.display()
            );
        }
        return Ok(());
    }

    std::fs::rename(staged, output).map_err(|error| {
        remove_staged(staged);
        format!("Cannot commit output '{}': {error}", output.display())
    })
}

fn remove_staged(path: &Path) {
    let _ = std::fs::remove_file(path);
}

fn print_result(result: &TransmissionResult) {
    output::banner("WAVE-CRYO-TX", "Acoustic CryoFrame Transmitter", "◈");
    output::section("Verified BFSK Encoding");
    output::kv("Input", &result.input_file);
    output::kv("Output", &result.output_file);
    output::kv("Frame Hash", &result.frame_hash);
    output::kv(
        "Frequencies",
        &format!(
            "mark={:.1}Hz space={:.1}Hz",
            result.mark_hz, result.space_hz
        ),
    );
    output::kv(
        "Signal",
        &format!("{} baud @ {}Hz", result.baud, result.sample_rate),
    );
    output::kv("Payload", &format!("{} bytes", result.payload_bytes));
    output::kv("Samples", &result.sample_count.to_string());
    output::kv("Duration", &format!("{:.3}s", result.duration_secs));
    output::success("CryoFrame encoded and WAV artifact committed");
    output::summary("wave-cryo-tx", "Verified BFSK artifact");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cli_signal_contract_matches_acoustic_defaults() {
        let cli = Cli::try_parse_from(["wave-cryo-tx", "encode", "--input", "input.cryo"])
            .expect("CLI defaults");
        let Commands::Encode {
            mark_hz,
            space_hz,
            baud,
            sample_rate,
            ..
        } = cli.command
        else {
            panic!("encode command expected");
        };
        let config =
            validated_config(mark_hz, space_hz, baud, sample_rate).expect("default config");
        let defaults = acoustic::AcousticConfig::default();
        assert_eq!(config.sample_rate, defaults.sample_rate);
        assert_eq!(config.mark_freq, defaults.mark_freq);
        assert_eq!(config.space_freq, defaults.space_freq);
        assert_eq!(config.symbol_rate, defaults.symbol_rate);
    }

    #[test]
    fn signal_contract_rejects_nyquist_and_rate_mismatch() {
        assert!(validated_config(4_000.0, 600.0, 100, 8_000).is_err());
        assert!(validated_config(1_200.0, 600.0, 333, 8_000).is_err());
        assert!(validated_config(1_200.0, 1_200.0, 100, 8_000).is_err());
    }

    #[tokio::test]
    async fn self_test_runs_a_real_verified_acoustic_roundtrip() {
        let args = vec![
            "wave-cryo-tx".to_string(),
            "test".to_string(),
            "--duration-ms".to_string(),
            "1".to_string(),
        ];
        dispatch(&args).await.unwrap();
    }
}
