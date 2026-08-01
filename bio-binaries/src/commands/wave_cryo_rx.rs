use crate::{acoustic, cryo, output};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "wave-cryo-rx",
    about = "Acoustic CryoFrame receiver — BFSK demodulation from WAV"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long = "echo-x")]
    pub echo_x: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Decode a BFSK WAV into a verified compressed CryoFrame (.cryo)
    Decode {
        /// Input BFSK WAV
        #[arg(long)]
        input: PathBuf,

        /// Output compressed CryoFrame (.cryo)
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

        /// Expected PCM sample rate (Hz)
        #[arg(long, default_value_t = acoustic::SAMPLE_RATE)]
        sample_rate: u32,

        /// Replace an existing output after a staged decode succeeds
        #[arg(long)]
        force: bool,
    },
    /// Timer-only monitor surface; it does not open an audio device or detect signals
    Monitor {
        /// Monitor interval to wait
        #[arg(long, default_value = "5000")]
        duration_ms: u64,
    },
}

#[derive(Debug, Serialize)]
pub struct DecodingResult {
    pub timestamp: String,
    pub input_file: String,
    pub output_file: String,
    pub frame_hash: String,
    pub mark_hz: f64,
    pub space_hz: f64,
    pub baud: u32,
    pub sample_rate: u32,
    pub payload_bytes: usize,
    pub decoded_bytes: usize,
    pub stored_bytes: usize,
    pub frame_valid: bool,
    pub crc_ok: bool,
}

