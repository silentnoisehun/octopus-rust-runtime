/// wave-field — The self-organizing wave interference field.
///
/// No agent. No scheduler. No rules engine.
/// The field observes its own interference patterns and emits new waves.
/// The field decides.
use crate::output;
use crate::wave_field::{load_persisted_events, EmergentEvent, WaveField, MAX_PERSISTED_EVENTS};
use crate::wave_store::{now_ms, WaveStore};
use clap::{Parser, Subcommand};
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "wave-field",
    about = "Self-organizing wave interference field — the space decides"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current field snapshot
    Snapshot,
    /// List recent emergent events
    Events {
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Monitor field in real-time
    Monitor {
        #[arg(long, default_value = "1000")]
        tick_ms: u64,
    },
}

pub async fn dispatch(args: &[String]) -> Result<String, String> {
    let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;

    match cli.command {
        Commands::Snapshot => {
            output::banner("WAVE-FIELD", "Live Interference Field Snapshot", "◈");
            output::section("Field State");
            let mut store = WaveStore::new(crate::wave_store::default_path(), 10000);
            // Merge inbox first to see any pending injections
            store.merge_inbox();
            let ts = now_ms();
            let energy = store.energy_map(ts);
            let live = store.live_count(ts);

            output::kv(
                "Total Energy",
                &format!("{:.3}", energy.values().sum::<f32>()),
            );
            output::kv("Live Packets", &live.to_string());
            for (freq, e) in &energy {
                let interference = store.interference_score(*freq as f32, ts);
                output::kv(
                    &format!("{:.0}Hz", freq),
                    &format!(
                        "{:.3} ({:?}, {} packets)",
                        e, interference.pattern, interference.active_count
                    ),
                );
            }
            // Persist after merge
            store.persist().ok();
            output::summary("wave-field", "Field snapshot captured");
            Ok("".to_string())
        }
        Commands::Events { limit } => {
            let events = load_events_for_cli(&crate::wave_store::default_path(), limit)?;
            output::banner("WAVE-FIELD", "Emergent Events", "◈");
            output::section("Recent Events");
            output::kv("Limit", &limit.to_string());
            output::kv("Found", &events.len().to_string());
            if events.is_empty() {
                output::warn("No persisted emergent events");
            } else {
                for event in &events {
                    output::kv(
                        &event.timestamp.to_string(),
                        &format!(
                            "{} | {:.0}Hz amp={:.3} {:?} → {}",
                            event.rule_name,
                            event.trigger_freq,
                            event.trigger_amplitude,
                            event.pattern,
                            event.response_description
                        ),
                    );
                }
            }
            output::summary(
                "wave-field",
                &format!("{} persisted event(s), newest first", events.len()),
            );
            Ok("".to_string())
        }
        Commands::Monitor { tick_ms } => {
            output::banner("WAVE-FIELD", "Field Monitor (Live)", "◈");
            output::section("Starting Monitor");
            output::kv("Tick Interval", &format!("{}ms", tick_ms));
            output::kv(
                "Rules",
                &format!("{}", crate::wave_field::default_rules().len()),
            );
            eprintln!("[WAVE-FIELD] Monitor loop started (tick={}ms)", tick_ms);
            eprintln!(
                "[WAVE-FIELD] Inbox: {:?}",
                crate::wave_store::default_path().with_extension("inbox.bin")
            );

            let store = WaveStore::new(crate::wave_store::default_path(), 10000);
            let mut field = WaveField::new(store, tick_ms).map_err(|error| error.to_string())?;

            loop {
                // Real field tick: merge inbox → evaluate rules → decay → persist
                field.tick().map_err(|error| error.to_string())?;

                let snap = field.snapshot();

                // Log every 5 ticks or when there's energy
                if field.tick_count % 5 == 0 || snap.total_energy.abs() > 0.001 {
                    eprintln!(
                        "[WAVE-FIELD] Tick {}: Energy={:.3} Packets={} Bands=[{}]",
                        field.tick_count,
                        snap.total_energy,
                        snap.live_packets,
                        snap.bands
                            .iter()
                            .filter(|b| b.active_packets > 0)
                            .map(|b| format!("{}:{:.2}/{:?}", b.name, b.amplitude, b.pattern))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }

                // Log emergent events as they happen
                for event in field.recent_events(5) {
                    if event.timestamp > now_ms() - tick_ms * 2 {
                        eprintln!(
                            "[WAVE-FIELD] ⚡ EMERGENT: {} | {:.0}Hz amp={:.3} {:?} → {}",
                            event.rule_name,
                            event.trigger_freq,
                            event.trigger_amplitude,
                            event.pattern,
                            event.response_description
                        );
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(tick_ms)).await;
            }
        }
    }
}

pub(crate) fn load_events_for_cli(
    store_path: &Path,
    limit: usize,
) -> Result<Vec<EmergentEvent>, String> {
    if !(1..=MAX_PERSISTED_EVENTS).contains(&limit) {
        return Err(format!(
            "event limit must be between 1 and {MAX_PERSISTED_EVENTS}, observed {limit}"
        ));
    }

    let events = load_persisted_events(store_path).map_err(|error| error.to_string())?;
    Ok(events.into_iter().rev().take(limit).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wave_field::{EmergentEvent, WaveField};
    use crate::wave_store::InterferencePattern;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    fn test_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "bio-wave-field-cli-{name}-{}-{}.bin",
            std::process::id(),
            TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn cli_load_helper_returns_newest_events_and_validates_limit() {
        let path = test_path("events");
        let store = WaveStore::new(path.clone(), 100);
        let mut field = WaveField::with_rules(store, 100, Vec::new()).unwrap();
        for timestamp in 1..=3 {
            field.events.push(EmergentEvent {
                timestamp,
                rule_name: format!("rule-{timestamp}"),
                trigger_freq: 4.0,
                trigger_amplitude: 0.5,
                pattern: InterferencePattern::Constructive,
                response_description: "test".to_string(),
            });
        }
        field.persist_events().unwrap();

        let events = load_events_for_cli(&path, 2).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].timestamp, 3);
        assert_eq!(events[1].timestamp, 2);
        assert!(load_events_for_cli(&path, 0).is_err());
        assert!(load_events_for_cli(&path, MAX_PERSISTED_EVENTS + 1).is_err());

        let _ = std::fs::remove_file(path.with_extension("events.bin"));
    }
}
