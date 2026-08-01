use crate::{
    capabilities, run_pipeline_outcome, CapabilityProfile, ExecutionOutcome, VerificationGrade,
};
use rand::{rngs::OsRng, Rng};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarshalTaskClass {
    Homeostasis,
    Memory,
    Inspect,
    Diagnose,
    Modify,
    Verify,
    Document,
    Orchestrate,
}

impl fmt::Display for MarshalTaskClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Homeostasis => "homeostasis",
            Self::Memory => "memory",
            Self::Inspect => "inspect",
            Self::Diagnose => "diagnose",
            Self::Modify => "modify",
            Self::Verify => "verify",
            Self::Document => "document",
            Self::Orchestrate => "orchestrate",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarshalCandidate {
    pub topology: &'static str,
    pub weight: u32,
    pub write_capable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarshalPlan {
    pub task_class: MarshalTaskClass,
    pub candidates: Vec<MarshalCandidate>,
    pub selected_index: usize,
    pub entropy_draw: u32,
    pub total_weight: u32,
}

impl MarshalPlan {
    pub fn selected(&self) -> &MarshalCandidate {
        &self.candidates[self.selected_index]
    }

    pub fn render(&self) -> String {
        let mut lines = vec![
            "MARSHAL / PSI ROUTE".to_string(),
            format!("task_class={}", self.task_class),
            "policy=safe-ready-only; context=reference-only".to_string(),
            "profile=windows-offline; minimum_verification=tested".to_string(),
            "entropy=OS-CSPRNG (OsRng)".to_string(),
        ];
        for (index, candidate) in self.candidates.iter().enumerate() {
            lines.push(format!(
                "candidate[{}] weight={} write={} topology={}",
                index + 1,
                candidate.weight,
                candidate.write_capable,
                candidate.topology
            ));
        }
        lines.push(format!(
            "collapse={}/{} selected={} topology={}",
            self.entropy_draw,
            self.total_weight,
            self.selected_index + 1,
            self.selected().topology
        ));
        lines.join("\n")
    }
}

const INSPECT: &[MarshalCandidate] = &[
    MarshalCandidate {
        topology: "code-reader || diagnostics",
        weight: 45,
        write_capable: false,
    },
    MarshalCandidate {
        topology: "code-analysis || diagnostics",
        weight: 30,
        write_capable: false,
    },
    MarshalCandidate {
        topology: "code-reader + summarize || diagnostics",
        weight: 25,
        write_capable: false,
    },
];

const HOMEOSTASIS: &[MarshalCandidate] = &[
    MarshalCandidate {
        topology: "macrophage || diagnostics || immune-antibody",
        weight: 60,
        write_capable: false,
    },
    MarshalCandidate {
        topology: "macrophage || circuit-breaker || retry-policy",
        weight: 40,
        write_capable: false,
    },
];

const MEMORY: &[MarshalCandidate] = &[
    MarshalCandidate {
        topology: "synaptic-pruning-v2 || duplicate-detector || dna-hebbian",
        weight: 60,
        write_capable: false,
    },
    MarshalCandidate {
        topology: "synaptic-pruning || summarize || dna-hebbian",
        weight: 40,
        write_capable: false,
    },
];

const DIAGNOSE: &[MarshalCandidate] = &[
    MarshalCandidate {
        topology: "diagnostics || code-analysis",
        weight: 45,
        write_capable: false,
    },
    MarshalCandidate {
        topology: "duplicate-detector || code-quality",
        weight: 25,
        write_capable: false,
    },
    MarshalCandidate {
        topology: "code-reader || diagnostics + code-analysis",
        weight: 30,
        write_capable: false,
    },
];

const MODIFY: &[MarshalCandidate] = &[
    MarshalCandidate {
        topology: "pipeline-architect + rust-surgeon || diagnostics",
        weight: 55,
        write_capable: true,
    },
    MarshalCandidate {
        topology: "pipeline-architect + rust-surgeon || code-analysis",
        weight: 45,
        write_capable: true,
    },
];

const VERIFY: &[MarshalCandidate] = &[
    MarshalCandidate {
        topology: "diagnostics || bench-meter",
        weight: 35,
        write_capable: false,
    },
    MarshalCandidate {
        topology: "code-analysis || code-quality",
        weight: 35,
        write_capable: false,
    },
    MarshalCandidate {
        topology: "duplicate-detector || diagnostics",
        weight: 30,
        write_capable: false,
    },
];

const DOCUMENT: &[MarshalCandidate] = &[
    MarshalCandidate {
        topology: "summarize || brand-voice",
        weight: 55,
        write_capable: false,
    },
    MarshalCandidate {
        topology: "code-reader + summarize || diagnostics",
        weight: 45,
        write_capable: false,
    },
];

const ORCHESTRATE: &[MarshalCandidate] = &[
    MarshalCandidate {
        topology: "code-reader + architect-mind || diagnostics + code-analysis",
        weight: 45,
        write_capable: false,
    },
    MarshalCandidate {
        topology: "planner || diagnostics",
        weight: 35,
        write_capable: false,
    },
    MarshalCandidate {
        topology: "mitosis || planner || architect-mind",
        weight: 20,
        write_capable: false,
    },
];

fn keyword_score(text: &str, keywords: &[&str]) -> usize {
    keywords
        .iter()
        .filter(|keyword| text.contains(**keyword))
        .count()
}

pub fn classify_task(task: &str) -> MarshalTaskClass {
    let normalized = task.to_lowercase();
    let classes = [
        (
            MarshalTaskClass::Homeostasis,
            keyword_score(
                &normalized,
                &[
                    "memory leak",
                    "out of memory",
                    "zombie process",
                    "high cpu",
                    "disk full",
                    "network timeout",
                    "file corruption",
                    "deadlock",
                    "hung thread",
                    "panic",
                    "crash",
                    "unstable",
                    "memória sziv",
                    "zombi folyamat",
                    "magas cpu",
                    "megtelt a lemez",
                    "holtpont",
                    "összeoml",
                    "instabil",
                ],
            ),
        ),
        (
            MarshalTaskClass::Memory,
            keyword_score(
                &normalized,
                &[
                    "memory",
                    "context",
                    "cache",
                    "duplicate memory",
                    "repeated context",
                    "prune",
                    "synaptic",
                    "hebbian",
                    "stale context",
                    "memória",
                    "emlékezet",
                    "kontextus",
                    "ismétlődő",
                    "elavult",
                    "szinapt",
                ],
            ),
        ),
        (
            MarshalTaskClass::Modify,
            keyword_score(
                &normalized,
                &[
                    "implement",
                    "build",
                    "change",
                    "modify",
                    "remove",
                    "write",
                    "patch",
                    "fix",
                    "kódol",
                    "csinál",
                    "módos",
                    "javít",
                    "töröl",
                    "hozzáad",
                ],
            ),
        ),
        (
            MarshalTaskClass::Diagnose,
            keyword_score(
                &normalized,
                &[
                    "diagnose",
                    "debug",
                    "failure",
                    "crash",
                    "broken",
                    "hiba",
                    "hibá",
                    "nem működik",
                    "elcsúsz",
                    "roml",
                ],
            ),
        ),
        (
            MarshalTaskClass::Verify,
            keyword_score(
                &normalized,
                &[
                    "test",
                    "verify",
                    "audit",
                    "benchmark",
                    "soak",
                    "stability",
                    "teszt",
                    "ellenőriz",
                    "stabil",
                ],
            ),
        ),
        (
            MarshalTaskClass::Document,
            keyword_score(
                &normalized,
                &[
                    "document", "readme", "manual", "paper", "write-up", "dokument", "leírás",
                    "papír",
                ],
            ),
        ),
        (
            MarshalTaskClass::Orchestrate,
            keyword_score(
                &normalized,
                &[
                    "octopus",
                    "blade",
                    "parallel",
                    "marshal",
                    "swarm",
                    "orchestrat",
                    "polip",
                    "raj",
                    "marsall",
                    "párhuz",
                ],
            ),
        ),
    ];
    classes
        .into_iter()
        .filter(|(_, score)| *score > 0)
        .max_by_key(|(class, score)| (*score, class_priority(*class)))
        .map(|(class, _)| class)
        .unwrap_or(MarshalTaskClass::Inspect)
}

fn class_priority(class: MarshalTaskClass) -> usize {
    match class {
        MarshalTaskClass::Homeostasis => 8,
        MarshalTaskClass::Memory => 7,
        MarshalTaskClass::Modify => 6,
        MarshalTaskClass::Diagnose => 5,
        MarshalTaskClass::Verify => 4,
        MarshalTaskClass::Orchestrate => 3,
        MarshalTaskClass::Document => 2,
        MarshalTaskClass::Inspect => 1,
    }
}

fn candidates_for(class: MarshalTaskClass) -> &'static [MarshalCandidate] {
    match class {
        MarshalTaskClass::Homeostasis => HOMEOSTASIS,
        MarshalTaskClass::Memory => MEMORY,
        MarshalTaskClass::Inspect => INSPECT,
        MarshalTaskClass::Diagnose => DIAGNOSE,
        MarshalTaskClass::Modify => MODIFY,
        MarshalTaskClass::Verify => VERIFY,
        MarshalTaskClass::Document => DOCUMENT,
        MarshalTaskClass::Orchestrate => ORCHESTRATE,
    }
}

