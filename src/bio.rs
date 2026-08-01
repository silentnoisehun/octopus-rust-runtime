use crate::ExecutionOutcome;
use std::collections::HashMap;

const MAX_RECORDS: usize = 64;
const MAX_RECEIPT_ITEMS: usize = 8;

struct ThreatSignature {
    name: &'static str,
    markers: &'static [&'static str],
    response: &'static str,
}

const THREATS: &[ThreatSignature] = &[
    ThreatSignature {
        name: "memory_leak",
        markers: &[
            "memory leak",
            "memory_leak",
            "out of memory",
            "oom",
            "memória sziv",
        ],
        response: "measure allocation growth; isolate the owner; release bounded caches; verify the slope",
    },
    ThreatSignature {
        name: "zombie_process",
        markers: &["zombie process", "zombie_process", "orphan process", "zombi folyamat"],
        response: "identify the parent and lease; request bounded termination; verify process exit",
    },
    ThreatSignature {
        name: "high_cpu",
        markers: &["high cpu", "high_cpu", "cpu spike", "100% cpu", "magas cpu"],
        response: "sample the hot path; bound concurrency; apply backpressure; re-measure load",
    },
    ThreatSignature {
        name: "disk_full",
        markers: &["disk full", "disk_full", "no space left", "megtelt a lemez"],
        response: "measure consumers; preserve protected state; rotate bounded logs; verify free space",
    },
    ThreatSignature {
        name: "network_timeout",
        markers: &["network timeout", "network_timeout", "connection timeout", "hálózati időtúllép"],
        response: "check reachability; open the circuit; apply bounded retry with jitter; verify recovery",
    },
    ThreatSignature {
        name: "file_corruption",
        markers: &["file corruption", "file_corruption", "checksum mismatch", "sérült fájl"],
        response: "quarantine the payload; verify the trusted hash; restore from sealed backup; re-verify",
    },
    ThreatSignature {
        name: "panic_crash",
        markers: &["panic", "crash", "fatal error", "összeoml", "kritikus hiba"],
        response: "capture the failure boundary; preserve state; isolate the trigger; validate a minimal repair",
    },
    ThreatSignature {
        name: "deadlock",
        markers: &["deadlock", "lock inversion", "hung thread", "beragadt szál", "holtpont"],
        response: "capture lock ownership; stop new work; break the cycle at one boundary; verify liveness",
    },
];

pub fn execute(name: &str, input: &str) -> Option<ExecutionOutcome> {
    match name {
        "macrophage" => Some(macrophage(input)),
        "immune-status" => Some(immune_status()),
        "immune-antibody" => Some(immune_antibody(input)),
        "immune-log" => Some(immune_log()),
        "synaptic-pruning" => Some(synaptic_pruning(input, false)),
        "synaptic-pruning-v2" => Some(synaptic_pruning(input, true)),
        "mitosis" => Some(mitosis(input)),
        "dna-hebbian" => Some(dna_hebbian(input)),
        _ => None,
    }
}

fn detect_threats(input: &str) -> Vec<&'static ThreatSignature> {
    let normalized = input.to_lowercase();
    THREATS
        .iter()
        .filter(|signature| {
            signature
                .markers
                .iter()
                .any(|marker| marker_matches(&normalized, marker))
        })
        .collect()
}

fn marker_matches(normalized: &str, marker: &str) -> bool {
    if marker.chars().all(|character| character.is_alphanumeric()) && marker.len() <= 3 {
        normalized
            .split(|character: char| !character.is_alphanumeric())
            .any(|token| token == marker)
    } else {
        normalized.contains(marker)
    }
}

fn macrophage(input: &str) -> ExecutionOutcome {
    if input.trim().is_empty() {
        return ExecutionOutcome::failed(
            "bio_input_missing",
            "[macrophage] input text or incident evidence is required",
        );
    }
    let threats = detect_threats(input);
    if threats.is_empty() {
        return ExecutionOutcome::completed(
            "[macrophage] scope=input-only signals=0 verdict=no-known-signature action=none"
                .to_string(),
        );
    }

    let mut output = format!(
        "[macrophage] scope=input-only signals={} verdict=attention-required action=advisory-only",
        threats.len()
    );
    for threat in threats {
        output.push_str(&format!(
            "\nfinding={} response={}",
            threat.name, threat.response
        ));
    }
    ExecutionOutcome::completed(output)
}