pub async fn dispatch(args: &[String]) -> Result<String, String> {
    let cli = Cli::try_parse_from(args).map_err(|error| error.to_string())?;
    if cli.echo_x.is_some() {
        return Err(
            "--echo-x reporting is not implemented for wave-cryo-rx; refusing to ignore it"
                .to_string(),
        );
    }

    match cli.command {
        Commands::Decode {
            input,
            output,
            mark_hz,
            space_hz,
            baud,
            sample_rate,
            force,
        } => {
            let config = validated_config(mark_hz, space_hz, baud, sample_rate)?;
            let output = output.unwrap_or_else(|| PathBuf::from("cryo_decoded.cryo"));
            let (input, output, staged) = prepare_paths(&input, &output, force)?;

            let (_, wav_rate) = acoustic::read_wav(&input)
                .map_err(|error| format!("Cannot read input WAV '{}': {error}", input.display()))?;
            if wav_rate != config.sample_rate {
                return Err(format!(
                    "WAV sample rate is {wav_rate}Hz, but decoder contract requires {}Hz",
                    config.sample_rate
                ));
            }

            let (decoded, frame) = acoustic::decode_wav_to_cryo(&input, &config)
                .map_err(|error| format!("BFSK decoding failed: {error}"))?;
            if !decoded.crc_ok {
                return Err("BFSK frame CRC verification failed".to_string());
            }
            if !decoded.frame_valid || !cryo::verify_frame(&frame) {
                return Err("Decoded CryoFrame integrity verification failed".to_string());
            }

            let binary = cryo::encode_binary(&frame);
            let compressed = cryo::compress(&binary);
            if compressed.is_empty() {
                return Err("Decoded CryoFrame compression produced an empty artifact".to_string());
            }
            if let Err(error) = write_staged(&staged, &compressed) {
                remove_staged(&staged);
                return Err(error);
            }
            let persisted = match cryo::load_frame_binary(&staged) {
                Ok(frame) => frame,
                Err(error) => {
                    remove_staged(&staged);
                    return Err(format!("Staged CryoFrame verification failed: {error}"));
                }
            };
            if !cryo::verify_frame(&persisted) || persisted.frame_hash != frame.frame_hash {
                remove_staged(&staged);
                return Err("Staged CryoFrame hash verification failed".to_string());
            }
            commit_staged(&staged, &output, force)?;

            let result = DecodingResult {
                timestamp: chrono::Utc::now().to_rfc3339(),
                input_file: input.to_string_lossy().into_owned(),
                output_file: output.to_string_lossy().into_owned(),
                frame_hash: frame.frame_hash,
                mark_hz: config.mark_freq,
                space_hz: config.space_freq,
                baud: config.symbol_rate,
                sample_rate: config.sample_rate,
                payload_bytes: decoded.payload_bytes,
                decoded_bytes: binary.len(),
                stored_bytes: compressed.len(),
                frame_valid: decoded.frame_valid,
                crc_ok: decoded.crc_ok,
            };
            print_result(&result);
            Ok(String::new())
        }
        Commands::Monitor { duration_ms } => {
            output::banner("WAVE-CRYO-RX", "Timer-Only Monitor", "◈");
            output::section("Monitor Contract");
            output::kv("Duration", &format!("{duration_ms}ms"));
            output::warn(
                "No audio device is opened and no signal detection is performed in this mode",
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;
            output::summary(
                "wave-cryo-rx",
                "Monitor interval elapsed; detection not performed",
            );
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
        return Err(format!("Input WAV not found: {}", input.display()));
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
                "Output already exists: {} (use --force to replace it after a successful staged decode)",
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

fn write_staged(path: &Path, data: &[u8]) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| format!("Cannot create staged output '{}': {error}", path.display()))?;
    file.write_all(data)
        .map_err(|error| format!("Cannot write staged output '{}': {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("Cannot flush staged output '{}': {error}", path.display()))
}

fn commit_staged(staged: &Path, output: &Path, force: bool) -> Result<(), String> {
    if output.exists() {
        if !force {
            remove_staged(staged);
            return Err(format!(
                "Output appeared during decoding: {}",
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
                "[WAVE-CRYO-RX] Warning: committed output, but backup cleanup failed at {}: {error}",
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

fn print_result(result: &DecodingResult) {
    output::banner("WAVE-CRYO-RX", "Acoustic CryoFrame Receiver", "◈");
    output::section("Verified BFSK Decoding");
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
    output::kv(
        "Acoustic Payload",
        &format!("{} bytes", result.payload_bytes),
    );
    output::kv("Decoded Frame", &format!("{} bytes", result.decoded_bytes));
    output::kv(
        "Stored CryoFrame",
        &format!("{} bytes", result.stored_bytes),
    );
    output::kv("CRC", if result.crc_ok { "verified" } else { "failed" });
    output::kv(
        "Frame Integrity",
        if result.frame_valid {
            "verified"
        } else {
            "failed"
        },
    );
    output::success("BFSK payload decoded and verified CryoFrame artifact committed");
    output::summary("wave-cryo-rx", "Verified CryoFrame artifact");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cli_signal_contract_matches_acoustic_defaults() {
        let cli = Cli::try_parse_from(["wave-cryo-rx", "decode", "--input", "input.wav"])
            .expect("CLI defaults");
        let Commands::Decode {
            mark_hz,
            space_hz,
            baud,
            sample_rate,
            ..
        } = cli.command
        else {
            panic!("decode command expected");
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
    async fn command_pipeline_roundtrips_a_real_cryo_artifact() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "wave-cryo-command-roundtrip-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&root).unwrap();

        let frame = cryo::freeze(0, vec!["command-roundtrip".to_string()], 1, 1);
        let saved = cryo::save_frame(&frame, &root).unwrap();
        let wav = root.join("encoded.wav");
        let decoded = root.join("decoded.cryo");

        let tx_args = vec![
            "wave-cryo-tx".to_string(),
            "encode".to_string(),
            "--input".to_string(),
            saved.binary_path.clone(),
            "--output".to_string(),
            wav.to_string_lossy().to_string(),
        ];
        crate::commands::wave_cryo_tx::dispatch(&tx_args)
            .await
            .unwrap();

        let rx_args = vec![
            "wave-cryo-rx".to_string(),
            "decode".to_string(),
            "--input".to_string(),
            wav.to_string_lossy().to_string(),
            "--output".to_string(),
            decoded.to_string_lossy().to_string(),
        ];
        dispatch(&rx_args).await.unwrap();

        let restored = cryo::load_frame_binary(&decoded).unwrap();
        assert_eq!(restored.frame_hash, frame.frame_hash);
        assert!(cryo::verify_frame(&restored));

        let _ = std::fs::remove_dir_all(root);
    }
}