fn components(topology: &str) -> impl Iterator<Item = &str> {
    topology
        .split("||")
        .flat_map(|arm| arm.split('+'))
        .map(str::trim)
        .filter(|component| !component.is_empty())
}

fn ready_candidates(class: MarshalTaskClass) -> Vec<MarshalCandidate> {
    let catalog: HashMap<_, _> = capabilities()
        .into_iter()
        .map(|capability| (capability.name.clone(), capability))
        .collect();
    candidates_for(class)
        .iter()
        .filter(|candidate| {
            components(candidate.topology).all(|component| {
                catalog.get(component).is_some_and(|capability| {
                    CapabilityProfile::WindowsOffline.allows(capability)
                        && capability.verification >= VerificationGrade::Tested
                })
            })
        })
        .cloned()
        .collect()
}

fn select_index(candidates: &[MarshalCandidate], draw: u32) -> Result<(usize, u32), String> {
    let total = candidates
        .iter()
        .try_fold(0u32, |sum, candidate| sum.checked_add(candidate.weight))
        .ok_or_else(|| "marshal candidate weight overflow".to_string())?;
    if candidates.is_empty() || total == 0 {
        return Err("marshal has no ready weighted candidates".to_string());
    }
    let target = draw % total;
    let mut boundary = 0u32;
    for (index, candidate) in candidates.iter().enumerate() {
        boundary += candidate.weight;
        if target < boundary {
            return Ok((index, total));
        }
    }
    Err("marshal selection boundary failure".to_string())
}

