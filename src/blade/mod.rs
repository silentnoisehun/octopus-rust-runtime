//! Natív blade dispatch — közvetlen Rust függvényhívás
//! A Bicska batch implementációk natívan Hope-ban

pub mod batch1;
pub mod batch10;
pub mod batch11;
pub mod batch12;
pub mod batch2;
pub mod batch3;
pub mod batch4;
pub mod batch5;
pub mod batch6;
pub mod batch7;
pub mod batch8;
pub mod batch9;

use batch1::Batch1;
use batch10::Batch10;
use batch11::Batch11;
use batch12::Batch12;
use batch2::Batch2;
use batch3::Batch3;
use batch4::Batch4;
use batch5::Batch5;
use batch6::Batch6;
use batch7::Batch7;
use batch8::Batch8;
use batch9::Batch9;

/// Natív blade dispatch
pub fn execute(blade: &str, prompt: &str) -> String {
    match blade {
        // batch1
        "code-reader" => Batch1::code_reader(prompt),
        "code-writer" => Batch1::code_writer(prompt),
        "summarize" => Batch1::summarize(prompt),
        "web-research" => Batch1::web_research(prompt),
        "sag" => Batch1::sag(prompt),
        "code-analysis" => Batch1::code_analysis(prompt),
        "diagnostics" => Batch1::diagnostics(prompt),
        "audio-diagnostics" => Batch1::audio_diagnostics(prompt),
        "openai-image-gen" => Batch1::openai_image_gen(prompt),
        "openai-whisper" => Batch1::openai_whisper(prompt),
        "sherpa-onnx-tts" => Batch1::sherpa_onnx_tts(prompt),
        "tts-voice" => Batch1::tts_voice(prompt),
        "stt-ear" => Batch1::stt_ear(prompt),
        "mermaid-agent" => Batch1::mermaid_agent(prompt),
        "github" => Batch1::github(prompt),
        "github-manager" => Batch1::github_manager(prompt),
        "git-nexus" => Batch1::git_nexus(prompt),
        "notion" => Batch1::notion(prompt),
        "discord" => Batch1::discord(prompt),
        "himalaya" => Batch1::himalaya(prompt),
        "1password" => Batch1::onepassword(prompt),

        // batch2
        "canvas" => Batch2::canvas(prompt),
        "canvas-design" => Batch2::canvas_design(prompt),
        "frontend-design" => Batch2::frontend_design(prompt),
        "ui-design-system" => Batch2::ui_design_system(prompt),
        "ui-ux-pro" => Batch2::ui_ux_pro_max(prompt),
        "theme-factory" => Batch2::theme_factory(prompt),
        "brand-guidelines" => Batch2::brand_guidelines(prompt),
        "brand-voice" => Batch2::brand_voice(prompt),
        "brand-writer" => Batch2::brand_writer(prompt),
        "prose" => Batch2::prose(prompt),
        "writing-rules" => Batch2::writing_rules(prompt),
        "doc-scribe" => Batch2::doc_scribe(prompt),
        "document-agent" => Batch2::document_agent(prompt),
        "agent-development" => Batch2::agent_development(prompt),
        "hook-development" => Batch2::hook_development(prompt),
        "plugin-structure" => Batch2::plugin_structure(prompt),
        "command-development" => Batch2::command_development(prompt),
        "testing-codegen" => Batch2::testing_codegen(prompt),
        "test-tui" => Batch2::test_tui(prompt),
        "mintlify" => Batch2::mintlify(prompt),

        // batch3
        "memory-skills" => Batch3::memory_skills(prompt),
        "memory-skills-v2" => Batch3::memory_skills_v2(prompt),
        "microscope-memory" => Batch3::microscope_memory(prompt),
        "emoti-mem" => Batch3::emoti_memory(prompt),
        "claude-logic" => Batch3::claude_logic(prompt),
        "claude-psi" => Batch3::claude_psi(prompt),
        "psi-logic" => Batch3::psi_logic(prompt),
        "psi-quantum" => Batch3::psi_quantum(prompt),
        "psi" => Batch3::psi_framework(prompt),
        "architect-mind" => Batch3::architect_mind(prompt),
        "senior-architect" => Batch3::senior_architect(prompt),
        "senior-prompt-engineer" => Batch3::senior_prompt_engineer(prompt),
        "planner" => Batch3::planner(prompt),
        "memory-bank" => Batch3::memory_bank(prompt),
        "rust-surgeon" => Batch3::rust_surgeon(prompt),
        "omni-surgeon" => Batch3::omni_surgeon(prompt),
        "file-surgeon" => Batch3::file_surgeon(prompt),
        "formatter" => Batch3::formatter(prompt),
        "stem-core" => Batch3::stem_core(prompt),
        "omni-connector" => Batch3::omni_connector(prompt),

        // batch4
        "parser" => Batch4::parser(prompt),
        "type-inference" => Batch4::type_inference(prompt),
        "lint-rules" => Batch4::lint_rules(prompt),
        "crispr-hotfix" => Batch4::crispr_hotfix(prompt),
        "crispr-hotfix-v2" => Batch4::crispr_hotfix_v2(prompt),
        "synaptic-pruning" => Batch4::synaptic_pruning(prompt),
        "synaptic-pruning-v2" => Batch4::synaptic_pruning_v2(prompt),
        "viral-transduction" => Batch4::viral_transduction(prompt),
        "hox-architecture" => Batch4::hox_architecture(prompt),
        "ai-synapse" => Batch4::ai_synapse(prompt),
        "hive-orchestrator" => Batch4::hive_orchestrator(prompt),
        "maestro" => Batch4::maestro_orchestration(prompt),
        "swarm" => Batch4::swarm_coordination(prompt),
        "colony-swarm" => Batch4::colony_swarm(prompt),
        "quality-bun" => Batch4::quality_feature_delivery(prompt),
        "react-practices" => Batch4::react_practices(prompt),
        "stem-cell-manager" => Batch4::stemcell_manager(prompt),
        "mitosis-agent" => Batch4::mitosis_agent(prompt),
        "blogwatcher" => Batch4::blogwatcher(prompt),
        "peekaboo" => Batch4::peekaboo(prompt),

        // batch5
        "merge-pr" => Batch5::merge_pr(prompt),
        "merge-pr-v1" => Batch5::merge_pr_v1(prompt),
        "review-pr" => Batch5::review_pr(prompt),
        "still-archive" => Batch5::still_archive(prompt),
        "eightctl" => Batch5::eightctl(prompt),
        "clawhub" => Batch5::clawhub(prompt),
        "wacli" => Batch5::wacli(prompt),
        "goplaces" => Batch5::goplaces(prompt),
        "local-places" => Batch5::local_places(prompt),
        "weather" => Batch5::weather(prompt),
        "web-extractor" => Batch5::web_extractor(prompt),
        "lobster-scraper" => Batch5::lobster_scraper(prompt),
        "nano-pdf" => Batch5::nano_pdf(prompt),
        "pptx" => Batch5::pptx_handler(prompt),
        "gog" => Batch5::gog_integration(prompt),
        "tmux" => Batch5::tmux_integration(prompt),
        "turborepo" => Batch5::turborepo_handler(prompt),
        "brainstorming" => Batch5::brainstorming(prompt),
        "voice-call" => Batch5::voice_call(prompt),
        "incubator" => Batch5::incubator(prompt),

        // batch6
        "video-frames" => Batch6::video_frames(prompt),
        "bench-meter" => Batch6::bench_meter(prompt),
        "forge-blade" => Batch6::forge_blade(prompt),
        "mcporter" => Batch6::mcporter(prompt),
        "apple-notes" => Batch6::apple_notes(prompt),
        "bear-notes" => Batch6::bear_notes(prompt),
        "hello-mate" => Batch6::hello_mate(prompt),
        "omega-striker" => Batch6::omega_striker(prompt),
        "sigma" => Batch6::sigma(prompt),
        "data-master" => Batch6::data_master(prompt),
        "model-usage" => Batch6::model_usage(prompt),
        "claude-migration" => Batch6::claude_migration(prompt),

        // batch7 — Rongyász AST surgery
        "ast-refactor" => Batch7::ast_refactor(prompt),
        "code-quality" => Batch7::code_quality(prompt),
        "connectome" => Batch7::connectome(prompt, "rust"),
        "connectome-rs" => Batch7::connectome(prompt, "rust"),
        "connectome-py" => Batch7::connectome(prompt, "python"),
        "connectome-js" => Batch7::connectome(prompt, "javascript"),
        "duplicate-detector" => Batch7::duplicate_detector(prompt),
        "safety-check" => Batch7::safety_check(prompt, "rust"),
        "safety-check-py" => Batch7::safety_check(prompt, "python"),
        "safety-check-js" => Batch7::safety_check(prompt, "javascript"),

        // batch8 — Polyglot + Resilience + Immune + Plugin
        "polyglot" => Batch8::detect_language(prompt),
        "polyglot-metrics" => Batch8::polyglot_metrics(prompt),
        "polyglot-convert" => {
            // Format: "python\nto\nrust\ncode..."
            let parts: Vec<&str> = prompt.splitn(3, '\n').collect();
            if parts.len() >= 3 {
                Batch8::polyglot_convert(parts[2], parts[0], parts[1])
            } else {
                "[polyglot-convert] Használat: python\nrust\n<kód>".to_string()
            }
        }
        "circuit-breaker" => Batch8::circuit_breaker(prompt),
        "retry-policy" => {
            let parts: Vec<&str> = prompt.split_whitespace().collect();
            let max = parts.get(0).and_then(|s| s.parse::<u32>().ok()).unwrap_or(3);
            let ms = parts.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(100);
            Batch8::retry_policy(max, ms)
        }
        "graceful-shutdown" => {
            let ms = prompt.trim().parse::<u64>().unwrap_or(5000);
            Batch8::graceful_shutdown(ms)
        }
        "immune-status" => Batch8::immune_status(),
        "immune-antibody" => Batch8::immune_antibody(prompt),
        "immune-log" => {
            let n = prompt.trim().parse::<u32>().unwrap_or(5);
            Batch8::immune_log(n)
        }
        "plugin-list" => Batch8::plugin_list(),
        "plugin-install" => {
            let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
            let name = parts.get(0).unwrap_or(&"unknown").trim();
            let source = parts.get(1).unwrap_or(&"local").trim();
            Batch8::plugin_install(name, source)
        }
        "plugin-remove" => Batch8::plugin_remove(prompt.trim()),

        // batch9 — Evolúció + Mitózis + OmniCoder + Code Review + Agent + Swarm + Replicator
        "dreamer-loop" => Batch9::dreamer_loop(prompt),
        "auto-evolve" => Batch9::auto_evolve(prompt),
        "adaptive-evolve" => Batch9::adaptive_evolve(prompt),
        "self-evolve" => Batch9::self_evolve(prompt),
        "mitosis" => Batch9::mitosis(prompt),
        "bio-mitosis" => Batch9::bio_mitosis(prompt),
        "metamorphic-trigger" => {
            let gens = prompt.trim().parse::<u32>().unwrap_or(10);
            Batch9::metamorphic_trigger(gens)
        }
        "omnicoder" => {
            let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
            let mode = parts.get(0).unwrap_or(&"refactor").trim();
            let code = parts.get(1).unwrap_or(&"").trim();
            Batch9::omnicoder(code, mode)
        }
        "code-review" => Batch9::code_review(prompt),
        "agent-factory" => {
            let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
            let agent_type = parts.get(0).unwrap_or(&"generic").trim();
            let caps = parts.get(1).unwrap_or(&"").trim();
            Batch9::agent_factory(agent_type, caps)
        }
        "commander" => {
            let parts: Vec<&str> = prompt.splitn(2, ' ').collect();
            let cmd = parts.get(0).unwrap_or(&"").trim();
            let args = parts.get(1).unwrap_or(&"").trim();
            Batch9::commander(cmd, args)
        }
        "swarm-queen" => {
            let n = prompt.trim().parse::<u32>().unwrap_or(5);
            Batch9::swarm_queen(n)
        }
        "replicator" => {
            let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
            let target = parts.get(0).unwrap_or(&"local").trim();
            let code = parts.get(1).unwrap_or(&"").trim();
            Batch9::replicator(code, target)
        }

        // batch10 — Hope-Os modulok
        "vision-analyze" => Batch10::vision_analyze(prompt),
        "vision-compare" => {
            let parts: Vec<&str> = prompt.splitn(2, ' ').collect();
            let img1 = parts.get(0).unwrap_or(&"img1.png").trim();
            let img2 = parts.get(1).unwrap_or(&"img2.png").trim();
            Batch10::vision_compare(img1, img2)
        }
        "vision-ocr" => Batch10::vision_ocr(prompt),
        "geolocation-lookup" => Batch10::geolocation_lookup(prompt),
        "geolocation-distance" => Batch10::geolocation_distance(prompt),
        "geolocation-memory-map" => Batch10::geolocation_memory_map(prompt),
        "navigation-route" => Batch10::navigation_route(prompt),
        "navigation-poi" => {
            let parts: Vec<&str> = prompt.splitn(2, ' ').collect();
            let query = parts.get(0).unwrap_or(&"").trim();
            let location = parts.get(1).unwrap_or(&"").trim();
            Batch10::navigation_poi(query, location)
        }
        "collective-decision" => Batch10::collective_decision(prompt),
        "collective-consciousness" => {
            let n = prompt.trim().parse::<u32>().unwrap_or(5);
            Batch10::collective_consciousness(n)
        }
        "distributed-raft" => {
            let parts: Vec<&str> = prompt.split_whitespace().collect();
            let nodes = parts.get(0).and_then(|s| s.parse::<u32>().ok()).unwrap_or(5);
            let id = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
            Batch10::distributed_raft(nodes, id)
        }
        "distributed-lock" => {
            let parts: Vec<&str> = prompt.split_whitespace().collect();
            let resource = parts.get(0).unwrap_or(&"resource").trim();
            let timeout = parts.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(5000);
            Batch10::distributed_lock(resource, timeout)
        }
        "alan-self-code" => {
            let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
            let code = parts.get(0).unwrap_or(&"").trim();
            let instruction = parts.get(1).unwrap_or(&"").trim();
            Batch10::alan_self_code(code, instruction)
        }
        "alan-learn" => {
            let parts: Vec<&str> = prompt.splitn(2, ' ').collect();
            let pattern = parts.get(0).unwrap_or(&"pattern").trim();
            let hours = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(24);
            Batch10::alan_learn(pattern, hours)
        }
        "templates-refactor" => {
            let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
            let template = parts.get(0).unwrap_or(&"extract-method").trim();
            let code = parts.get(1).unwrap_or(&"").trim();
            Batch10::templates_refactor(template, code)
        }
        "templates-list" => Batch10::templates_list(),
        "pollinations-generate" => Batch10::pollinations_generate(prompt),
        "pollinations-memory-viz" => Batch10::pollinations_memory_visualize(prompt),
        // QR kód
        "qr-generate" => Batch10::qr_generate(prompt),
        "qr-spine" => Batch10::qr_spine(prompt),
        "qr-scan" => Batch10::qr_scan(prompt),
        "cryo-snap" => Batch10::cryo_snap(prompt),

        // batch11 — CodeDNA (LLM-mentes evolúciós tanulás)
        "dna-extract" => Batch11::dna_extract(prompt),
        "dna-mutate" => Batch11::dna_mutate(prompt, "all"),
        "dna-mutate-point" => Batch11::dna_mutate(prompt, "point"),
        "dna-mutate-insert" => Batch11::dna_mutate(prompt, "insertion"),
        "dna-mutate-delete" => Batch11::dna_mutate(prompt, "deletion"),
        "dna-mutate-optimize" => Batch11::dna_mutate(prompt, "optimization"),
        "dna-crossover" => {
            let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
            let code1 = parts.get(0).unwrap_or(&"").trim();
            let code2 = parts.get(1).unwrap_or(&"").trim();
            Batch11::dna_crossover(code1, code2)
        }
        "dna-select" => Batch11::dna_select(prompt),
        "dna-evolve" => {
            let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
            let code = parts.get(0).unwrap_or(&"").trim();
            let gens = parts.get(1).and_then(|s| s.trim().parse::<u32>().ok()).unwrap_or(5);
            Batch11::dna_evolve(code, gens)
        }
        "dna-teach" => Batch11::dna_teach(prompt),
        "dna-hebbian" => Batch11::dna_hebbian(prompt),
        "dna-stats" => Batch11::dna_stats(prompt),
        "brain" => {
            let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
            let mode = parts.get(0).unwrap_or(&"analyze").trim();
            let code = parts.get(1).unwrap_or(&"").trim();
            Batch11::brain_process(code, mode)
        }
        "brain-compare" => Batch11::brain_compare(),

        // batch12 — Dual Generation (Silent Worker Teaching Method)
        "dual-generate" => Batch12::dual_generate(prompt),
        "dual-cache" => Batch12::dual_cache(prompt),
        "dual-learn" => Batch12::dual_learn(prompt),
        "dual-record" => Batch12::dual_record(prompt),
        "dual-status" => Batch12::dual_status(prompt),
        "dual-teach" => Batch12::dual_teach(prompt),

        // legacy aliases
        "02_Memory_Skills" => Batch3::memory_skills(prompt),
        "crispr_hotfix" => Batch4::crispr_hotfix(prompt),
        "synaptic_pruning" => Batch4::synaptic_pruning(prompt),
        "macrophage" => Batch4::crispr_hotfix(prompt), // closest match

        "list" | "--list" | "-l" => {
            let blades = list();
            format!("Elérhető blade-ek ({}):\n  {}", blades.len(), blades.join(", "))
        }
        _ => format!("[{}] Ismeretlen blade. Elérhető: code-reader, summarize, diagnostics, parser, architect-mind, emoti-mem, github, canvas, prose, memory-skills, planner, rust-surgeon, stb.", blade),
    }
}