fn immune_status() -> ExecutionOutcome {
    ExecutionOutcome::completed(format!(
        "[immune-status] scope=capability-only live-health-claim=false detectors={} mode=advisory mutation=none",
        THREATS.len()
    ))
}

fn immune_antibody(input: &str) -> ExecutionOutcome {
    if input.trim().is_empty() {
        return ExecutionOutcome::failed(
            "bio_input_missing",
            "[immune-antibody] a threat description is required",
        );
    }
    let threats = detect_threats(input);
    if threats.is_empty() {
        return ExecutionOutcome::completed(
            "[immune-antibody] matched=unknown response=collect evidence; quarantine only after confirmation execution=advisory-only"
                .to_string(),
        );
    }

    let mut output = String::from("[immune-antibody] execution=advisory-only");
    for threat in threats {
        output.push_str(&format!(
            "\nmatched={} response={}",
            threat.name, threat.response
        ));
    }
    ExecutionOutcome::completed(output)
}

fn immune_log() -> ExecutionOutcome {
    ExecutionOutcome::completed(
        "[immune-log] source=runtime-persistence events=0 note=no immune event store is bound"
            .to_string(),
    )
}

fn records(input: &str) -> (Vec<String>, bool) {
    let mut primary: Vec<_> = input
        .lines()
        .flat_map(|line| line.split([';', ',']))
        .map(str::trim)
        .filter(|record| !record.is_empty())
        .take(MAX_RECORDS + 1)
        .map(str::to_string)
        .collect();
    let truncated = primary.len() > MAX_RECORDS;
    primary.truncate(MAX_RECORDS);
    if primary.is_empty() && !input.trim().is_empty() {
        (vec![input.trim().to_string()], false)
    } else {
        (primary, truncated)
    }
}