fn plan_with_draw(task: &str, draw: u32) -> Result<MarshalPlan, String> {
    if task.trim().is_empty() {
        return Err("marshal task must not be empty".to_string());
    }
    let task_class = classify_task(task);
    let candidates = ready_candidates(task_class);
    let (selected_index, total_weight) = select_index(&candidates, draw)?;
    Ok(MarshalPlan {
        task_class,
        candidates,
        selected_index,
        entropy_draw: draw % total_weight,
        total_weight,
    })
}

pub fn plan(task: &str) -> Result<MarshalPlan, String> {
    if task.trim().is_empty() {
        return Err("marshal task must not be empty".to_string());
    }
    let task_class = classify_task(task);
    let candidates = ready_candidates(task_class);
    let total_weight = candidates
        .iter()
        .try_fold(0u32, |sum, candidate| sum.checked_add(candidate.weight))
        .ok_or_else(|| "marshal candidate weight overflow".to_string())?;
    if candidates.is_empty() || total_weight == 0 {
        return Err("marshal has no ready weighted candidates".to_string());
    }
    let draw = OsRng.gen_range(0..total_weight);
    plan_with_draw(task, draw)
}

pub fn plan_outcome(task: &str) -> ExecutionOutcome {
    match plan(task) {
        Ok(plan) => ExecutionOutcome::completed(plan.render()),
        Err(error) => ExecutionOutcome::failed("marshal_plan_failed", error),
    }
}