pub fn execute_resilient(spec: &str, prompt: &str) -> String {
    let candidates: Vec<&str> = spec
        .split('|')
        .map(str::trim)
        .filter(|candidate| !candidate.is_empty())
        .collect();

    if candidates.is_empty() {
        return "Üres blade failover lánc".to_string();
    }

    let requested = candidates[0];
    let mut failures = Vec::new();
    for candidate in candidates {
        let result = std::panic::catch_unwind(|| execute(candidate, prompt));
        match result {
            Ok(output) if !output.contains("Ismeretlen blade") && !output.trim().is_empty() => {
                if candidate == requested {
                    return output;
                }
                return format!("──◆ failover {requested} -> {candidate} ──✓\n{output}");
            }
            Ok(_) => failures.push(format!("{candidate}: unavailable")),
            Err(_) => failures.push(format!("{candidate}: panic isolated")),
        }
    }

    format!("Blade arm failed: {}", failures.join(", "))
}

/// Pipeline: blade-ek Merkle-fűzése, párhuzamos végrehajtás
/// Formátum: "rust-surgeon + test-worker" vagy "code-reader + summarize"
/// Minden blade párhuzamosan fut, az eredmények Merkle-fűzve
#[allow(dead_code)]
pub fn pipeline(spec: &str, prompt: &str) -> String {
    let blades: Vec<&str> = spec
        .split('+')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if blades.is_empty() {
        return "Üres pipeline".to_string();
    }
    if blades.len() == 1 {
        return execute_resilient(blades[0], prompt);
    }

    // Párhuzamos végrehajtás
    let mut results: Vec<(String, String)> = Vec::new();
    let mut handles = Vec::new();

    for blade in &blades {
        let b = blade.to_string();
        let p = prompt.to_string();
        handles.push(std::thread::spawn(move || {
            let result = execute_resilient(&b, &p);
            (b, result)
        }));
    }

    for handle in handles {
        results.push(handle.join().unwrap_or_default());
    }

    // Merkle-fűzés: minden eredményt kriptográfiailag összekötünk
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let mut output = format!("═══ PIPELINE: {} ═══\n", blades.join(" + "));

    for (i, (blade, result)) in results.iter().enumerate() {
        let block = format!("[{}] {}", blade, result);
        hasher.update(block.as_bytes());
        let hash = hex::encode(hasher.finalize_reset());
        output.push_str(&format!(
            "\n── Step {}: {} ──\n{}\nMerkle: {}\n",
            i + 1,
            blade,
            result,
            &hash[..16]
        ));
    }

    // Végső Merkle root
    hasher.update(output.as_bytes());
    let root = hex::encode(hasher.finalize());
    output.push_str(&format!("\n═══ Merkle Root: {} ═══", &root[..16]));

    output
}