fn normalize_record(record: &str) -> String {
    record
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn synaptic_pruning(input: &str, enhanced: bool) -> ExecutionOutcome {
    let (records, truncated) = records(input);
    if records.is_empty() {
        return ExecutionOutcome::failed(
            "bio_input_missing",
            "[synaptic-pruning] context records are required",
        );
    }

    let mut order = Vec::new();
    let mut frequencies: HashMap<String, (String, usize)> = HashMap::new();
    for record in &records {
        let key = normalize_record(record);
        let entry = frequencies.entry(key.clone()).or_insert_with(|| {
            order.push(key.clone());
            (record.clone(), 0)
        });
        entry.1 += 1;
    }

    let kept = order.len();
    let pruned = records.len().saturating_sub(kept);
    let reinforced = frequencies.values().filter(|(_, count)| *count > 1).count();
    let decay_candidates = if enhanced {
        frequencies
            .values()
            .filter(|(record, count)| *count == 1 && record.split_whitespace().count() <= 2)
            .count()
    } else {
        0
    };
    let label = if enhanced {
        "synaptic-pruning-v2"
    } else {
        "synaptic-pruning"
    };
    let mut output = format!(
        "[{label}] input={} kept={} pruned={} reinforced={} decay_candidates={} truncated={} mutation=none",
        records.len(),
        kept,
        pruned,
        reinforced,
        decay_candidates,
        truncated
    );
    for key in order.into_iter().take(MAX_RECEIPT_ITEMS) {
        let (record, count) = &frequencies[&key];
        output.push_str(&format!("\npath_count={count} path={record}"));
    }
    ExecutionOutcome::completed(output)
}

fn mitosis(input: &str) -> ExecutionOutcome {
    if input.trim().is_empty() {
        return ExecutionOutcome::failed(
            "bio_input_missing",
            "[mitosis] code or task text is required",
        );
    }
    let functions = input.matches("fn ").count();
    let long_lines = input
        .lines()
        .filter(|line| line.chars().count() > 100)
        .count();
    let units: Vec<_> = input
        .lines()
        .flat_map(|line| line.split([';', '.']))
        .map(str::trim)
        .filter(|unit| !unit.is_empty())
        .take(MAX_RECEIPT_ITEMS)
        .collect();
    let suggested_arms = units.len().clamp(1, MAX_RECEIPT_ITEMS);
    let mut output = format!(
        "[mitosis] functions={} long_lines={} candidate_units={} suggested_arms={} mutation=none",
        functions,
        long_lines,
        units.len(),
        suggested_arms
    );
    for (index, unit) in units.iter().enumerate() {
        output.push_str(&format!("\nunit[{}]={unit}", index + 1));
    }
    ExecutionOutcome::completed(output)
}

fn dna_hebbian(input: &str) -> ExecutionOutcome {
    let (patterns, truncated) = records(input);
    if patterns.is_empty() {
        return ExecutionOutcome::failed(
            "bio_input_missing",
            "[dna-hebbian] at least one pattern is required",
        );
    }
    let pairs = patterns.len().saturating_mul(patterns.len());
    let mut output = format!(
        "[dna-hebbian] patterns={} synapses={} truncated={} mode=deterministic-association mutation=none",
        patterns.len(),
        pairs,
        truncated
    );
    for (index, pattern) in patterns.iter().take(MAX_RECEIPT_ITEMS).enumerate() {
        let self_weight = 1.0_f64;
        let neighbor_weight = if patterns.len() > 1 {
            1.0 / patterns.len() as f64
        } else {
            0.0
        };
        output.push_str(&format!(
            "\npattern[{}]={pattern} self={self_weight:.2} neighbor={neighbor_weight:.2}",
            index + 1
        ));
    }
    ExecutionOutcome::completed(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macrophage_detects_multiple_incident_signatures_without_claiming_mutation() {
        let outcome = macrophage("panic after a memory leak");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("finding=memory_leak"));
        assert!(outcome.output.contains("finding=panic_crash"));
        assert!(outcome.output.contains("action=advisory-only"));
    }

    #[test]
    fn macrophage_clean_result_is_scoped_to_input() {
        let outcome = macrophage("ordinary bounded work");
        assert!(outcome.output.contains("scope=input-only"));
        assert!(outcome.output.contains("no-known-signature"));
    }

    #[test]
    fn macrophage_does_not_match_oom_inside_an_unrelated_word() {
        let outcome = macrophage("inspect the room layout");
        assert!(
            outcome.output.contains("no-known-signature"),
            "{}",
            outcome.output
        );
        assert!(!outcome.output.contains("finding=memory_leak"));
    }

    #[test]
    fn pruning_reports_real_duplicate_counts() {
        let outcome = synaptic_pruning("alpha\nbeta\nalpha", true);
        assert!(outcome.output.contains("input=3"));
        assert!(outcome.output.contains("kept=2"));
        assert!(outcome.output.contains("pruned=1"));
        assert!(outcome.output.contains("reinforced=1"));
        assert!(outcome.output.contains("truncated=false"));
    }

    #[test]
    fn pruning_reports_when_the_bounded_input_was_truncated() {
        let input = (0..=MAX_RECORDS)
            .map(|index| format!("record-{index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let outcome = synaptic_pruning(&input, true);
        assert!(outcome.output.contains("input=64"), "{}", outcome.output);
        assert!(
            outcome.output.contains("truncated=true"),
            "{}",
            outcome.output
        );
    }

    #[test]
    fn mitosis_extracts_bounded_task_units() {
        let outcome = mitosis("inspect state; repair parser; verify tests");
        assert!(outcome.output.contains("candidate_units=3"));
        assert!(outcome.output.contains("suggested_arms=3"));
    }

    #[test]
    fn hebbian_handles_one_pattern_without_panicking() {
        let outcome = dna_hebbian("single pattern");
        assert!(!outcome.is_failed());
        assert!(outcome.output.contains("patterns=1"));
        assert!(outcome.output.contains("synapses=1"));
    }
}