pub fn dispatch_outcome(task: &str, allow_write: bool) -> ExecutionOutcome {
    let plan = match plan(task) {
        Ok(plan) => plan,
        Err(error) => return ExecutionOutcome::failed("marshal_plan_failed", error),
    };
    if plan.selected().write_capable && !allow_write {
        return ExecutionOutcome::failed(
            "marshal_write_confirmation_required",
            format!(
                "{}\nexecution=refused; reason=write topology requires --allow-write",
                plan.render()
            ),
        );
    }
    let outcome = run_pipeline_outcome(plan.selected().topology, task);
    let rendered = format!(
        "{}\nexecution=dispatched\n\n{}",
        plan.render(),
        outcome.output
    );
    if outcome.is_failed() {
        ExecutionOutcome::failed(
            outcome
                .code
                .unwrap_or_else(|| "marshal_dispatch_failed".into()),
            rendered,
        )
    } else {
        ExecutionOutcome::completed(rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_understands_hungarian_and_prefers_mutation() {
        assert_eq!(
            classify_task("nézd meg a hibát"),
            MarshalTaskClass::Diagnose
        );
        assert_eq!(
            classify_task("javítsd meg és teszteld"),
            MarshalTaskClass::Modify
        );
        assert_eq!(
            classify_task("írj külön dokumentumot"),
            MarshalTaskClass::Document
        );
    }

    #[test]
    fn empty_task_is_rejected() {
        assert!(plan_with_draw("  ", 0).is_err());
    }

    #[test]
    fn weighted_collapse_respects_boundaries() {
        let first = plan_with_draw("inspect source", 0).unwrap();
        assert_eq!(first.selected_index, 0);
        let second = plan_with_draw("inspect source", 45).unwrap();
        assert_eq!(second.selected_index, 1);
        let last = plan_with_draw("inspect source", 99).unwrap();
        assert_eq!(last.selected_index, 2);
        assert_eq!(last.total_weight, 100);
    }

    #[test]
    fn every_candidate_uses_ready_capabilities() {
        for class in [
            MarshalTaskClass::Homeostasis,
            MarshalTaskClass::Memory,
            MarshalTaskClass::Inspect,
            MarshalTaskClass::Diagnose,
            MarshalTaskClass::Modify,
            MarshalTaskClass::Verify,
            MarshalTaskClass::Document,
            MarshalTaskClass::Orchestrate,
        ] {
            assert_eq!(ready_candidates(class), candidates_for(class));
        }
    }

    #[test]
    fn classifier_activates_bio_routes_for_incidents_and_memory_pressure() {
        assert_eq!(
            classify_task("fix the crash caused by a memory leak"),
            MarshalTaskClass::Homeostasis
        );
        assert_eq!(
            classify_task("prune repeated stale context from memory"),
            MarshalTaskClass::Memory
        );
        assert!(candidates_for(MarshalTaskClass::Homeostasis)
            .iter()
            .all(|candidate| candidate.topology.contains("macrophage")));
        assert!(candidates_for(MarshalTaskClass::Memory)
            .iter()
            .all(|candidate| candidate.topology.contains("dna-hebbian")));
    }

    #[test]
    fn receipt_is_compact_and_does_not_repeat_task() {
        let secret_task = "inspect SECRET-LONG-TASK-CONTENT";
        let rendered = plan_with_draw(secret_task, 0).unwrap().render();
        assert!(!rendered.contains("SECRET-LONG-TASK-CONTENT"));
        assert!(rendered.contains("context=reference-only"));
        assert!(rendered.contains("profile=windows-offline"));
        assert!(rendered.contains("minimum_verification=tested"));
        assert!(rendered.contains("entropy=OS-CSPRNG"));
    }

    #[test]
    fn writes_are_refused_without_explicit_permission() {
        let outcome = dispatch_outcome("javítsd a kódot", false);
        assert!(outcome.is_failed());
        assert_eq!(
            outcome.code.as_deref(),
            Some("marshal_write_confirmation_required")
        );
    }
}