/// Elérhető blade-ek listája
pub fn list() -> Vec<&'static str> {
    vec![
        "code-reader",
        "code-writer",
        "summarize",
        "web-research",
        "sag",
        "code-analysis",
        "diagnostics",
        "audio-diagnostics",
        "openai-image-gen",
        "openai-whisper",
        "sherpa-onnx-tts",
        "tts-voice",
        "stt-ear",
        "mermaid-agent",
        "github",
        "github-manager",
        "git-nexus",
        "notion",
        "discord",
        "himalaya",
        "1password",
        "canvas",
        "canvas-design",
        "frontend-design",
        "ui-design-system",
        "ui-ux-pro",
        "theme-factory",
        "brand-guidelines",
        "brand-voice",
        "brand-writer",
        "prose",
        "writing-rules",
        "doc-scribe",
        "document-agent",
        "agent-development",
        "hook-development",
        "plugin-structure",
        "command-development",
        "testing-codegen",
        "test-tui",
        "memory-skills",
        "memory-skills-v2",
        "microscope-memory",
        "emoti-mem",
        "claude-logic",
        "claude-psi",
        "psi-logic",
        "psi-quantum",
        "psi",
        "architect-mind",
        "senior-architect",
        "senior-prompt-engineer",
        "planner",
        "memory-bank",
        "rust-surgeon",
        "omni-surgeon",
        "file-surgeon",
        "formatter",
        "stem-core",
        "omni-connector",
        "mintlify",
        "parser",
        "type-inference",
        "lint-rules",
        "crispr-hotfix",
        "crispr-hotfix-v2",
        "synaptic-pruning",
        "synaptic-pruning-v2",
        "viral-transduction",
        "hox-architecture",
        "ai-synapse",
        "hive-orchestrator",
        "maestro",
        "swarm",
        "colony-swarm",
        "quality-bun",
        "react-practices",
        "stem-cell-manager",
        "mitosis-agent",
        "blogwatcher",
        "peekaboo",
        "merge-pr",
        "merge-pr-v1",
        "review-pr",
        "still-archive",
        "eightctl",
        "clawhub",
        "wacli",
        "goplaces",
        "local-places",
        "weather",
        "web-extractor",
        "lobster-scraper",
        "nano-pdf",
        "pptx",
        "gog",
        "tmux",
        "turborepo",
        "brainstorming",
        "voice-call",
        "incubator",
        "video-frames",
        "bench-meter",
        "forge-blade",
        "mcporter",
        "apple-notes",
        "bear-notes",
        "hello-mate",
        "omega-striker",
        "sigma",
        "data-master",
        "model-usage",
        "claude-migration",
        "ast-refactor",
        "code-quality",
        "connectome",
        "connectome-rs",
        "connectome-py",
        "connectome-js",
        "duplicate-detector",
        "safety-check",
        "safety-check-py",
        "safety-check-js",
        // batch8
        "polyglot",
        "polyglot-metrics",
        "polyglot-convert",
        "circuit-breaker",
        "retry-policy",
        "graceful-shutdown",
        "immune-status",
        "immune-antibody",
        "immune-log",
        "plugin-list",
        "plugin-install",
        "plugin-remove",
        "dreamer-loop",
        "auto-evolve",
        "adaptive-evolve",
        "self-evolve",
        "mitosis",
        "bio-mitosis",
        "metamorphic-trigger",
        "omnicoder",
        "code-review",
        "agent-factory",
        "commander",
        "swarm-queen",
        "replicator",
        // batch10
        "vision-analyze",
        "vision-compare",
        "vision-ocr",
        "geolocation-lookup",
        "geolocation-distance",
        "geolocation-memory-map",
        "navigation-route",
        "navigation-poi",
        "collective-decision",
        "collective-consciousness",
        "distributed-raft",
        "distributed-lock",
        "alan-self-code",
        "alan-learn",
        "templates-refactor",
        "templates-list",
        "pollinations-generate",
        "pollinations-memory-viz",
        "qr-generate",
        "qr-spine",
        "qr-scan",
        "cryo-snap",
        // batch11
        "dna-extract",
        "dna-mutate",
        "dna-mutate-point",
        "dna-mutate-insert",
        "dna-mutate-delete",
        "dna-mutate-optimize",
        "dna-crossover",
        "dna-select",
        "dna-evolve",
        "dna-teach",
        "dna-hebbian",
        "dna-stats",
        "brain",
        "brain-compare",
        // batch12
        "dual-generate",
        "dual-cache",
        "dual-learn",
        "dual-record",
        "dual-status",
        "dual-teach",
    ]
}

#[cfg(test)]
mod resilient_tests {
    use super::*;

    #[test]
    fn failover_uses_next_healthy_blade() {
        let output = execute_resilient("missing-blade|code-reader", "src/main.rs");
        assert!(output.contains("missing-blade -> code-reader"));
    }
}
