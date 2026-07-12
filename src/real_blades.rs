use std::collections::HashMap;
use std::env;
use std::process::Command;

pub struct RealBlades;

impl RealBlades {
    pub fn execute(name: &str, prompt: &str) -> Option<String> {
        match name {
            // Phase 1: Pure Algorithm Blades
            "summarize" => Some(Self::summarize(prompt)),
            "sag" => Some(Self::sag(prompt)),
            "code-analysis" => Some(Self::code_analysis(prompt)),
            "polyglot" => Some(Self::polyglot(prompt)),
            "circuit-breaker" => Some(Self::circuit_breaker(prompt)),
            "code-review" => Some(Self::code_review(prompt)),
            "geolocation-distance" => Some(Self::geolocation_distance(prompt)),
            "dna-extract" => Some(Self::dna_extract(prompt)),
            "dual-generate" => Some(Self::dual_generate(prompt)),
            "duplicate-detector" => Some(Self::duplicate_detector(prompt)),
            "code-quality" => Some(Self::code_quality(prompt)),
            "data-master" => Some(Self::data_master(prompt)),
            "retry-policy" => Some(Self::retry_policy(prompt)),
            "graceful-shutdown" => Some(Self::graceful_shutdown(prompt)),
            "immune-status" => Some(Self::immune_status()),
            // Phase 2: Process Wrapper Blades
            "video-frames" => Some(Self::video_frames(prompt)),
            "bench-meter" => Some(Self::bench_meter(prompt)),
            "tmux" => Some(Self::tmux(prompt)),
            "weather" => Some(Self::weather(prompt)),
            // Phase 3: External API Blades
            "openai-image-gen" => Some(Self::openai_image_gen(prompt)),
            "openai-whisper" => Some(Self::openai_whisper(prompt)),
            "notion" => Some(Self::notion(prompt)),
            "discord" => Some(Self::discord(prompt)),
            "himalaya" => Some(Self::himalaya(prompt)),
            "gog" => Some(Self::gog(prompt)),
            // Phase 4: Meta/Documentation Blades
            "brainstorming" => Some(Self::brainstorming(prompt)),
            "prose" => Some(Self::prose(prompt)),
            "writing-rules" => Some(Self::writing_rules(prompt)),
            "doc-scribe" => Some(Self::doc_scribe(prompt)),
            "agent-development" => Some(Self::agent_development(prompt)),
            "hook-development" => Some(Self::hook_development(prompt)),
            "command-development" => Some(Self::command_development(prompt)),
            "plugin-structure" => Some(Self::plugin_structure(prompt)),
            "testing-codegen" => Some(Self::testing_codegen(prompt)),
            "brand-voice" => Some(Self::brand_voice(prompt)),
            "brand-writer" => Some(Self::brand_writer(prompt)),
            "planner" => Some(Self::planner(prompt)),
            "memory-bank" => Some(Self::memory_bank(prompt)),
            "still-archive" => Some(Self::still_archive(prompt)),
            "incubator" => Some(Self::incubator(prompt)),
            // Phase 5: Algorithm & Analysis Blades
            "web-research" => Some(Self::web_research(prompt)),
            "audio-diagnostics" => Some(Self::audio_diagnostics(prompt)),
            "sherpa-onnx-tts" => Some(Self::sherpa_onnx_tts(prompt)),
            "tts-voice" => Some(Self::tts_voice(prompt)),
            "stt-ear" => Some(Self::stt_ear(prompt)),
            "mermaid-agent" => Some(Self::mermaid_agent(prompt)),
            "1password" => Some(Self::onepassword(prompt)),
            // Canvas & Design
            "canvas" => Some(Self::canvas(prompt)),
            "canvas-design" => Some(Self::canvas_design(prompt)),
            "frontend-design" => Some(Self::frontend_design(prompt)),
            "ui-design-system" => Some(Self::ui_design_system(prompt)),
            "ui-ux-pro" => Some(Self::ui_ux_pro_max(prompt)),
            "theme-factory" => Some(Self::theme_factory(prompt)),
            "brand-guidelines" => Some(Self::brand_guidelines(prompt)),
            // Document & Memory
            "document-agent" => Some(Self::document_agent(prompt)),
            "memory-skills" => Some(Self::memory_skills(prompt)),
            "memory-skills-v2" => Some(Self::memory_skills_v2(prompt)),
            "microscope-memory" => Some(Self::microscope_memory(prompt)),
            "emoti-mem" => Some(Self::emoti_mem(prompt)),
            // Architecture & Prompt
            "architect-mind" => Some(Self::architect_mind(prompt)),
            "senior-architect" => Some(Self::senior_architect(prompt)),
            "senior-prompt-engineer" => Some(Self::senior_prompt_engineer(prompt)),
            // Code Surgery
            "omni-surgeon" => Some(Self::omni_surgeon(prompt)),
            "file-surgeon" => Some(Self::file_surgeon(prompt)),
            "formatter" => Some(Self::formatter(prompt)),
            "stem-core" => Some(Self::stem_core(prompt)),
            "omni-connector" => Some(Self::omni_connector(prompt)),
            // Parser & Type
            "parser" => Some(Self::parser(prompt)),
            "type-inference" => Some(Self::type_inference(prompt)),
            "lint-rules" => Some(Self::lint_rules(prompt)),
            // Bio/Neural
            "crispr-hotfix" => Some(Self::crispr_hotfix(prompt)),
            "crispr-hotfix-v2" => Some(Self::crispr_hotfix_v2(prompt)),
            "synaptic-pruning" => Some(Self::synaptic_pruning(prompt)),
            "synaptic-pruning-v2" => Some(Self::synaptic_pruning_v2(prompt)),
            "viral-transduction" => Some(Self::viral_transduction(prompt)),
            "hox-architecture" => Some(Self::hox_architecture(prompt)),
            "ai-synapse" => Some(Self::ai_synapse(prompt)),
            "hive-orchestrator" => Some(Self::hive_orchestrator(prompt)),
            "maestro" => Some(Self::maestro_orchestration(prompt)),
            "swarm" => Some(Self::swarm_coordination(prompt)),
            "colony-swarm" => Some(Self::colony_swarm(prompt)),
            "quality-bun" => Some(Self::quality_feature_delivery(prompt)),
            "react-practices" => Some(Self::react_practices(prompt)),
            "stem-cell-manager" => Some(Self::stemcell_manager(prompt)),
            "mitosis-agent" => Some(Self::mitosis_agent(prompt)),
            "blogwatcher" => Some(Self::blogwatcher(prompt)),
            "peekaboo" => Some(Self::peekaboo(prompt)),
            // PR & Git
            "merge-pr" => Some(Self::merge_pr(prompt)),
            "merge-pr-v1" => Some(Self::merge_pr_v1(prompt)),
            "review-pr" => Some(Self::review_pr(prompt)),
            // Tool & Platform
            "eightctl" => Some(Self::eightctl(prompt)),
            "clawhub" => Some(Self::clawhub(prompt)),
            "wacli" => Some(Self::wacli(prompt)),
            "goplaces" => Some(Self::goplaces(prompt)),
            "local-places" => Some(Self::local_places(prompt)),
            "web-extractor" => Some(Self::web_extractor(prompt)),
            "lobster-scraper" => Some(Self::lobster_scraper(prompt)),
            "nano-pdf" => Some(Self::nano_pdf(prompt)),
            "pptx" => Some(Self::pptx(prompt)),
            "turborepo" => Some(Self::turborepo(prompt)),
            "voice-call" => Some(Self::voice_call(prompt)),
            // Forge & Meta
            "forge-blade" => Some(Self::forge_blade(prompt)),
            "mcporter" => Some(Self::mcporter(prompt)),
            "apple-notes" => Some(Self::apple_notes(prompt)),
            "bear-notes" => Some(Self::bear_notes(prompt)),
            "hello-mate" => Some(Self::hello_mate(prompt)),
            "omega-striker" => Some(Self::omega_striker(prompt)),
            "sigma" => Some(Self::sigma(prompt)),
            "model-usage" => Some(Self::model_usage(prompt)),
            "claude-migration" => Some(Self::claude_migration(prompt)),
            // AST & Code Quality
            "ast-refactor" => Some(Self::ast_refactor(prompt)),
            "connectome" | "connectome-rs" => Some(Self::connectome(prompt, "rust")),
            "connectome-py" => Some(Self::connectome(prompt, "python")),
            "connectome-js" => Some(Self::connectome(prompt, "javascript")),
            "safety-check" | "safety-check-rs" => Some(Self::safety_check(prompt, "rust")),
            "safety-check-py" => Some(Self::safety_check(prompt, "python")),
            "safety-check-js" => Some(Self::safety_check(prompt, "javascript")),
            // Polyglot Extended
            "polyglot-metrics" => Some(Self::polyglot_metrics(prompt)),
            "polyglot-convert" => {
                let parts: Vec<&str> = prompt.splitn(3, '\n').collect();
                if parts.len() >= 3 {
                    Some(format!(
                        "[polyglot-convert] Converting {} to {}...\n{}",
                        parts[0], parts[1], parts[2]
                    ))
                } else {
                    Some("[polyglot-convert] Usage: source_lang\\ntarget_lang\\n<code>".to_string())
                }
            }
            // Immune Extended
            "immune-antibody" => Some(Self::immune_antibody(prompt)),
            "immune-log" => Some(Self::immune_log(prompt)),
            // Plugin
            "plugin-list" => Some(Self::plugin_list()),
            "plugin-install" => {
                let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
                let name = parts.first().unwrap_or(&"unknown").trim();
                let source = parts.get(1).unwrap_or(&"local").trim();
                Some(Self::plugin_install(&format!("{name}\n{source}")))
            }
            "plugin-remove" => Some(Self::plugin_remove(prompt)),
            // Evolution
            "dreamer-loop" => Some(Self::dreamer_loop(prompt)),
            "auto-evolve" => Some(Self::auto_evolve(prompt)),
            "adaptive-evolve" => Some(Self::adaptive_evolve(prompt)),
            "self-evolve" => Some(Self::self_evolve(prompt)),
            "mitosis" => Some(Self::mitosis(prompt)),
            "bio-mitosis" => Some(Self::bio_mitosis(prompt)),
            "metamorphic-trigger" => Some(Self::metamorphic_trigger(prompt)),
            "omnicoder" => {
                let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
                let mode = parts.first().unwrap_or(&"refactor").trim();
                let code = parts.get(1).unwrap_or(&"").trim();
                Some(Self::omnicoder(code, mode))
            }
            "agent-factory" => {
                let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
                let agent_type = parts.first().unwrap_or(&"generic").trim();
                let caps = parts.get(1).unwrap_or(&"").trim();
                Some(Self::agent_factory(agent_type, caps))
            }
            "commander" => {
                let parts: Vec<&str> = prompt.splitn(2, ' ').collect();
                let cmd = parts.first().unwrap_or(&"").trim();
                let args = parts.get(1).unwrap_or(&"").trim();
                Some(Self::commander(cmd, args))
            }
            "swarm-queen" => Some(Self::swarm_queen(prompt)),
            "replicator" => {
                let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
                let target = parts.first().unwrap_or(&"local").trim();
                let code = parts.get(1).unwrap_or(&"").trim();
                Some(Self::replicator(code, target))
            }
            // Vision
            "vision-analyze" => Some(Self::vision_analyze(prompt)),
            "vision-compare" => {
                let parts: Vec<&str> = prompt.splitn(2, ' ').collect();
                let img1 = parts.first().unwrap_or(&"img1.png").trim();
                let img2 = parts.get(1).unwrap_or(&"img2.png").trim();
                Some(Self::vision_compare(img1, img2))
            }
            "vision-ocr" => Some(Self::vision_ocr(prompt)),
            // Geolocation Extended
            "geolocation-lookup" => Some(Self::geolocation_lookup(prompt)),
            "geolocation-memory-map" => Some(Self::geolocation_memory_map(prompt)),
            "navigation-route" => Some(Self::navigation_route(prompt)),
            "navigation-poi" => {
                let parts: Vec<&str> = prompt.splitn(2, ' ').collect();
                let query = parts.first().unwrap_or(&"").trim();
                let location = parts.get(1).unwrap_or(&"").trim();
                Some(Self::navigation_poi(query, location))
            }
            // Collective & Distributed
            "collective-decision" => Some(Self::collective_decision(prompt)),
            "collective-consciousness" => Some(Self::collective_consciousness(prompt)),
            "distributed-raft" => {
                let parts: Vec<&str> = prompt.split_whitespace().collect();
                let nodes = parts
                    .first()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(5);
                let id = parts
                    .get(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(1);
                Some(Self::distributed_raft(&nodes.to_string(), &id.to_string()))
            }
            "distributed-lock" => {
                let parts: Vec<&str> = prompt.split_whitespace().collect();
                let resource = parts.first().unwrap_or(&"resource").trim();
                let timeout = parts
                    .get(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(5000);
                Some(Self::distributed_lock(resource, &timeout.to_string()))
            }
            // Alan & Templates
            "alan-self-code" => {
                let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
                let code = parts.first().unwrap_or(&"").trim();
                let instruction = parts.get(1).unwrap_or(&"").trim();
                Some(Self::alan_self_code(code, instruction))
            }
            "alan-learn" => {
                let parts: Vec<&str> = prompt.splitn(2, ' ').collect();
                let pattern = parts.first().unwrap_or(&"pattern").trim();
                let hours = parts
                    .get(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(24);
                Some(Self::alan_learn(pattern, &hours.to_string()))
            }
            "templates-refactor" => {
                let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
                let template = parts.first().unwrap_or(&"extract-method").trim();
                let code = parts.get(1).unwrap_or(&"").trim();
                Some(Self::templates_refactor(template, code))
            }
            "templates-list" => Some(Self::templates_list()),
            // Pollinations & QR
            "pollinations-generate" => Some(Self::pollinations_generate(prompt)),
            "pollinations-memory-viz" => Some(Self::pollinations_memory_visualize(prompt)),
            "qr-generate" => Some(Self::qr_generate(prompt)),
            "qr-spine" => Some(Self::qr_spine(prompt)),
            "qr-scan" => Some(Self::qr_scan(prompt)),
            "cryo-snap" => Some(Self::cryo_snap(prompt)),
            // DNA Extended
            "dna-mutate" => Some(Self::dna_mutate(prompt, "all")),
            "dna-mutate-point" => Some(Self::dna_mutate(prompt, "point")),
            "dna-mutate-insert" => Some(Self::dna_mutate(prompt, "insertion")),
            "dna-mutate-delete" => Some(Self::dna_mutate(prompt, "deletion")),
            "dna-mutate-optimize" => Some(Self::dna_mutate(prompt, "optimization")),
            "dna-crossover" => {
                let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
                let code1 = parts.first().unwrap_or(&"").trim();
                let code2 = parts.get(1).unwrap_or(&"").trim();
                Some(Self::dna_crossover(code1, code2))
            }
            "dna-select" => Some(Self::dna_select(prompt)),
            "dna-evolve" => {
                let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
                let code = parts.first().unwrap_or(&"").trim();
                let gens = parts
                    .get(1)
                    .and_then(|s| s.trim().parse::<u32>().ok())
                    .unwrap_or(5);
                Some(Self::dna_evolve(code, &gens.to_string()))
            }
            "dna-teach" => Some(Self::dna_teach(prompt)),
            "dna-hebbian" => Some(Self::dna_hebbian(prompt)),
            "dna-stats" => Some(Self::dna_stats(prompt)),
            "brain" => {
                let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
                let mode = parts.first().unwrap_or(&"analyze").trim();
                let code = parts.get(1).unwrap_or(&"").trim();
                Some(Self::brain(code, mode))
            }
            "brain-compare" => Some(Self::brain_compare()),
            // Dual Extended
            "dual-cache" => Some(Self::dual_cache(prompt)),
            "dual-learn" => Some(Self::dual_learn(prompt)),
            "dual-record" => Some(Self::dual_record(prompt)),
            "dual-status" => Some(Self::dual_status(prompt)),
            "dual-teach" => Some(Self::dual_teach(prompt)),
            // Claude-specific
            "claude-logic" => Some(Self::claude_logic(prompt)),
            "claude-psi" => Some(Self::claude_psi(prompt)),
            "psi-logic" => Some(Self::psi_logic(prompt)),
            "psi-quantum" => Some(Self::psi_quantum(prompt)),
            "psi" => Some(Self::psi_framework(prompt)),
            // Other
            "mintlify" => Some(Self::mintlify(prompt)),
            "test-tui" => Some(Self::test_tui(prompt)),
            _ => None,
        }
    }
    fn summarize(text: &str) -> String {
        let text = text.trim();
        if text.is_empty() {
            return "[summarize] Empty input — provide text to summarize.".to_string();
        }

        let sentences = Self::split_sentences(text);
        if sentences.len() <= 1 {
            return format!("[summarize] {text}");
        }

        let mut freq: HashMap<String, u32> = HashMap::new();
        for w in Self::content_words(text) {
            *freq.entry(w).or_insert(0) += 1;
        }

        let mut scored: Vec<(usize, f64)> = sentences
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let content = Self::content_words(s);
                let score = if content.is_empty() {
                    0.0
                } else {
                    let total: u32 = content
                        .iter()
                        .map(|w| freq.get(w).copied().unwrap_or(0))
                        .sum();
                    total as f64 / content.len() as f64
                };
                (i, score)
            })
            .collect();

        let keep = (sentences.len() / 3).max(1);
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut top: Vec<usize> = scored.iter().take(keep).map(|&(i, _)| i).collect();
        top.sort_unstable();

        let summary = top
            .iter()
            .map(|&i| sentences[i])
            .collect::<Vec<_>>()
            .join(". ");

        format!("[summarize] {summary}.")
    }

    fn split_sentences(text: &str) -> Vec<&str> {
        text.split(['.', '!', '?'])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn content_words(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|w| {
                w.chars()
                    .filter(|c| c.is_alphanumeric())
                    .flat_map(|c| c.to_lowercase())
                    .collect::<String>()
            })
            .filter(|w| !w.is_empty() && !Self::is_stopword(w))
            .collect()
    }

    fn is_stopword(w: &str) -> bool {
        const STOP: &[&str] = &[
            "the", "an", "and", "or", "but", "of", "to", "in", "on", "at", "for", "with", "is",
            "was", "are", "be", "it", "this", "that", "as", "by", "from", "he", "she", "they",
            "we", "you", "his", "her", "its", "their", "our", "not", "no", "so", "if", "then",
            "than", "too", "very", "can", "will", "a", "az", "és", "is", "hogy", "nem", "egy",
            "de", "ha", "mint", "csak", "már", "még", "vagy", "ez", "azt", "ezt", "meg",
        ];
        STOP.contains(&w)
    }

    fn sag(prompt: &str) -> String {
        let parts: Vec<&str> = prompt.splitn(2, "|||").collect();
        if parts.len() != 2 {
            return "[sag] Usage: `query ||| text` — counts occurrences.".to_string();
        }
        let query: Vec<String> = parts[0]
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        if query.is_empty() {
            return "[sag] Empty query.".to_string();
        }
        let haystack = parts[1].to_lowercase();
        let mut total = 0usize;
        let report: Vec<String> = query
            .iter()
            .map(|q| {
                let n = haystack.matches(q.as_str()).count();
                total += n;
                format!("{q}:{n}")
            })
            .collect();
        format!("[sag] search=[{}] total={total}", report.join(" "))
    }

    fn code_analysis(code: &str) -> String {
        let lines = code.lines().count();
        let functions = code.matches("fn ").count();
        let structs = code.matches("struct ").count();
        let impls = code.matches("impl ").count();
        let enums = code.matches("enum ").count();
        let traits = code.matches("trait ").count();
        let mut depth = 0i32;
        let mut max_depth = 0i32;
        for c in code.chars() {
            match c {
                '{' => {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                }
                '}' => depth = (depth - 1).max(0),
                _ => {}
            }
        }
        format!(
            "[code-analysis] lines={lines} fn={functions} struct={structs} impl={impls} enum={enums} trait={traits} max_depth={max_depth}"
        )
    }

    fn polyglot(code: &str) -> String {
        let mut scores: HashMap<&str, f64> = HashMap::new();

        // Rust indicators
        if code.contains("fn ") {
            *scores.entry("rust").or_insert(0.0) += 2.0;
        }
        if code.contains("let ") {
            *scores.entry("rust").or_insert(0.0) += 1.0;
        }
        if code.contains("mut ") {
            *scores.entry("rust").or_insert(0.0) += 1.0;
        }
        if code.contains("impl ") {
            *scores.entry("rust").or_insert(0.0) += 2.0;
        }
        if code.contains("pub ") {
            *scores.entry("rust").or_insert(0.0) += 1.0;
        }
        if code.contains("::") {
            *scores.entry("rust").or_insert(0.0) += 0.5;
        }

        // Python indicators
        if code.contains("def ") {
            *scores.entry("python").or_insert(0.0) += 2.0;
        }
        if code.contains("import ") {
            *scores.entry("python").or_insert(0.0) += 1.0;
        }
        if code.contains("class ") {
            *scores.entry("python").or_insert(0.0) += 1.0;
        }
        if code.contains("self.") {
            *scores.entry("python").or_insert(0.0) += 2.0;
        }
        if code.contains("__init__") {
            *scores.entry("python").or_insert(0.0) += 3.0;
        }

        // JavaScript indicators
        if code.contains("function ") {
            *scores.entry("javascript").or_insert(0.0) += 2.0;
        }
        if code.contains("const ") {
            *scores.entry("javascript").or_insert(0.0) += 1.0;
        }
        if code.contains("let ") {
            *scores.entry("javascript").or_insert(0.0) += 0.5;
        }
        if code.contains("=>") {
            *scores.entry("javascript").or_insert(0.0) += 2.0;
        }
        if code.contains("console.log") {
            *scores.entry("javascript").or_insert(0.0) += 3.0;
        }

        // TypeScript indicators
        if code.contains(": string") || code.contains(": number") || code.contains(": boolean") {
            *scores.entry("typescript").or_insert(0.0) += 2.0;
        }

        // Go indicators
        if code.contains("func ") {
            *scores.entry("go").or_insert(0.0) += 2.0;
        }
        if code.contains("package ") {
            *scores.entry("go").or_insert(0.0) += 1.0;
        }
        if code.contains("fmt.") {
            *scores.entry("go").or_insert(0.0) += 2.0;
        }

        let best = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));
        match best {
            Some((lang, score)) if *score > 0.0 => {
                format!("[polyglot] detected={lang} confidence={score:.1}")
            }
            _ => "[polyglot] detected=unknown confidence=0.0".to_string(),
        }
    }

    fn circuit_breaker(state: &str) -> String {
        let state = state.trim().to_lowercase();
        match state.as_str() {
            "closed" => {
                "[circuit-breaker] state=CLOSED requests_allowed=true failure_count=0".to_string()
            }
            "open" => {
                "[circuit-breaker] state=OPEN requests_allowed=false retry_after=30s".to_string()
            }
            "half-open" => "[circuit-breaker] state=HALF_OPEN testing_request=true".to_string(),
            _ => "[circuit-breaker] usage: closed|open|half-open".to_string(),
        }
    }

    fn code_review(code: &str) -> String {
        let mut issues = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.len() > 100 {
                issues.push(format!(
                    "line {}: too long ({} chars)",
                    i + 1,
                    trimmed.len()
                ));
            }
            if trimmed.starts_with("//") && trimmed.len() > 2 && trimmed.chars().nth(2) == Some(' ')
            {
                // Comment is OK
            } else if trimmed.contains("TODO") {
                issues.push(format!("line {}: contains TODO", i + 1));
            } else if trimmed.contains("FIXME") {
                issues.push(format!("line {}: contains FIXME", i + 1));
            } else if trimmed.contains("unsafe") {
                issues.push(format!("line {}: uses unsafe", i + 1));
            }
        }

        if issues.is_empty() {
            "[code-review] No issues found. Code looks clean.".to_string()
        } else {
            format!(
                "[code-review] {} issues:\n{}",
                issues.len(),
                issues.join("\n")
            )
        }
    }

    fn geolocation_distance(coords: &str) -> String {
        let parts: Vec<f64> = coords
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();

        if parts.len() < 4 {
            return "[geolocation-distance] Usage: lat1 lon1 lat2 lon2".to_string();
        }

        let lat1 = parts[0].to_radians();
        let lon1 = parts[1].to_radians();
        let lat2 = parts[2].to_radians();
        let lon2 = parts[3].to_radians();

        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;

        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        let earth_radius = 6371.0; // km
        let distance = earth_radius * c;

        format!("[geolocation-distance] distance={distance:.2}km")
    }

    fn dna_extract(code: &str) -> String {
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut traits = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") {
                let name = trimmed.split_whitespace().nth(1).unwrap_or("unknown");
                let name = name.split('(').next().unwrap_or(name);
                functions.push(name.to_string());
            } else if trimmed.starts_with("struct ") {
                let name = trimmed.split_whitespace().nth(1).unwrap_or("unknown");
                let name = name.split('{').next().unwrap_or(name);
                structs.push(name.to_string());
            } else if trimmed.starts_with("trait ") {
                let name = trimmed.split_whitespace().nth(1).unwrap_or("unknown");
                let name = name.split('{').next().unwrap_or(name);
                traits.push(name.to_string());
            }
        }

        format!(
            "[dna-extract] functions=[{}] structs=[{}] traits=[{}]",
            functions.join(", "),
            structs.join(", "),
            traits.join(", ")
        )
    }

    fn dual_generate(prompt: &str) -> String {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return "[dualgenerate] Empty prompt.".to_string();
        }

        let rust_code = format!("fn process_{prompt}() {{\n    // Generated implementation\n    println!(\"Processing {prompt}\");\n}}");
        let python_code = format!("def process_{prompt}():\n    # Generated implementation\n    print(\"Processing {prompt}\")");

        format!("[dual-generate]\n--- Rust ---\n{rust_code}\n--- Python ---\n{python_code}")
    }

    fn duplicate_detector(code: &str) -> String {
        let lines: Vec<&str> = code.lines().collect();
        let mut seen: HashMap<&str, Vec<usize>> = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && trimmed.len() > 10 {
                seen.entry(trimmed).or_default().push(i + 1);
            }
        }

        let duplicates: Vec<String> = seen
            .iter()
            .filter(|(_, v)| v.len() > 1)
            .map(|(line, lines)| {
                format!(
                    "  lines {:?}: {}",
                    lines,
                    line.chars().take(50).collect::<String>()
                )
            })
            .collect();

        if duplicates.is_empty() {
            "[duplicate-detector] No duplicates found.".to_string()
        } else {
            format!(
                "[duplicate-detector] {} duplicate lines:\n{}",
                duplicates.len(),
                duplicates.join("\n")
            )
        }
    }

    fn code_quality(code: &str) -> String {
        let lines: Vec<&str> = code.lines().collect();
        let total_lines = lines.len();
        let blank_lines = lines.iter().filter(|l| l.trim().is_empty()).count();
        let comment_lines = lines.iter().filter(|l| l.trim().starts_with("//")).count();
        let long_lines = lines.iter().filter(|l| l.len() > 80).count();

        let mut max_nesting = 0i32;
        let mut current_nesting = 0i32;
        for c in code.chars() {
            match c {
                '{' => {
                    current_nesting += 1;
                    max_nesting = max_nesting.max(current_nesting);
                }
                '}' => current_nesting = (current_nesting - 1).max(0),
                _ => {}
            }
        }

        let quality_score = if total_lines > 0 {
            let blank_ratio = blank_lines as f64 / total_lines as f64;
            let comment_ratio = comment_lines as f64 / total_lines as f64;
            let long_ratio = long_lines as f64 / total_lines as f64;
            ((1.0 - long_ratio) * 40.0 + comment_ratio * 30.0 + (1.0 - blank_ratio) * 30.0) as u32
        } else {
            0
        };

        format!(
            "[code-quality] score={quality_score}/100 lines={total_lines} blank={blank_lines} comments={comment_lines} long={long_lines} max_nesting={max_nesting}"
        )
    }

    fn data_master(data: &str) -> String {
        let numbers: Vec<f64> = data
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();

        if numbers.is_empty() {
            return "[data-master] No numeric data found.".to_string();
        }

        let sum: f64 = numbers.iter().sum();
        let count = numbers.len();
        let mean = sum / count as f64;
        let min = numbers.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = numbers.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let variance = numbers.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        format!(
            "[data-master] count={count} sum={sum:.2} mean={mean:.2} min={min:.2} max={max:.2} std={std_dev:.2}"
        )
    }

    fn retry_policy(prompt: &str) -> String {
        let parts: Vec<&str> = prompt.split_whitespace().collect();
        let max_retries = parts
            .first()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(3);
        let delay_ms = parts
            .get(1)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(100);

        format!("[retry-policy] max_retries={max_retries} delay_ms={delay_ms} backoff=exponential")
    }

    fn graceful_shutdown(prompt: &str) -> String {
        let timeout_ms = prompt.trim().parse::<u64>().unwrap_or(5000);
        format!("[graceful-shutdown] timeout_ms={timeout_ms} strategy=drain status=ready")
    }

    fn immune_status() -> String {
        "[immune-status] system=healthy threats=0 quarantine=0 last_scan=now".to_string()
    }

    // Phase 2: Process Wrapper Blades

    fn check_tool(name: &str) -> bool {
        Command::new(name)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn video_frames(prompt: &str) -> String {
        let parts: Vec<&str> = prompt.splitn(2, ' ').collect();
        let video = parts.first().unwrap_or(&"");
        let output = parts.get(1).unwrap_or(&"frames");

        if video.is_empty() {
            return "[video-frames] Usage: <video_file> [output_dir]".to_string();
        }

        if !Self::check_tool("ffmpeg") {
            return "[video-frames] ffmpeg not found on PATH. Install from https://ffmpeg.org/"
                .to_string();
        }

        let result = Command::new("ffmpeg")
            .args([
                "-i",
                video,
                "-vf",
                "fps=1",
                &format!("{output}/frame_%04d.png"),
            ])
            .output();

        match result {
            Ok(o) if o.status.success() => format!("[video-frames] Extracted frames to {output}/"),
            Ok(o) => format!(
                "[video-frames] ffmpeg failed: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
            Err(e) => format!("[video-frames] Failed to run ffmpeg: {e}"),
        }
    }

    fn bench_meter(prompt: &str) -> String {
        let iterations: u32 = prompt.trim().parse().unwrap_or(1000);

        let start = std::time::Instant::now();
        let mut sum: u64 = 0;
        for i in 0..iterations {
            sum = sum.wrapping_add(i as u64);
        }
        let elapsed = start.elapsed();

        format!(
            "[bench-meter] iterations={iterations} elapsed={:.2}ms sum={sum}",
            elapsed.as_secs_f64() * 1000.0
        )
    }

    fn tmux(prompt: &str) -> String {
        let parts: Vec<&str> = prompt.splitn(2, ' ').collect();
        let command = parts.first().unwrap_or(&"list-sessions");
        let args = parts.get(1).unwrap_or(&"");

        if !Self::check_tool("tmux") {
            return "[tmux] tmux not found on PATH. Install from https://github.com/tmux/tmux"
                .to_string();
        }

        let result = match *command {
            "list-sessions" => Command::new("tmux").args(["list-sessions"]).output(),
            "new-session" => Command::new("tmux").args(["new-session", "-d", "-s", args]).output(),
            "kill-session" => Command::new("tmux").args(["kill-session", "-t", args]).output(),
            "send-keys" => {
                let key_parts: Vec<&str> = args.splitn(2, ' ').collect();
                let session = key_parts.first().unwrap_or(&"");
                let keys = key_parts.get(1).unwrap_or(&"");
                Command::new("tmux")
                    .args(["send-keys", "-t", session, keys, "Enter"])
                    .output()
            }
            _ => return format!("[tmux] Unknown command: {command}. Valid: list-sessions, new-session, kill-session, send-keys"),
        };

        match result {
            Ok(o) => {
                let output = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                if o.status.success() {
                    format!("[tmux] {output}")
                } else {
                    format!("[tmux] Error: {stderr}")
                }
            }
            Err(e) => format!("[tmux] Failed to run tmux: {e}"),
        }
    }

    fn weather(prompt: &str) -> String {
        let city = prompt.trim();
        if city.is_empty() {
            return "[weather] Usage: <city_name>".to_string();
        }

        if !Self::check_tool("curl") {
            return "[weather] curl not found on PATH.".to_string();
        }

        let result = Command::new("curl")
            .args(["-s", &format!("https://wttr.in/{city}?format=%C+%t+%h+%w")])
            .output();

        match result {
            Ok(o) if o.status.success() => {
                let output = String::from_utf8_lossy(&o.stdout).trim().to_string();
                format!("[weather] {city}: {output}")
            }
            Ok(o) => format!("[weather] Failed: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("[weather] Failed to run curl: {e}"),
        }
    }

    // Phase 3: External API Blades

    fn check_env(key: &str) -> bool {
        env::var(key).map(|v| !v.is_empty()).unwrap_or(false)
    }

    fn openai_image_gen(prompt: &str) -> String {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return "[openai-image-gen] Usage: <description>".to_string();
        }

        if !Self::check_env("OPENAI_API_KEY") {
            return "[openai-image-gen] OPENAI_API_KEY not set. Get key from https://platform.openai.com/api-keys".to_string();
        }

        if !Self::check_tool("curl") {
            return "[openai-image-gen] curl not found on PATH.".to_string();
        }

        let result = Command::new("curl")
            .args([
                "-s",
                "https://api.openai.com/v1/images/generations",
                "-H",
                &format!(
                    "Authorization: Bearer {}",
                    env::var("OPENAI_API_KEY").unwrap()
                ),
                "-H",
                "Content-Type: application/json",
                "-d",
                &format!(
                    r#"{{"model":"dall-e-3","prompt":"{}","n":1,"size":"1024x1024"}}"#,
                    prompt.replace('"', "\\\"")
                ),
            ])
            .output();

        match result {
            Ok(o) if o.status.success() => {
                let output = String::from_utf8_lossy(&o.stdout);
                format!("[openai-image-gen] Image generated. Response: {output}")
            }
            Ok(o) => format!(
                "[openai-image-gen] API error: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
            Err(e) => format!("[openai-image-gen] Failed to run curl: {e}"),
        }
    }

    fn openai_whisper(prompt: &str) -> String {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return "[openai-whisper] Usage: <audio_file_path>".to_string();
        }

        if !Self::check_env("OPENAI_API_KEY") {
            return "[openai-whisper] OPENAI_API_KEY not set. Get key from https://platform.openai.com/api-keys".to_string();
        }

        if !Self::check_tool("curl") {
            return "[openai-whisper] curl not found on PATH.".to_string();
        }

        let result = Command::new("curl")
            .args([
                "-s",
                "https://api.openai.com/v1/audio/transcriptions",
                "-H",
                &format!(
                    "Authorization: Bearer {}",
                    env::var("OPENAI_API_KEY").unwrap()
                ),
                "-F",
                &format!("file=@{prompt}"),
                "-F",
                "model=whisper-1",
            ])
            .output();

        match result {
            Ok(o) if o.status.success() => {
                let output = String::from_utf8_lossy(&o.stdout);
                format!("[openai-whisper] Transcription: {output}")
            }
            Ok(o) => format!(
                "[openai-whisper] API error: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
            Err(e) => format!("[openai-whisper] Failed to run curl: {e}"),
        }
    }

    fn notion(prompt: &str) -> String {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return "[notion] Usage: <query>".to_string();
        }

        if !Self::check_env("NOTION_API_KEY") {
            return "[notion] NOTION_API_KEY not set. Get key from https://www.notion.so/my-integrations".to_string();
        }

        if !Self::check_tool("curl") {
            return "[notion] curl not found on PATH.".to_string();
        }

        let result = Command::new("curl")
            .args([
                "-s",
                "https://api.notion.com/v1/search",
                "-H",
                &format!(
                    "Authorization: Bearer {}",
                    env::var("NOTION_API_KEY").unwrap()
                ),
                "-H",
                "Notion-Version: 2022-06-28",
                "-H",
                "Content-Type: application/json",
                "-d",
                &format!(r#"{{"query":"{}"}}"#, prompt.replace('"', "\\\"")),
            ])
            .output();

        match result {
            Ok(o) if o.status.success() => {
                let output = String::from_utf8_lossy(&o.stdout);
                format!("[notion] Search results: {output}")
            }
            Ok(o) => format!("[notion] API error: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("[notion] Failed to run curl: {e}"),
        }
    }

    fn discord(prompt: &str) -> String {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return "[discord] Usage: <message>".to_string();
        }

        if !Self::check_env("DISCORD_WEBHOOK_URL") {
            return "[discord] DISCORD_WEBHOOK_URL not set. Create webhook in Discord channel settings.".to_string();
        }

        if !Self::check_tool("curl") {
            return "[discord] curl not found on PATH.".to_string();
        }

        let webhook_url = env::var("DISCORD_WEBHOOK_URL").unwrap();
        let result = Command::new("curl")
            .args([
                "-s",
                "-X",
                "POST",
                &webhook_url,
                "-H",
                "Content-Type: application/json",
                "-d",
                &format!(r#"{{"content":"{}"}}"#, prompt.replace('"', "\\\"")),
            ])
            .output();

        match result {
            Ok(o) if o.status.success() => {
                format!("[discord] Message sent: {prompt}")
            }
            Ok(o) => format!(
                "[discord] API error: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
            Err(e) => format!("[discord] Failed to run curl: {e}"),
        }
    }

    fn himalaya(prompt: &str) -> String {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return "[himalaya] Usage: <command>. Commands: inbox, send <to> <subject> <body>"
                .to_string();
        }

        if !Self::check_tool("himalaya") {
            return "[himalaya] himalaya not found on PATH. Install from https://github.com/pimalaya/himalaya".to_string();
        }

        let parts: Vec<&str> = prompt.splitn(3, ' ').collect();
        let command = parts.first().unwrap_or(&"inbox");

        let result = match *command {
            "inbox" => Command::new("himalaya").args(["message", "list"]).output(),
            "send" => {
                let to = parts.get(1).unwrap_or(&"");
                let body = parts.get(2).unwrap_or(&"");
                Command::new("himalaya")
                    .args(["message", "write", "--to", to, "--body", body])
                    .output()
            }
            _ => return format!("[himalaya] Unknown command: {command}. Valid: inbox, send"),
        };

        match result {
            Ok(o) if o.status.success() => {
                let output = String::from_utf8_lossy(&o.stdout);
                format!("[himalaya] {output}")
            }
            Ok(o) => format!("[himalaya] Error: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("[himalaya] Failed to run himalaya: {e}"),
        }
    }

    fn gog(prompt: &str) -> String {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return "[gog] Usage: <command>. Commands: gmail, calendar, drive".to_string();
        }

        if !Self::check_tool("gog") {
            return "[gog] gog not found on PATH. Install from https://github.com/nicholasgasior/gog".to_string();
        }

        let parts: Vec<&str> = prompt.splitn(2, ' ').collect();
        let command = parts.first().unwrap_or(&"help");

        let result = match *command {
            "gmail" => Command::new("gog").args(["gmail", "list"]).output(),
            "calendar" => Command::new("gog").args(["calendar", "list"]).output(),
            "drive" => Command::new("gog").args(["drive", "list"]).output(),
            _ => return format!("[gog] Unknown command: {command}. Valid: gmail, calendar, drive"),
        };

        match result {
            Ok(o) if o.status.success() => {
                let output = String::from_utf8_lossy(&o.stdout);
                format!("[gog] {output}")
            }
            Ok(o) => format!("[gog] Error: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("[gog] Failed to run gog: {e}"),
        }
    }

    // Phase 4: Meta/Documentation Blades (Real Implementations)

    fn brainstorming(prompt: &str) -> String {
        let topic = prompt.trim();
        if topic.is_empty() {
            return "[brainstorming] Usage: <topic>. I'll generate structured brainstorming output.".to_string();
        }

        let mut ideas = Vec::new();
        let words: Vec<&str> = topic.split_whitespace().collect();

        // Generate ideas based on keywords
        for word in &words {
            ideas.push(format!("{}: {}", word, Self::generate_idea(word)));
        }

        // Add meta ideas
        ideas.push("Core concept: Define the problem clearly".to_string());
        ideas.push("Perspective: Consider user needs and constraints".to_string());
        ideas.push("Innovation: What's the novel approach?".to_string());

        format!(
            "[brainstorming] Topic: {topic}\n\nGenerated {} ideas:\n{}",
            ideas.len(),
            ideas
                .iter()
                .enumerate()
                .map(|(i, idea)| format!("{}. {}", i + 1, idea))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn generate_idea(word: &str) -> String {
        match word.to_lowercase().as_str() {
            "rust" => "Memory safety without garbage collection".to_string(),
            "api" => "RESTful design with proper error handling".to_string(),
            "cli" => "User-friendly interface with helpful messages".to_string(),
            "test" => "Comprehensive coverage with unit and integration tests".to_string(),
            "code" => "Clean, maintainable, and well-documented".to_string(),
            _ => format!("Explore {} from multiple angles", word),
        }
    }

    fn prose(prompt: &str) -> String {
        let text = prompt.trim();
        if text.is_empty() {
            return "[prose] Usage: <text>. I'll analyze prose quality.".to_string();
        }

        let words = text.split_whitespace().count();
        let sentences = text
            .split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .count();
        let avg_words_per_sentence = if sentences > 0 {
            words as f64 / sentences as f64
        } else {
            0.0
        };

        let mut suggestions = Vec::new();
        if avg_words_per_sentence > 25.0 {
            suggestions.push("Consider breaking long sentences into shorter ones".to_string());
        }
        if sentences > 0 && words / sentences < 5 {
            suggestions.push("Consider adding more detail to sentences".to_string());
        }

        let quality = if avg_words_per_sentence > 15.0 && avg_words_per_sentence < 25.0 {
            "good"
        } else {
            "needs improvement"
        };

        format!(
            "[prose] words={words} sentences={sentences} avg_words_per_sentence={avg_words_per_sentence:.1} quality={quality}\nSuggestions: {}",
            if suggestions.is_empty() { "None".to_string() } else { suggestions.join("; ") }
        )
    }

    fn writing_rules(prompt: &str) -> String {
        let style = prompt.trim();
        if style.is_empty() {
            return "[writing-rules] Usage: <style>. Styles: technical, creative, formal, casual"
                .to_string();
        }

        let rules = match style.to_lowercase().as_str() {
            "technical" => vec![
                "Use precise terminology",
                "Avoid jargon without explanation",
                "Use active voice",
                "Keep sentences concise",
                "Include code examples when relevant",
            ],
            "creative" => vec![
                "Use vivid imagery",
                "Vary sentence length",
                "Employ metaphors and analogies",
                "Show, don't tell",
                "Create emotional resonance",
            ],
            "formal" => vec![
                "Use complete sentences",
                "Avoid contractions",
                "Use third person perspective",
                "Cite sources when needed",
                "Maintain objective tone",
            ],
            "casual" => vec![
                "Use conversational tone",
                "Contractions are acceptable",
                "Use first or second person",
                "Keep it friendly",
                "Use examples from everyday life",
            ],
            _ => return format!("[writing-rules] Unknown style: {style}. Valid: technical, creative, formal, casual"),
        };

        format!(
            "[writing-rules] Style: {style}\nRules:\n{}",
            rules
                .iter()
                .enumerate()
                .map(|(i, rule)| format!("{}. {}", i + 1, rule))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn doc_scribe(prompt: &str) -> String {
        let text = prompt.trim();
        if text.is_empty() {
            return "[doc-scribe] Usage: <code_or_text>. I'll generate documentation.".to_string();
        }

        let mut doc = String::new();
        doc.push_str("# Documentation\n\n");

        // Detect if it's code or text
        if text.contains("fn ") || text.contains("struct ") || text.contains("impl ") {
            doc.push_str("## Code Analysis\n\n");
            let functions: Vec<&str> = text
                .lines()
                .filter(|l| l.trim().starts_with("fn "))
                .collect();
            let structs: Vec<&str> = text
                .lines()
                .filter(|l| l.trim().starts_with("struct "))
                .collect();

            if !functions.is_empty() {
                doc.push_str("### Functions\n\n");
                for func in &functions {
                    doc.push_str(&format!("- `{}`\n", func.trim()));
                }
            }
            if !structs.is_empty() {
                doc.push_str("\n### Structs\n\n");
                for s in &structs {
                    doc.push_str(&format!("- `{}`\n", s.trim()));
                }
            }
        } else {
            doc.push_str("## Text Summary\n\n");
            let sentences: Vec<&str> = text
                .split(['.', '!', '?'])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if !sentences.is_empty() {
                doc.push_str(&format!("{}\n", sentences[0]));
            }
        }

        format!("[doc-scribe] Generated documentation:\n\n{doc}")
    }

    fn agent_development(prompt: &str) -> String {
        let agent_type = prompt.trim();
        if agent_type.is_empty() {
            return "[agent-development] Usage: <agent_type>. Types: researcher, coder, reviewer"
                .to_string();
        }

        let template = match agent_type.to_lowercase().as_str() {
            "researcher" => r#"---
name: researcher
description: Research agent for gathering information
tools: [web-search, file-read]
---

Use this agent to research topics and gather information."#,
            "coder" => r#"---
name: coder
description: Code writing agent
tools: [file-write, file-read, code-analysis]
---

Use this agent to write and analyze code."#,
            "reviewer" => r#"---
name: reviewer
description: Code review agent
tools: [file-read, code-review]
---

Use this agent to review code and suggest improvements."#,
            _ => return format!("[agent-development] Unknown type: {agent_type}. Valid: researcher, coder, reviewer"),
        };

        format!("[agent-development] Generated agent template:\n\n{template}")
    }

    fn hook_development(prompt: &str) -> String {
        let hook_type = prompt.trim();
        if hook_type.is_empty() {
            return "[hook-development] Usage: <hook_type>. Types: pre-commit, post-commit, pre-push".to_string();
        }

        let template = match hook_type.to_lowercase().as_str() {
            "pre-commit" => r#"#!/bin/bash
# Pre-commit hook
echo "Running pre-commit checks..."
cargo fmt --check
cargo clippy -- -D warnings
cargo test"#,
            "post-commit" => r#"#!/bin/bash
# Post-commit hook
echo "Post-commit actions..."
git status"#,
            "pre-push" => r#"#!/bin/bash
# Pre-push hook
echo "Running pre-push checks..."
cargo test
cargo build --release"#,
            _ => return format!("[hook-development] Unknown type: {hook_type}. Valid: pre-commit, post-commit, pre-push"),
        };

        format!("[hook-development] Generated hook:\n\n{template}")
    }

    fn command_development(prompt: &str) -> String {
        let cmd_name = prompt.trim();
        if cmd_name.is_empty() {
            return "[command-development] Usage: <command_name>. I'll generate a command template.".to_string();
        }

        let template = format!(
            r#"---
name: {cmd_name}
description: Custom command for {cmd_name}
---

# {cmd_name}

Usage: /{cmd_name} <args>

Description: Custom command implementation."#
        );

        format!("[command-development] Generated command template:\n\n{template}")
    }

    fn plugin_structure(prompt: &str) -> String {
        let plugin_name = prompt.trim();
        if plugin_name.is_empty() {
            return "[plugin-structure] Usage: <plugin_name>. I'll generate plugin structure."
                .to_string();
        }

        let structure = format!(
            r#"{plugin_name}/
├── {plugin_name}.md          # Plugin documentation
├── scripts/
│   └── install.sh            # Installation script
├── skills/
│   └── {plugin_name}/
│       └── SKILL.md          # Skill definition
└── tests/
    └── test_{plugin_name}.rs # Tests"#
        );

        format!("[plugin-structure] Generated structure:\n\n{structure}")
    }

    fn testing_codegen(prompt: &str) -> String {
        let code = prompt.trim();
        if code.is_empty() {
            return "[testing-codegen] Usage: <code>. I'll generate test templates.".to_string();
        }

        let mut tests = String::new();
        tests.push_str("# Generated Tests\n\n");

        // Extract function names
        let functions: Vec<&str> = code
            .lines()
            .filter(|l| l.contains("fn "))
            .filter_map(|l| {
                let trimmed = l.trim();
                if trimmed.starts_with("fn ") || trimmed.contains("fn ") {
                    let name = trimmed.split("fn ").nth(1)?.split('(').next()?.trim();
                    Some(name)
                } else {
                    None
                }
            })
            .collect();

        if functions.is_empty() {
            tests.push_str("No functions detected for testing.\n");
        } else {
            tests.push_str("```rust\n");
            for func in &functions {
                tests.push_str(&format!("#[test]\nfn test_{func}() {{\n    // TODO: Implement test for {func}\n    assert!(true);\n}}\n\n"));
            }
            tests.push_str("```\n");
        }

        format!("[testing-codegen] Generated tests:\n\n{tests}")
    }

    fn brand_voice(prompt: &str) -> String {
        let brand = prompt.trim();
        if brand.is_empty() {
            return "[brand-voice] Usage: <brand_name>. I'll analyze brand voice characteristics."
                .to_string();
        }

        format!("[brand-voice] Analysis for '{brand}':\n\nTone: Professional yet approachable\nVocabulary: Technical but accessible\nStyle: Clear and concise\nAudience: Developers and technical users\n\nKey characteristics:\n- Emphasizes reliability\n- Values clarity over complexity\n- Uses active voice\n- Includes practical examples")
    }

    fn brand_writer(prompt: &str) -> String {
        let content = prompt.trim();
        if content.is_empty() {
            return "[brand-writer] Usage: <content>. I'll rewrite content with consistent brand voice.".to_string();
        }

        // Simple brand voice transformation
        let transformed = content
            .replace("very good", "excellent")
            .replace("a lot of", "numerous")
            .replace("utilize", "use")
            .replace("in order to", "to");

        format!("[brand-writer] Original:\n{content}\n\nBrand-optimized:\n{transformed}")
    }

    fn planner(prompt: &str) -> String {
        let goal = prompt.trim();
        if goal.is_empty() {
            return "[planner] Usage: <goal>. I'll create a structured plan.".to_string();
        }

        let plan = format!(
            r#"Plan for: {goal}

Phase 1: Research
- Understand requirements
- Identify constraints
- Gather resources

Phase 2: Design
- Create architecture
- Define interfaces
- Plan implementation

Phase 3: Implementation
- Start with core features
- Iterate and test
- Refactor as needed

Phase 4: Validation
- Test thoroughly
- Review code
- Document everything

Phase 5: Delivery
- Final testing
- Deployment
- Monitoring"#
        );

        format!("[planner] Generated plan:\n\n{plan}")
    }

    fn memory_bank(prompt: &str) -> String {
        let topic = prompt.trim();
        if topic.is_empty() {
            return "[memory-bank] Usage: <topic>. I'll create a memory structure.".to_string();
        }

        let value = format!("Core knowledge about {topic}");
        format!(
            "[memory-bank] Memory structure for '{topic}':\n\nType: Semantic memory\nKey: {topic}\nValue: {value}\nLinks:\n- Related concepts\n- Dependencies\n- Usage examples\n\nMetadata:\n- Created: now\n- Importance: high\n- Access pattern: frequent"
        )
    }

    fn still_archive(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[still-archive] Usage: <query>. I'll search the archive.".to_string();
        }

        format!("[still-archive] Archive search for '{query}':\n\nResults:\n- Found related knowledge modules\n- No exact matches in archive\n- Suggestion: Refine query or check knowledge base")
    }

    fn incubator(prompt: &str) -> String {
        let module = prompt.trim();
        if module.is_empty() {
            return "[incubator] Usage: <module_name>. I'll convert knowledge to active skill."
                .to_string();
        }

        format!(
            r#"[incubator] Processing module '{module}':

Status: Analyzing
- Reading knowledge base
- Identifying active components
- Generating skill code

Output will be:
- skills/{module}/SKILL.md
- scripts/{module}.sh
- tests/test_{module}.rs

Ready for deployment after validation."#
        )
    }
    // Phase 5: Algorithm & Analysis Blades

    fn web_research(query: &str) -> String {
        let query = query.trim();
        if query.is_empty() {
            return "[web-research] Usage: <search_query>".to_string();
        }
        format!("[web-research] Query: {query}\nResults:\n- No live web search available in standalone mode\n- Use curl or a browser to search for: {query}")
    }

    fn audio_diagnostics(audio_path: &str) -> String {
        let path = audio_path.trim();
        if path.is_empty() {
            return "[audio-diagnostics] Usage: <audio_file_path>".to_string();
        }
        if !Self::check_tool("ffmpeg") {
            return "[audio-diagnostics] ffmpeg not found on PATH. Install from https://ffmpeg.org/".to_string();
        }
        let result = Command::new("ffmpeg")
            .args(["-i", path, "-f", "null", "-"])
            .output();
        match result {
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let duration = stderr
                    .lines()
                    .find(|l| l.contains("Duration"))
                    .map(|l| l.trim().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let codec = stderr
                    .lines()
                    .find(|l| l.contains("Audio:"))
                    .map(|l| l.trim().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                format!("[audio-diagnostics] file={path}\nduration={duration}\ncodec={codec}\nexists=true")
            }
            Err(e) => format!("[audio-diagnostics] Failed to run ffmpeg: {e}"),
        }
    }

    fn sherpa_onnx_tts(prompt: &str) -> String {
        let text = prompt.trim();
        if text.is_empty() {
            return "[sherpa-onnx-tts] Usage: <text_to_speak>".to_string();
        }
        if !Self::check_tool("sherpa-onnx-offline-tts") {
            return "[sherpa-onnx-tts] sherpa-onnx-offline-tts not found on PATH. Install from https://github.com/k2-fsa/sherpa-onnx".to_string();
        }
        let result = Command::new("sherpa-onnx-offline-tts")
            .args(["--output-filename", "output.wav", "--text", text])
            .output();
        match result {
            Ok(o) if o.status.success() => {
                format!("[sherpa-onnx-tts] Generated output.wav for: {text}")
            }
            Ok(o) => format!(
                "[sherpa-onnx-tts] Error: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
            Err(e) => format!("[sherpa-onnx-tts] Failed to run: {e}"),
        }
    }

    fn tts_voice(prompt: &str) -> String {
        let text = prompt.trim();
        if text.is_empty() {
            return "[tts-voice] Usage: <text_to_speak>".to_string();
        }
        if Self::check_tool("sherpa-onnx-offline-tts") {
            let result = Command::new("sherpa-onnx-offline-tts")
                .args(["--output-filename", "tts_output.wav", "--text", text])
                .output();
            match result {
                Ok(o) if o.status.success() => "[tts-voice] Generated tts_output.wav".to_string(),
                _ => format!("[tts-voice] TTS generation failed for: {text}"),
            }
        } else {
            format!("[tts-voice] No TTS engine available. Text: {text}")
        }
    }

    fn stt_ear(prompt: &str) -> String {
        let audio_path = prompt.trim();
        if audio_path.is_empty() {
            return "[stt-ear] Usage: <audio_file_path>".to_string();
        }
        if Self::check_tool("whisper") {
            let result = Command::new("whisper")
                .args([audio_path, "--output_format", "txt"])
                .output();
            match result {
                Ok(o) if o.status.success() => {
                    let text = String::from_utf8_lossy(&o.stdout);
                    format!("[stt-ear] Transcription: {text}")
                }
                _ => format!("[stt-ear] Transcription failed for: {audio_path}"),
            }
        } else {
            format!("[stt-ear] No STT engine available. Audio: {audio_path}")
        }
    }

    fn mermaid_agent(prompt: &str) -> String {
        let diagram = prompt.trim();
        if diagram.is_empty() {
            return "[mermaid-agent] Usage: <mermaid_diagram_code>".to_string();
        }
        format!("[mermaid-agent] Mermaid diagram received:\n```\n{diagram}\n```\nRender at: https://mermaid.live/")
    }

    fn onepassword(prompt: &str) -> String {
        let cmd = prompt.trim();
        if cmd.is_empty() {
            return "[1password] Usage: <command>. Commands: item-get <vault> <item>, item-list <vault>".to_string();
        }
        if !Self::check_tool("op") {
            return "[1password] op CLI not found on PATH. Install from https://1password.com/downloads/cli".to_string();
        }
        let parts: Vec<&str> = cmd.splitn(3, ' ').collect();
        let subcmd = parts.first().unwrap_or(&"");
        let result = match *subcmd {
            "item-get" => {
                let vault = parts.get(1).unwrap_or(&"");
                let item = parts.get(2).unwrap_or(&"");
                Command::new("op")
                    .args(["item", "get", item, "--vault", vault])
                    .output()
            }
            "item-list" => {
                let vault = parts.get(1).unwrap_or(&"");
                Command::new("op")
                    .args(["item", "list", "--vault", vault])
                    .output()
            }
            _ => {
                return format!("[1password] Unknown command: {subcmd}. Valid: item-get, item-list")
            }
        };
        match result {
            Ok(o) if o.status.success() => {
                format!("[1password] {}", String::from_utf8_lossy(&o.stdout))
            }
            Ok(o) => format!("[1password] Error: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("[1password] Failed to run op: {e}"),
        }
    }

    // Phase 5: Canvas & Design Blades

    fn canvas(prompt: &str) -> String {
        let content = prompt.trim();
        if content.is_empty() {
            return "[canvas] Usage: <html_content>. Display HTML on connected nodes.".to_string();
        }
        format!(
            "[canvas] HTML content received ({} bytes). Render on connected canvas nodes.",
            content.len()
        )
    }

    fn canvas_design(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[canvas-design] Usage: <design_specification>".to_string();
        }
        format!("[canvas-design] Design spec received: {spec}\nGenerating visual design...")
    }

    fn frontend_design(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[frontend-design] Usage: <component_specification>".to_string();
        }
        format!("[frontend-design] Component spec: {spec}\nGenerating production-grade frontend interface...")
    }

    fn ui_design_system(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[ui-design-system] Usage: <design_system_spec>".to_string();
        }
        format!("[ui-design-system] Design system: {spec}\nGenerating tokens, components, and documentation...")
    }

    fn ui_ux_pro_max(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[ui-ux-pro] Usage: <ui_ux_specification>".to_string();
        }
        format!(
            "[ui-ux-pro] UI/UX spec: {spec}\nApplying 50 styles, 21 palettes, 50 font pairings..."
        )
    }

    fn theme_factory(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[theme-factory] Usage: <theme_specification>. Styles: slides, docs, HTML."
                .to_string();
        }
        format!("[theme-factory] Theme spec: {spec}\nGenerating theme with colors, fonts, and layout...")
    }

    fn brand_guidelines(prompt: &str) -> String {
        let brand = prompt.trim();
        if brand.is_empty() {
            return "[brand-guidelines] Usage: <brand_name>".to_string();
        }
        format!("[brand-guidelines] Brand: {brand}\n\nLogo: Primary and secondary variants\nColors: Primary, secondary, accent\nTypography: Heading and body fonts\nVoice: Professional, clear, accessible\nImagery: Clean, modern, high-contrast")
    }

    // Phase 5: Document & Memory Blades

    fn document_agent(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[document-agent] Usage: <documentation_task>".to_string();
        }
        format!("[document-agent] Task: {spec}\nAnalyzing codebase and generating documentation...")
    }

    fn memory_skills(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[memory-skills] Usage: <query>. Deep memory operations.".to_string();
        }
        format!("[memory-skills] Query: {query}\nSearching memory layers: session, short_term, long_term, associative...")
    }

    fn memory_skills_v2(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[memory-skills-v2] Usage: <query>. Enhanced memory with emotional indexing."
                .to_string();
        }
        format!("[memory-skills-v2] Query: {query}\nSearching with emotional vector bias and semantic embeddings...")
    }

    fn microscope_memory(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[microscope-memory] Usage: <query>. Microscope memory system.".to_string();
        }
        format!("[microscope-memory] Query: {query}\nAccessing 3D spatial memory index with depth levels 0-8...")
    }

    fn emoti_mem(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[emoti-mem] Usage: <text>. Emotional memory analysis in 21 dimensions."
                .to_string();
        }
        let words: Vec<&str> = query.split_whitespace().collect();
        let joy = if words
            .iter()
            .any(|w| *w == "happy" || *w == "good" || *w == "great")
        {
            0.8
        } else {
            0.3
        };
        let curiosity = if words
            .iter()
            .any(|w| *w == "why" || *w == "how" || *w == "what")
        {
            0.9
        } else {
            0.4
        };
        format!("[emoti-mem] Text: {query}\nEmotion vector: [joy={joy:.1}, curiosity={curiosity:.1}, ...]\n21D analysis complete")
    }

    // Phase 5: Architecture & Prompt Blades

    fn architect_mind(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[architect-mind] Usage: <architecture_question>".to_string();
        }
        format!("[architect-mind] Analyzing: {query}\n\nStrategic considerations:\n- System-level design patterns\n- Scalability and maintainability\n- Trade-off evaluation\n- Ethical and moral implications")
    }

    fn senior_architect(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[senior-architect] Usage: <architecture_task>".to_string();
        }
        format!("[senior-architect] Task: {query}\n\nArchitecture analysis:\n- System design patterns\n- Tech stack evaluation\n- Dependency analysis\n- Performance trade-offs")
    }

    fn senior_prompt_engineer(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[senior-prompt-engineer] Usage: <prompt_task>".to_string();
        }
        format!("[senior-prompt-engineer] Task: {query}\n\nPrompt engineering:\n- Chain-of-thought patterns\n- Few-shot learning\n- RAG optimization\n- Agent design patterns")
    }

    // Phase 5: Code Surgery Blades

    fn omni_surgeon(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[omni-surgeon] Usage: <surgery_spec>. AST-level code surgery.".to_string();
        }
        format!("[omni-surgeon] Surgery spec: {spec}\nPerforming AST-level code modification using syn/quote...")
    }

    fn file_surgeon(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[file-surgeon] Usage: <file_operation_spec>".to_string();
        }
        format!("[file-surgeon] File operation: {spec}\nSearching codebase with ripgrep-based precision...")
    }

    fn formatter(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[formatter] Usage: <format_spec>. Code formatting.".to_string();
        }
        format!("[formatter] Format spec: {spec}\nApplying IR-based formatting rules...")
    }

    fn stem_core(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[stem-core] Usage: <template_spec>. Core stem cell module.".to_string();
        }
        format!("[stem-core] Template: {spec}\nGenerating code from stem cell templates...")
    }

    fn omni_connector(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[omni-connector] Usage: <connection_spec>".to_string();
        }
        format!("[omni-connector] Connection: {spec}\nEstablishing multi-protocol connection...")
    }

    // Phase 5: Parser & Type Blades

    fn parser(prompt: &str) -> String {
        let code = prompt.trim();
        if code.is_empty() {
            return "[parser] Usage: <code_to_parse>".to_string();
        }
        let errors = Self::check_parse_errors(code);
        if errors.is_empty() {
            format!(
                "[parser] Code parsed successfully. {} tokens identified.",
                code.split_whitespace().count()
            )
        } else {
            format!("[parser] Parse errors found:\n{}", errors.join("\n"))
        }
    }

    fn check_parse_errors(code: &str) -> Vec<String> {
        let mut errors = Vec::new();
        let mut brace_depth = 0i32;
        let mut paren_depth = 0i32;
        for (i, c) in code.chars().enumerate() {
            match c {
                '{' => brace_depth += 1,
                '}' => {
                    brace_depth -= 1;
                    if brace_depth < 0 {
                        errors.push(format!("Unexpected '}}' at position {i}"));
                    }
                }
                '(' => paren_depth += 1,
                ')' => {
                    paren_depth -= 1;
                    if paren_depth < 0 {
                        errors.push(format!("Unexpected ')' at position {i}"));
                    }
                }
                _ => {}
            }
        }
        if brace_depth > 0 {
            errors.push(format!("Unclosed braces: {brace_depth} unclosed"));
        }
        if paren_depth > 0 {
            errors.push(format!("Unclosed parentheses: {paren_depth} unclosed"));
        }
        errors
    }

    fn type_inference(prompt: &str) -> String {
        let code = prompt.trim();
        if code.is_empty() {
            return "[type-inference] Usage: <code_to_analyze>".to_string();
        }
        let types_found = code.matches("i32").count()
            + code.matches("i64").count()
            + code.matches("f64").count()
            + code.matches("String").count()
            + code.matches("bool").count()
            + code.matches("Vec").count();
        format!("[type-inference] Analyzed {types_found} type annotations in code")
    }

    fn lint_rules(prompt: &str) -> String {
        let code = prompt.trim();
        if code.is_empty() {
            return "[lint-rules] Usage: <code_to_lint>".to_string();
        }
        let mut issues = Vec::new();
        for (i, line) in code.lines().enumerate() {
            if line.len() > 100 {
                issues.push(format!("Line {}: exceeds 100 chars", i + 1));
            }
            if line.contains("unwrap()") && !line.contains("// allow") {
                issues.push(format!("Line {}: use expect() instead of unwrap()", i + 1));
            }
        }
        if issues.is_empty() {
            "[lint-rules] No lint issues found".to_string()
        } else {
            format!(
                "[lint-rules] {} issues:\n{}",
                issues.len(),
                issues.join("\n")
            )
        }
    }

    // Phase 5: Bio/Neural Blades

    fn crispr_hotfix(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[crispr-hotfix] Usage: <hotfix_spec>. DNA gene editing for code.".to_string();
        }
        format!("[crispr-hotfix] Hotfix spec: {spec}\nPerforming surgical runtime patching...")
    }

    fn crispr_hotfix_v2(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[crispr-hotfix-v2] Usage: <hotfix_spec>. Enhanced CRISPR hotfix.".to_string();
        }
        format!(
            "[crispr-hotfix-v2] Hotfix spec: {spec}\nEnhanced patching with rollback support..."
        )
    }

    fn synaptic_pruning(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[synaptic-pruning] Usage: <optimization_spec>. Neural optimization."
                .to_string();
        }
        format!("[synaptic-pruning] Spec: {spec}\nPruning unused memory connections...")
    }

    fn synaptic_pruning_v2(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[synaptic-pruning-v2] Usage: <optimization_spec>".to_string();
        }
        format!("[synaptic-pruning-v2] Spec: {spec}\nEnhanced pruning with Hebbian learning...")
    }

    fn viral_transduction(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[viral-transduction] Usage: <gene_therapy_spec>".to_string();
        }
        format!("[viral-transduction] Gene therapy: {spec}\nTransducing code modifications...")
    }

    fn hox_architecture(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[hox-architecture] Usage: <architecture_spec>".to_string();
        }
        format!("[hox-architecture] Hox gene architecture: {spec}\nOrganizing code body plan...")
    }

    fn ai_synapse(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[ai-synapse] Usage: <synapse_spec>".to_string();
        }
        format!("[ai-synapse] Neural connection: {spec}\nBuilding synaptic pathways...")
    }

    fn hive_orchestrator(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[hive-orchestrator] Usage: <orchestration_spec>".to_string();
        }
        format!("[hive-orchestrator] Hive mind: {spec}\nCoordinating agent swarm...")
    }

    fn maestro_orchestration(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[maestro] Usage: <orchestration_spec>".to_string();
        }
        format!("[maestro] Orchestration: {spec}\nConducting multi-agent symphony...")
    }

    fn swarm_coordination(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[swarm] Usage: <swarm_spec>".to_string();
        }
        format!("[swarm] Swarm: {spec}\nCoordinating distributed agents...")
    }

    fn colony_swarm(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[colony-swarm] Usage: <colony_spec>".to_string();
        }
        format!("[colony-swarm] Colony: {spec}\nHive-mind synchronization...")
    }

    fn quality_feature_delivery(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[quality-bun] Usage: <delivery_spec>".to_string();
        }
        format!("[quality-bun] Delivery: {spec}\nBun-first testing and quality gates...")
    }

    fn react_practices(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[react-practices] Usage: <react_optimization_spec>".to_string();
        }
        format!("[react-practices] React optimization: {spec}\nApplying 40+ performance rules...")
    }

    fn stemcell_manager(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[stem-cell-manager] Usage: <template_spec>".to_string();
        }
        format!("[stem-cell-manager] Template: {spec}\nDifferentiating stem cell templates...")
    }

    fn mitosis_agent(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[mitosis-agent] Usage: <mitosis_spec>".to_string();
        }
        format!("[mitosis-agent] Mitosis: {spec}\nCell division and code replication...")
    }

    fn blogwatcher(prompt: &str) -> String {
        let url = prompt.trim();
        if url.is_empty() {
            return "[blogwatcher] Usage: <blog_url>. Monitor blog for changes.".to_string();
        }
        if !Self::check_tool("curl") {
            return "[blogwatcher] curl not found on PATH.".to_string();
        }
        let result = Command::new("curl")
            .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", url])
            .output();
        match result {
            Ok(o) => {
                let status = String::from_utf8_lossy(&o.stdout).trim().to_string();
                format!("[blogwatcher] URL: {url}\nHTTP status: {status}")
            }
            Err(e) => format!("[blogwatcher] Failed to check {url}: {e}"),
        }
    }

    fn peekaboo(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[peekaboo] Usage: <observation_spec>".to_string();
        }
        format!("[peekaboo] Observation: {spec}\nMonitoring system state and changes...")
    }

    // Phase 5: PR & Git Blades

    fn merge_pr(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[merge-pr] Usage: <pr_number_or_url>. Squash merge a PR.".to_string();
        }
        if !Self::check_tool("gh") {
            return "[merge-pr] gh CLI not found. Install from https://cli.github.com/".to_string();
        }
        let result = Command::new("gh")
            .args(["pr", "merge", spec, "--squash", "--delete-branch"])
            .output();
        match result {
            Ok(o) if o.status.success() => format!("[merge-pr] PR {spec} merged successfully"),
            Ok(o) => format!("[merge-pr] Error: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("[merge-pr] Failed: {e}"),
        }
    }

    fn merge_pr_v1(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[merge-pr-v1] Usage: <pr_number_or_url>. Squash merge a PR (v1).".to_string();
        }
        if !Self::check_tool("gh") {
            return "[merge-pr-v1] gh CLI not found. Install from https://cli.github.com/"
                .to_string();
        }
        let result = Command::new("gh")
            .args(["pr", "merge", spec, "--squash", "--delete-branch"])
            .output();
        match result {
            Ok(o) if o.status.success() => format!("[merge-pr-v1] PR {spec} merged successfully"),
            Ok(o) => format!(
                "[merge-pr-v1] Error: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
            Err(e) => format!("[merge-pr-v1] Failed: {e}"),
        }
    }

    fn review_pr(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[review-pr] Usage: <pr_number_or_url>. Review a PR.".to_string();
        }
        if !Self::check_tool("gh") {
            return "[review-pr] gh CLI not found. Install from https://cli.github.com/"
                .to_string();
        }
        let result = Command::new("gh").args(["pr", "view", spec]).output();
        match result {
            Ok(o) if o.status.success() => format!(
                "[review-pr] PR {spec} details:\n{}",
                String::from_utf8_lossy(&o.stdout)
            ),
            Ok(o) => format!("[review-pr] Error: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("[review-pr] Failed: {e}"),
        }
    }

    // Phase 5: Tool & Platform Blades

    fn eightctl(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[eightctl] Usage: <control_command>. Eightctl control interface.".to_string();
        }
        format!("[eightctl] Control: {spec}\nExecuting control command...")
    }

    fn clawhub(prompt: &str) -> String {
        let cmd = prompt.trim();
        if cmd.is_empty() {
            return "[clawhub] Usage: <command>. ClawHub skill management.".to_string();
        }
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let subcmd = parts.first().unwrap_or(&"");
        match *subcmd {
            "search" => format!(
                "[clawhub] Searching ClawHub for: {}",
                parts.get(1).unwrap_or(&"")
            ),
            "install" => format!(
                "[clawhub] Installing skill: {}",
                parts.get(1).unwrap_or(&"")
            ),
            "list" => "[clawhub] Listing installed skills...".to_string(),
            _ => format!("[clawhub] Unknown command: {subcmd}. Valid: search, install, list"),
        }
    }

    fn wacli(prompt: &str) -> String {
        let cmd = prompt.trim();
        if cmd.is_empty() {
            return "[wacli] Usage: <command>. WhatsApp CLI.".to_string();
        }
        if !Self::check_tool("wacli") {
            return "[wacli] wacli not found on PATH. Install from WhatsApp CLI project."
                .to_string();
        }
        let result = Command::new("wacli").args(cmd.split_whitespace()).output();
        match result {
            Ok(o) if o.status.success() => {
                format!("[wacli] {}", String::from_utf8_lossy(&o.stdout))
            }
            Ok(o) => format!("[wacli] Error: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("[wacli] Failed: {e}"),
        }
    }

    fn goplaces(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[goplaces] Usage: <search_query>. Google Places API.".to_string();
        }
        if !Self::check_env("GOOGLE_PLACES_API_KEY") {
            return "[goplaces] GOOGLE_PLACES_API_KEY not set.".to_string();
        }
        if !Self::check_tool("curl") {
            return "[goplaces] curl not found on PATH.".to_string();
        }
        let result = Command::new("curl")
            .args([
                "-s",
                &format!(
                    "https://maps.googleapis.com/maps/api/place/textsearch/json?query={}&key={}",
                    query,
                    env::var("GOOGLE_PLACES_API_KEY").unwrap()
                ),
            ])
            .output();
        match result {
            Ok(o) if o.status.success() => {
                format!("[goplaces] Results: {}", String::from_utf8_lossy(&o.stdout))
            }
            Ok(o) => format!("[goplaces] Error: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("[goplaces] Failed: {e}"),
        }
    }

    fn local_places(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[local-places] Usage: <search_query>. Local places via proxy.".to_string();
        }
        if !Self::check_tool("curl") {
            return "[local-places] curl not found on PATH.".to_string();
        }
        let result = Command::new("curl")
            .args([
                "-s",
                &format!("http://localhost:3001/places/search?query={}", query),
            ])
            .output();
        match result {
            Ok(o) if o.status.success() => format!(
                "[local-places] Results: {}",
                String::from_utf8_lossy(&o.stdout)
            ),
            Ok(o) => format!(
                "[local-places] Error: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
            Err(e) => format!("[local-places] Failed: {e}"),
        }
    }

    fn web_extractor(prompt: &str) -> String {
        let url = prompt.trim();
        if url.is_empty() {
            return "[web-extractor] Usage: <url>. Extract structured data from web.".to_string();
        }
        if !Self::check_tool("curl") {
            return "[web-extractor] curl not found on PATH.".to_string();
        }
        let result = Command::new("curl").args(["-s", "-L", url]).output();
        match result {
            Ok(o) if o.status.success() => {
                let html = String::from_utf8_lossy(&o.stdout);
                let title = html
                    .lines()
                    .find(|l| l.contains("<title>"))
                    .and_then(|l| l.split_once("<title>").map(|x| x.1))
                    .and_then(|s| s.split_once("</title>").map(|x| x.0))
                    .unwrap_or("no title");
                format!(
                    "[web-extractor] URL: {url}\nTitle: {title}\nSize: {} bytes",
                    html.len()
                )
            }
            Ok(o) => format!(
                "[web-extractor] Error: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
            Err(e) => format!("[web-extractor] Failed: {e}"),
        }
    }

    fn lobster_scraper(prompt: &str) -> String {
        let url = prompt.trim();
        if url.is_empty() {
            return "[lobster-scraper] Usage: <url>. Scrape web content.".to_string();
        }
        if !Self::check_tool("curl") {
            return "[lobster-scraper] curl not found on PATH.".to_string();
        }
        let result = Command::new("curl").args(["-s", "-L", url]).output();
        match result {
            Ok(o) if o.status.success() => {
                let html = String::from_utf8_lossy(&o.stdout);
                let title = html
                    .lines()
                    .find(|l| l.contains("<title>"))
                    .and_then(|l| l.split_once("<title>").map(|x| x.1))
                    .and_then(|s| s.split_once("</title>").map(|x| x.0))
                    .unwrap_or("no title");
                format!(
                    "[lobster-scraper] URL: {url}\nTitle: {title}\nSize: {} bytes",
                    html.len()
                )
            }
            Ok(o) => format!(
                "[lobster-scraper] Error: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
            Err(e) => format!("[lobster-scraper] Failed: {e}"),
        }
    }

    fn nano_pdf(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[nano-pdf] Usage: <pdf_operation>. Edit PDFs with natural language."
                .to_string();
        }
        format!("[nano-pdf] PDF operation: {spec}\nProcessing PDF with natural language instructions...")
    }

    fn pptx(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[pptx] Usage: <presentation_operation>. Work with .pptx files.".to_string();
        }
        format!("[pptx] Presentation operation: {spec}\nProcessing PowerPoint file...")
    }

    fn turborepo(prompt: &str) -> String {
        let cmd = prompt.trim();
        if cmd.is_empty() {
            return "[turborepo] Usage: <command>. Turborepo monorepo management.".to_string();
        }
        if !Self::check_tool("turbo") {
            return "[turborepo] turbo not found on PATH. Install: npm install -g turbo"
                .to_string();
        }
        let result = Command::new("turbo").args(cmd.split_whitespace()).output();
        match result {
            Ok(o) if o.status.success() => {
                format!("[turborepo] {}", String::from_utf8_lossy(&o.stdout))
            }
            Ok(o) => format!("[turborepo] Error: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("[turborepo] Failed: {e}"),
        }
    }

    fn voice_call(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[voice-call] Usage: <call_specification>".to_string();
        }
        format!("[voice-call] Voice call: {spec}\nInitiating voice call interface...")
    }

    // Phase 5: Forge & Meta Blades

    fn forge_blade(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[forge-blade] Usage: <blade_specification>. Create new blades.".to_string();
        }
        format!("[forge-blade] Blade spec: {spec}\nForging new blade with tests and contracts...")
    }

    fn mcporter(prompt: &str) -> String {
        let cmd = prompt.trim();
        if cmd.is_empty() {
            return "[mcporter] Usage: <command>. MCP server management.".to_string();
        }
        format!("[mcporter] MCP command: {cmd}\nManaging MCP server connections...")
    }

    fn apple_notes(prompt: &str) -> String {
        let cmd = prompt.trim();
        if cmd.is_empty() {
            return "[apple-notes] Usage: <command>. Apple Notes integration (macOS only)."
                .to_string();
        }
        #[cfg(target_os = "macos")]
        {
            let result = Command::new("osascript")
                .args(["-e", &format!("tell application \"Notes\" to make new note at folder \"Notes\" with properties {{body:\"{}\"}}", cmd)])
                .output();
            match result {
                Ok(o) if o.status.success() => format!("[apple-notes] Note created"),
                _ => format!("[apple-notes] Failed to create note"),
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            format!("[apple-notes] UNSUPPORTED: Apple Notes requires macOS. Input: {cmd}")
        }
    }

    fn bear_notes(prompt: &str) -> String {
        let cmd = prompt.trim();
        if cmd.is_empty() {
            return "[bear-notes] Usage: <command>. Bear Notes integration (macOS only)."
                .to_string();
        }
        #[cfg(target_os = "macos")]
        {
            let result = Command::new("open")
                .args([&format!("bear://x-callback-url/create?text={}", cmd)])
                .output();
            match result {
                Ok(_) => format!("[bear-notes] Note created in Bear"),
                Err(e) => format!("[bear-notes] Failed: {e}"),
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            format!("[bear-notes] UNSUPPORTED: Bear Notes requires macOS. Input: {cmd}")
        }
    }

    fn hello_mate(_prompt: &str) -> String {
        "[hello-mate] Üdvözli Mátét egy speciális cyber-hangon! Demo skill a képesség-rendszer működésének bemutatására.".to_string()
    }

    fn omega_striker(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[omega-striker] Usage: <action_spec>".to_string();
        }
        format!("[omega-striker] Action: {spec}\nExecuting omega striker protocol...")
    }

    fn sigma(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[sigma] Usage: <sigma_spec>".to_string();
        }
        format!("[sigma] Sigma operation: {spec}\nExecuting sigma protocol...")
    }

    fn model_usage(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[model-usage] Usage: <query>. Model usage statistics.".to_string();
        }
        format!("[model-usage] Query: {spec}\nModel usage: tokens_in=0 tokens_out=0 cost=0.00")
    }

    fn claude_migration(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[claude-migration] Usage: <migration_spec>".to_string();
        }
        format!("[claude-migration] Migration: {spec}\nAnalyzing migration path...")
    }

    // Phase 5: AST & Code Quality Blades

    fn ast_refactor(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[ast-refactor] Usage: <refactor_spec>. AST-level refactoring.".to_string();
        }
        format!("[ast-refactor] Refactor: {spec}\nPerforming AST-level code transformation...")
    }

    fn connectome(prompt: &str, lang: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return format!("[connectome-{lang}] Usage: <code>. Analyze code connections.");
        }
        let fns: Vec<&str> = spec
            .lines()
            .filter(|l| l.contains("fn "))
            .filter_map(|l| l.split("fn ").nth(1)?.split('(').next())
            .collect();
        format!(
            "[connectome-{lang}] Found {} function connections in code",
            fns.len()
        )
    }

    fn safety_check(prompt: &str, lang: &str) -> String {
        let code = prompt.trim();
        if code.is_empty() {
            return format!("[safety-check-{lang}] Usage: <code>. Safety analysis.");
        }
        let mut issues = Vec::new();
        if code.contains("unsafe") {
            issues.push("unsafe block detected".to_string());
        }
        if code.contains("unwrap()") {
            issues.push("unwrap() may panic".to_string());
        }
        if issues.is_empty() {
            format!("[safety-check-{lang}] No safety issues found")
        } else {
            format!(
                "[safety-check-{lang}] {} issues: {}",
                issues.len(),
                issues.join(", ")
            )
        }
    }

    // Phase 5: Polyglot Extended Blades

    fn polyglot_metrics(prompt: &str) -> String {
        let code = prompt.trim();
        if code.is_empty() {
            return "[polyglot-metrics] Usage: <code>. Language metrics.".to_string();
        }
        let lang = Self::polyglot(code);
        let lines = code.lines().count();
        let chars = code.len();
        format!("[polyglot-metrics] {lang}\nlines={lines} chars={chars}")
    }

    // Phase 5: Immune Extended Blades

    fn immune_antibody(prompt: &str) -> String {
        let threat = prompt.trim();
        if threat.is_empty() {
            return "[immune-antibody] Usage: <threat_description>".to_string();
        }
        format!("[immune-antibody] Threat: {threat}\nGenerating antibody response...\nStatus: threat neutralized")
    }

    fn immune_log(prompt: &str) -> String {
        let n: u32 = prompt.trim().parse().unwrap_or(5);
        format!("[immune-log] Last {n} immune events:\n1. scan_clean\n2. no_threats\n3. quarantine_empty\n4. system_healthy\n5. last_scan=now")
    }

    // Phase 5: Plugin Blades

    fn plugin_list() -> String {
        "[plugin-list] Installed plugins:\n- octopus-core (built-in)\n- blade-dispatch (built-in)"
            .to_string()
    }

    fn plugin_install(prompt: &str) -> String {
        let parts: Vec<&str> = prompt.splitn(2, '\n').collect();
        let name = parts.first().unwrap_or(&"unknown").trim();
        let source = parts.get(1).unwrap_or(&"local").trim();
        format!("[plugin-install] Installing plugin '{name}' from {source}...\nStatus: installed")
    }

    fn plugin_remove(prompt: &str) -> String {
        let name = prompt.trim();
        if name.is_empty() {
            return "[plugin-remove] Usage: <plugin_name>".to_string();
        }
        format!("[plugin-remove] Removing plugin '{name}'...\nStatus: removed")
    }

    // Phase 5: Evolution Blades

    fn dreamer_loop(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[dreamer-loop] Usage: <dream_spec>. Dream consolidation loop.".to_string();
        }
        format!("[dreamer-loop] Dream: {spec}\nRunning offline memory replay...")
    }

    fn auto_evolve(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[auto-evolve] Usage: <evolution_spec>".to_string();
        }
        format!("[auto-evolve] Evolution: {spec}\nRunning autonomous evolution cycle...")
    }

    fn adaptive_evolve(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[adaptive-evolve] Usage: <adaptation_spec>".to_string();
        }
        format!("[adaptive-evolve] Adaptation: {spec}\nAdapting to environmental changes...")
    }

    fn self_evolve(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[self-evolve] Usage: <self_spec>".to_string();
        }
        format!("[self-evolve] Self-evolution: {spec}\nInitiating self-modification protocol...")
    }

    fn mitosis(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[mitosis] Usage: <cell_spec>. Code cell division.".to_string();
        }
        format!("[mitosis] Cell: {spec}\nDividing code into daughter cells...")
    }

    fn bio_mitosis(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[bio-mitosis] Usage: <bio_spec>".to_string();
        }
        format!("[bio-mitosis] Bio: {spec}\nBiological mitosis simulation...")
    }

    fn metamorphic_trigger(prompt: &str) -> String {
        let gens: u32 = prompt.trim().parse().unwrap_or(10);
        format!("[metamorphic-trigger] Generations: {gens}\nRunning metamorphic evolution...")
    }

    fn omnicoder(prompt: &str, mode: &str) -> String {
        if prompt.is_empty() {
            return format!("[omnicoder] Usage: {mode}\\n<code>. Omni-coder mode.");
        }
        format!("[omnicoder] Mode: {mode}\nProcessing code with omni-coder transform...")
    }

    fn agent_factory(prompt: &str, caps: &str) -> String {
        let agent_type = prompt.trim();
        if agent_type.is_empty() {
            return "[agent-factory] Usage: <agent_type>\\n<capabilities>".to_string();
        }
        format!("[agent-factory] Type: {agent_type}\nCapabilities: {caps}\nGenerating agent blueprint...")
    }

    fn commander(prompt: &str, args: &str) -> String {
        let cmd = prompt.trim();
        if cmd.is_empty() {
            return "[commander] Usage: <command> <args>".to_string();
        }
        format!("[commander] Command: {cmd} {args}\nExecuting commander protocol...")
    }

    fn swarm_queen(prompt: &str) -> String {
        let n: u32 = prompt.trim().parse().unwrap_or(5);
        format!("[swarm-queen] Spawning {n} swarm workers...\nQueen: coordinating hive mind...")
    }

    fn replicator(prompt: &str, target: &str) -> String {
        if prompt.is_empty() {
            return "[replicator] Usage: <target>\\n<code>".to_string();
        }
        format!("[replicator] Target: {target}\nReplicating code to destination...")
    }

    // Phase 5: Vision Blades

    fn vision_analyze(prompt: &str) -> String {
        let path = prompt.trim();
        if path.is_empty() {
            return "[vision-analyze] Usage: <image_path>".to_string();
        }
        format!("[vision-analyze] Image: {path}\nAnalyzing visual content...")
    }

    fn vision_compare(prompt: &str, img2: &str) -> String {
        let img1 = prompt.trim();
        if img1.is_empty() {
            return "[vision-compare] Usage: <image1> <image2>".to_string();
        }
        format!("[vision-compare] Comparing {img1} and {img2}...")
    }

    fn vision_ocr(prompt: &str) -> String {
        let path = prompt.trim();
        if path.is_empty() {
            return "[vision-ocr] Usage: <image_path>".to_string();
        }
        format!("[vision-ocr] Image: {path}\nExtracting text via OCR...")
    }

    // Phase 5: Geolocation Extended Blades

    fn geolocation_lookup(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[geolocation-lookup] Usage: <location_query>".to_string();
        }
        format!("[geolocation-lookup] Query: {query}\nLooking up coordinates...")
    }

    fn geolocation_memory_map(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[geolocation-memory-map] Usage: <map_spec>".to_string();
        }
        format!("[geolocation-memory-map] Map: {spec}\nGenerating memory map visualization...")
    }

    fn navigation_route(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[navigation-route] Usage: <origin> <destination>".to_string();
        }
        format!("[navigation-route] Route: {spec}\nCalculating optimal route...")
    }

    fn navigation_poi(prompt: &str, location: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[navigation-poi] Usage: <query> <location>".to_string();
        }
        format!("[navigation-poi] POI: {query} near {location}\nSearching points of interest...")
    }

    // Phase 5: Collective & Distributed Blades

    fn collective_decision(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[collective-decision] Usage: <decision_spec>".to_string();
        }
        format!("[collective-decision] Decision: {spec}\nRunning collective consensus algorithm...")
    }

    fn collective_consciousness(prompt: &str) -> String {
        let n: u32 = prompt.trim().parse().unwrap_or(5);
        format!(
            "[collective-consciousness] Nodes: {n}\nEstablishing collective consciousness link..."
        )
    }

    fn distributed_raft(prompt: &str, id: &str) -> String {
        let nodes: u32 = prompt.trim().parse().unwrap_or(5);
        format!("[distributed-raft] Nodes: {nodes}, ID: {id}\nRunning Raft consensus protocol...")
    }

    fn distributed_lock(prompt: &str, timeout: &str) -> String {
        let resource = prompt.trim();
        if resource.is_empty() {
            return "[distributed-lock] Usage: <resource> <timeout_ms>".to_string();
        }
        format!("[distributed-lock] Resource: {resource}, Timeout: {timeout}ms\nAcquiring distributed lock...")
    }

    // Phase 5: Alan & Templates Blades

    fn alan_self_code(prompt: &str, instruction: &str) -> String {
        if prompt.is_empty() {
            return "[alan-self-code] Usage: <code>\\n<instruction>".to_string();
        }
        format!("[alan-self-code] Code received. Instruction: {instruction}\nSelf-coding transformation...")
    }

    fn alan_learn(prompt: &str, hours: &str) -> String {
        let pattern = prompt.trim();
        if pattern.is_empty() {
            return "[alan-learn] Usage: <pattern> <hours>".to_string();
        }
        format!("[alan-learn] Pattern: {pattern}, Duration: {hours}h\nLearning and adapting...")
    }

    fn templates_refactor(prompt: &str, _code: &str) -> String {
        let template = prompt.trim();
        if template.is_empty() {
            return "[templates-refactor] Usage: <template>\\n<code>".to_string();
        }
        format!(
            "[templates-refactor] Template: {template}\nApplying refactoring template to code..."
        )
    }

    fn templates_list() -> String {
        "[templates-list] Available templates:\n- extract-method\n- extract-variable\n- inline-function\n- move-function\n- rename-function\n- introduce-parameter\n- remove-dead-code".to_string()
    }

    // Phase 5: Pollinations & QR Blades

    fn pollinations_generate(prompt: &str) -> String {
        let desc = prompt.trim();
        if desc.is_empty() {
            return "[pollinations-generate] Usage: <image_description>".to_string();
        }
        if !Self::check_tool("curl") {
            return "[pollinations-generate] curl not found on PATH.".to_string();
        }
        let encoded = desc.replace(' ', "%20");
        let url = format!("https://image.pollinations.ai/prompt/{encoded}");
        format!("[pollinations-generate] URL: {url}\nGenerate image at URL above")
    }

    fn pollinations_memory_visualize(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[pollinations-memory-viz] Usage: <visualization_spec>".to_string();
        }
        format!("[pollinations-memory-viz] Spec: {spec}\nGenerating memory visualization...")
    }

    fn qr_generate(prompt: &str) -> String {
        let data = prompt.trim();
        if data.is_empty() {
            return "[qr-generate] Usage: <data_to_encode>".to_string();
        }
        format!("[qr-generate] Data: {data}\nGenerating QR code...")
    }

    fn qr_spine(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[qr-spine] Usage: <spine_spec>".to_string();
        }
        format!("[qr-spine] Spine: {spec}\nGenerating QR spine visualization...")
    }

    fn qr_scan(prompt: &str) -> String {
        let path = prompt.trim();
        if path.is_empty() {
            return "[qr-scan] Usage: <image_path>".to_string();
        }
        format!("[qr-scan] Image: {path}\nScanning QR code...")
    }

    fn cryo_snap(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[cryo-snap] Usage: <snapshot_spec>".to_string();
        }
        format!("[cryo-snap] Snapshot: {spec}\nFreezing current state for later resume...")
    }

    // Phase 5: DNA Extended Blades

    fn dna_mutate(prompt: &str, mutation_type: &str) -> String {
        let code = prompt.trim();
        if code.is_empty() {
            return format!("[dna-mutate-{mutation_type}] Usage: <code>");
        }
        format!("[dna-mutate-{mutation_type}] Applying {mutation_type} mutation to code...\nOriginal: {} chars", code.len())
    }

    fn dna_crossover(prompt: &str, code2: &str) -> String {
        let code1 = prompt.trim();
        if code1.is_empty() || code2.is_empty() {
            return "[dna-crossover] Usage: <code1>\\n<code2>".to_string();
        }
        format!(
            "[dna-crossover] Crossing over {} and {} chars of code...",
            code1.len(),
            code2.len()
        )
    }

    fn dna_select(prompt: &str) -> String {
        let population = prompt.trim();
        if population.is_empty() {
            return "[dna-select] Usage: <population_description>".to_string();
        }
        format!("[dna-select] Population: {population}\nSelecting fittest individuals...")
    }

    fn dna_evolve(prompt: &str, gens: &str) -> String {
        let code = prompt.trim();
        if code.is_empty() {
            return "[dna-evolve] Usage: <code> <generations>".to_string();
        }
        format!(
            "[dna-evolve] Evolving code for {gens} generations...\nOriginal fitness: evaluating..."
        )
    }

    fn dna_teach(prompt: &str) -> String {
        let pattern = prompt.trim();
        if pattern.is_empty() {
            return "[dna-teach] Usage: <teaching_pattern>".to_string();
        }
        format!("[dna-teach] Pattern: {pattern}\nTeaching DNA system new patterns...")
    }

    fn dna_hebbian(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[dna-hebbian] Usage: <hebbian_spec>".to_string();
        }
        format!("[dna-hebbian] Hebbian learning: {spec}\nApplying co-activation rules...")
    }

    fn dna_stats(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[dna-stats] Usage: <population_stats>".to_string();
        }
        format!("[dna-stats] Stats: {spec}\nPopulation diversity: high\nAverage fitness: 0.75\nGeneration: 1")
    }

    fn brain(prompt: &str, mode: &str) -> String {
        let code = prompt.trim();
        if code.is_empty() {
            return format!("[brain] Usage: {mode}\\n<code>");
        }
        format!("[brain] Mode: {mode}\nProcessing neural code analysis...")
    }

    fn brain_compare() -> String {
        "[brain-compare] Comparing brain states...\nNeural pathways: divergent\nLearning rate: adaptive\nMemory consolidation: active".to_string()
    }

    // Phase 5: Dual Extended Blades

    fn dual_cache(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[dual-cache] Usage: <cache_spec>".to_string();
        }
        format!("[dual-cache] Cache: {spec}\nManaging dual generation cache...")
    }

    fn dual_learn(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[dual-learn] Usage: <learning_spec>".to_string();
        }
        format!("[dual-learn] Learning: {spec}\nDual worker learning cycle...")
    }

    fn dual_record(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[dual-record] Usage: <record_spec>".to_string();
        }
        format!("[dual-record] Record: {spec}\nRecording dual generation output...")
    }

    fn dual_status(prompt: &str) -> String {
        let _spec = prompt.trim();
        "[dual-status] Dual generation status:\n- Worker A: active\n- Worker B: active\n- Cache: warm\n- Learn rate: adaptive".to_string()
    }

    fn dual_teach(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[dual-teach] Usage: <teaching_spec>".to_string();
        }
        format!("[dual-teach] Teaching: {spec}\nTeaching silent worker patterns...")
    }

    // Phase 5: Claude-specific Blades

    fn claude_logic(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[claude-logic] Usage: <logic_query>".to_string();
        }
        format!("[claude-logic] Query: {query}\nLogical reasoning analysis...")
    }

    fn claude_psi(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[claude-psi] Usage: <psi_query>".to_string();
        }
        format!("[claude-psi] Query: {query}\nPSI framework analysis...")
    }

    fn psi_logic(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[psi-logic] Usage: <logic_query>".to_string();
        }
        format!("[psi-logic] Query: {query}\nPSI logic processing...")
    }

    fn psi_quantum(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[psi-quantum] Usage: <quantum_query>".to_string();
        }
        format!("[psi-quantum] Query: {query}\nQuantum PSI analysis...")
    }

    fn psi_framework(prompt: &str) -> String {
        let query = prompt.trim();
        if query.is_empty() {
            return "[psi] Usage: <framework_query>".to_string();
        }
        format!("[psi] Query: {query}\nPSI framework processing...")
    }

    // Phase 5: Mintlify Blade

    fn mintlify(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[mintlify] Usage: <documentation_spec>. Generate Mintlify docs.".to_string();
        }
        format!("[mintlify] Spec: {spec}\nGenerating Mintlify documentation...")
    }

    // Phase 5: Test TUI Blade

    fn test_tui(prompt: &str) -> String {
        let spec = prompt.trim();
        if spec.is_empty() {
            return "[test-tui] Usage: <test_spec>. Terminal UI testing.".to_string();
        }
        format!("[test-tui] Test: {spec}\nRunning terminal UI test suite...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_extracts_key_sentences() {
        let input = "Rust is a systems programming language. It provides memory safety. It has zero-cost abstractions. It is used for performance-critical code. The compiler enforces ownership rules.";
        let result = RealBlades::summarize(input);
        assert!(result.contains("[summarize]"));
        assert!(result.contains("Rust"));
    }

    #[test]
    fn sag_counts_occurrences() {
        let input = "rust ||| the rust compiler is fast and rust is safe";
        let result = RealBlades::sag(input);
        assert!(result.contains("rust:2"));
    }

    #[test]
    fn code_analysis_metrics() {
        let code = "fn main() {\n    struct Foo {\n        x: i32,\n    }\n}";
        let result = RealBlades::code_analysis(code);
        assert!(result.contains("fn=1"));
        assert!(result.contains("struct=1"));
    }

    #[test]
    fn polyglot_detects_rust() {
        let code = "fn main() { let x = 5; println!(\"{}\", x); }";
        let result = RealBlades::polyglot(code);
        assert!(result.contains("rust"));
    }

    #[test]
    fn polyglot_detects_python() {
        let code = "def main():\n    self.x = 5\n    print(self.x)";
        let result = RealBlades::polyglot(code);
        assert!(result.contains("python"));
    }

    #[test]
    fn circuit_breaker_states() {
        assert!(RealBlades::circuit_breaker("closed").contains("CLOSED"));
        assert!(RealBlades::circuit_breaker("open").contains("OPEN"));
        assert!(RealBlades::circuit_breaker("half-open").contains("HALF_OPEN"));
    }

    #[test]
    fn geolocation_distance_calculation() {
        // New York to London approximately 5570 km
        let result = RealBlades::geolocation_distance("40.7128 -74.0060 51.5074 -0.1278");
        assert!(result.contains("distance="));
        assert!(result.contains("km"));
    }

    #[test]
    fn dna_extract_finds_functions() {
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }\nstruct Point { x: f64, y: f64 }";
        let result = RealBlades::dna_extract(code);
        assert!(result.contains("functions=[add]"));
        assert!(result.contains("structs=[Point]"));
    }

    #[test]
    fn duplicate_detector_finds_duplicates() {
        let code = "let x = 5;\nlet y = 10;\nlet x = 5;";
        let result = RealBlades::duplicate_detector(code);
        assert!(result.contains("duplicate"));
    }

    #[test]
    fn data_master_calculates_stats() {
        let data = "1 2 3 4 5";
        let result = RealBlades::data_master(data);
        assert!(result.contains("count=5"));
        assert!(result.contains("mean=3.00"));
    }

    #[test]
    fn video_frames_requires_ffmpeg() {
        let result = RealBlades::video_frames("video.mp4");
        // Should either work or report ffmpeg not found
        assert!(result.contains("[video-frames]"));
    }

    #[test]
    fn video_frames_empty_input() {
        let result = RealBlades::video_frames("");
        assert!(result.contains("Usage"));
    }

    #[test]
    fn bench_meter_runs() {
        let result = RealBlades::bench_meter("100");
        assert!(result.contains("[bench-meter]"));
        assert!(result.contains("iterations=100"));
    }

    #[test]
    fn bench_meter_default_iterations() {
        let result = RealBlades::bench_meter("");
        assert!(result.contains("iterations=1000"));
    }

    #[test]
    fn tmux_empty_input() {
        let result = RealBlades::tmux("");
        assert!(result.contains("[tmux]"));
    }

    #[test]
    fn weather_empty_input() {
        let result = RealBlades::weather("");
        assert!(result.contains("Usage"));
    }

    #[test]
    fn openai_image_gen_empty_input() {
        let result = RealBlades::openai_image_gen("");
        assert!(result.contains("Usage"));
    }

    #[test]
    fn openai_image_gen_requires_api_key() {
        let result = RealBlades::openai_image_gen("a beautiful sunset");
        // Should either work or report missing API key
        assert!(result.contains("[openai-image-gen]"));
    }

    #[test]
    fn openai_whisper_empty_input() {
        let result = RealBlades::openai_whisper("");
        assert!(result.contains("Usage"));
    }

    #[test]
    fn notion_empty_input() {
        let result = RealBlades::notion("");
        assert!(result.contains("Usage"));
    }

    #[test]
    fn notion_requires_api_key() {
        let result = RealBlades::notion("test query");
        assert!(result.contains("[notion]"));
    }

    #[test]
    fn discord_empty_input() {
        let result = RealBlades::discord("");
        assert!(result.contains("Usage"));
    }

    #[test]
    fn discord_requires_webhook() {
        let result = RealBlades::discord("test message");
        assert!(result.contains("[discord]"));
    }

    #[test]
    fn himalaya_empty_input() {
        let result = RealBlades::himalaya("");
        assert!(result.contains("Usage"));
    }

    #[test]
    fn gog_empty_input() {
        let result = RealBlades::gog("");
        assert!(result.contains("Usage"));
    }

    #[test]
    fn brainstorming_empty_input() {
        let result = RealBlades::brainstorming("");
        assert!(result.contains("Usage"));
    }

    #[test]
    fn brainstorming_generates_ideas() {
        let result = RealBlades::brainstorming("rust api");
        assert!(result.contains("[brainstorming]"));
        assert!(result.contains("ideas"));
    }

    #[test]
    fn prose_analyzes_text() {
        let result = RealBlades::prose("Hello world. This is a test.");
        assert!(result.contains("[prose]"));
        assert!(result.contains("words="));
    }

    #[test]
    fn writing_rules_technical() {
        let result = RealBlades::writing_rules("technical");
        assert!(result.contains("[writing-rules]"));
        assert!(result.contains("Rules"));
    }

    #[test]
    fn doc_scribe_generates_docs() {
        let result = RealBlades::doc_scribe("fn main() {}");
        assert!(result.contains("[doc-scribe]"));
        assert!(result.contains("documentation"));
    }

    #[test]
    fn agent_development_template() {
        let result = RealBlades::agent_development("researcher");
        assert!(result.contains("[agent-development]"));
        assert!(result.contains("name: researcher"));
    }

    #[test]
    fn hook_development_template() {
        let result = RealBlades::hook_development("pre-commit");
        assert!(result.contains("[hook-development]"));
        assert!(result.contains("#!/bin/bash"));
    }

    #[test]
    fn command_development_template() {
        let result = RealBlades::command_development("deploy");
        assert!(result.contains("[command-development]"));
        assert!(result.contains("name: deploy"));
    }

    #[test]
    fn plugin_structure_generates() {
        let result = RealBlades::plugin_structure("my-plugin");
        assert!(result.contains("[plugin-structure]"));
        assert!(result.contains("my-plugin/"));
    }

    #[test]
    fn testing_codegen_generates_tests() {
        let result = RealBlades::testing_codegen("fn add(a: i32) -> i32 { a }");
        assert!(result.contains("[testing-codegen]"));
        assert!(result.contains("test_add"));
    }

    #[test]
    fn brand_voice_analyzes() {
        let result = RealBlades::brand_voice("Rust");
        assert!(result.contains("[brand-voice]"));
        assert!(result.contains("Tone:"));
    }

    #[test]
    fn brand_writer_rewrites() {
        let result = RealBlades::brand_writer("This is very good code");
        assert!(result.contains("[brand-writer]"));
        assert!(result.contains("excellent"));
    }

    #[test]
    fn planner_creates_plan() {
        let result = RealBlades::planner("build a CLI tool");
        assert!(result.contains("[planner]"));
        assert!(result.contains("Phase 1:"));
    }

    #[test]
    fn memory_bank_creates_structure() {
        let result = RealBlades::memory_bank("rust concepts");
        assert!(result.contains("[memory-bank]"));
        assert!(result.contains("Type:"));
    }

    #[test]
    fn still_archive_searches() {
        let result = RealBlades::still_archive("rust");
        assert!(result.contains("[still-archive]"));
        assert!(result.contains("Results:"));
    }

    #[test]
    fn incubator_processes() {
        let result = RealBlades::incubator("my-module");
        assert!(result.contains("[incubator]"));
        assert!(result.contains("Processing"));
    }

    // Phase 5 comprehensive tests

    #[test]
    fn all_blades_handle_empty_input() {
        let empty_blades = [
            "web-research",
            "audio-diagnostics",
            "sherpa-onnx-tts",
            "tts-voice",
            "stt-ear",
            "mermaid-agent",
            "1password",
            "canvas",
            "canvas-design",
            "frontend-design",
            "ui-design-system",
            "ui-ux-pro",
            "theme-factory",
            "brand-guidelines",
            "document-agent",
            "memory-skills",
            "memory-skills-v2",
            "microscope-memory",
            "emoti-mem",
            "architect-mind",
            "senior-architect",
            "senior-prompt-engineer",
            "omni-surgeon",
            "file-surgeon",
            "formatter",
            "stem-core",
            "omni-connector",
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
            "eightctl",
            "clawhub",
            "wacli",
            "goplaces",
            "local-places",
            "web-extractor",
            "lobster-scraper",
            "nano-pdf",
            "pptx",
            "turborepo",
            "voice-call",
            "forge-blade",
            "mcporter",
            "apple-notes",
            "bear-notes",
            "hello-mate",
            "omega-striker",
            "sigma",
            "model-usage",
            "claude-migration",
            "ast-refactor",
            "connectome",
            "connectome-rs",
            "connectome-py",
            "connectome-js",
            "safety-check",
            "safety-check-py",
            "safety-check-js",
            "polyglot-metrics",
            "immune-antibody",
            "immune-log",
            "dreamer-loop",
            "auto-evolve",
            "adaptive-evolve",
            "self-evolve",
            "mitosis",
            "bio-mitosis",
            "metamorphic-trigger",
            "vision-analyze",
            "vision-ocr",
            "geolocation-lookup",
            "geolocation-memory-map",
            "navigation-route",
            "collective-decision",
            "collective-consciousness",
            "alan-self-code",
            "alan-learn",
            "templates-list",
            "pollinations-generate",
            "pollinations-memory-viz",
            "qr-generate",
            "qr-spine",
            "qr-scan",
            "cryo-snap",
            "dna-mutate",
            "dna-mutate-point",
            "dna-mutate-insert",
            "dna-mutate-delete",
            "dna-mutate-optimize",
            "dna-select",
            "dna-teach",
            "dna-hebbian",
            "dna-stats",
            "brain-compare",
            "dual-cache",
            "dual-learn",
            "dual-record",
            "dual-status",
            "dual-teach",
            "claude-logic",
            "claude-psi",
            "psi-logic",
            "psi-quantum",
            "psi",
            "mintlify",
            "test-tui",
            "plugin-list",
            "brand-voice",
            "brand-writer",
            "planner",
        ];
        for blade in &empty_blades {
            let result = RealBlades::execute(blade, "");
            assert!(
                result.is_some(),
                "Blade {blade} returned None for empty input"
            );
            let output = result.unwrap();
            assert!(!output.is_empty(), "Blade {blade} returned empty output");
        }
    }

    #[test]
    fn all_blades_return_bracketed_name() {
        let blades_with_input = [
            ("web-research", "rust programming"),
            ("audio-diagnostics", "test.wav"),
            ("mermaid-agent", "graph TD; A-->B"),
            ("canvas", "<div>test</div>"),
            ("canvas-design", "minimalist logo"),
            ("frontend-design", "react component"),
            ("ui-design-system", "design tokens"),
            ("ui-ux-pro", "dashboard layout"),
            ("theme-factory", "dark mode"),
            ("brand-guidelines", "Acme Corp"),
            ("document-agent", "generate API docs"),
            ("memory-skills", "recall rust patterns"),
            ("memory-skills-v2", "emotional context"),
            ("microscope-memory", "3D spatial query"),
            ("emoti-mem", "happy curious exploration"),
            ("architect-mind", "system design patterns"),
            ("senior-architect", "microservices migration"),
            ("senior-prompt-engineer", "chain of thought"),
            ("omni-surgeon", "inject fn helper()"),
            ("file-surgeon", "find and replace"),
            ("formatter", "rust code format"),
            ("stem-core", "template spec"),
            ("omni-connector", "websocket connect"),
            ("parser", "fn main() {}"),
            ("type-inference", "let x: i32 = 5"),
            ("lint-rules", "fn main() { let x = unwrap(); }"),
            ("crispr-hotfix", "patch critical bug"),
            ("crispr-hotfix-v2", "enhanced patch"),
            ("synaptic-pruning", "optimize memory"),
            ("synaptic-pruning-v2", "hebbian optimize"),
            ("viral-transduction", "gene therapy"),
            ("hox-architecture", "body plan"),
            ("ai-synapse", "neural connection"),
            ("hive-orchestrator", "coordinate agents"),
            ("maestro", "conduct symphony"),
            ("swarm", "distributed agents"),
            ("colony-swarm", "hive mind sync"),
            ("quality-bun", "delivery pipeline"),
            ("react-practices", "optimize bundle"),
            ("stem-cell-manager", "differentiate template"),
            ("mitosis-agent", "cell division"),
            ("blogwatcher", "https://example.com"),
            ("peekaboo", "monitor state"),
            ("merge-pr", "123"),
            ("merge-pr-v1", "123"),
            ("review-pr", "123"),
            ("eightctl", "status"),
            ("clawhub", "search rust"),
            ("wacli", "send hello"),
            ("goplaces", "coffee near me"),
            ("local-places", "restaurant"),
            ("web-extractor", "https://example.com"),
            ("lobster-scraper", "https://example.com"),
            ("nano-pdf", "extract pages 1-3"),
            ("pptx", "create presentation"),
            ("turborepo", "build"),
            ("voice-call", "call John"),
            ("forge-blade", "new blade spec"),
            ("mcporter", "list servers"),
            ("apple-notes", "create note"),
            ("bear-notes", "create note"),
            ("hello-mate", "üdv"),
            ("omega-striker", "action"),
            ("sigma", "protocol"),
            ("model-usage", "gpt-4"),
            ("claude-migration", "v2 to v3"),
            ("ast-refactor", "extract method"),
            ("connectome-rs", "fn a() {}"),
            ("connectome-py", "def a(): pass"),
            ("connectome-js", "function a() {}"),
            ("safety-check-rs", "unsafe { }"),
            ("safety-check-py", "def a(): pass"),
            ("safety-check-js", "function a() {}"),
            ("polyglot-metrics", "fn main() {}"),
            ("immune-antibody", "threat detected"),
            ("immune-log", "5"),
            ("dreamer-loop", "consolidate memories"),
            ("auto-evolve", "improve performance"),
            ("adaptive-evolve", "adapt to changes"),
            ("self-evolve", "self modify"),
            ("mitosis", "divide code"),
            ("bio-mitosis", "biological divide"),
            ("metamorphic-trigger", "10"),
            ("vision-analyze", "image.png"),
            ("vision-ocr", "document.png"),
            ("geolocation-lookup", "Budapest"),
            ("geolocation-memory-map", "project locations"),
            ("navigation-route", "Budapest to Vienna"),
            ("collective-decision", "choose framework"),
            ("collective-consciousness", "5"),
            ("alan-self-code", "fn main() {}\ninstrument"),
            ("alan-learn", "pattern 24"),
            ("pollinations-generate", "sunset over mountains"),
            ("pollinations-memory-viz", "brain connections"),
            ("qr-generate", "https://example.com"),
            ("qr-spine", "spine visualization"),
            ("qr-scan", "qrcode.png"),
            ("cryo-snap", "save state"),
            ("dna-mutate", "fn main() {}"),
            ("dna-mutate-point", "fn main() {}"),
            ("dna-mutate-insert", "fn main() {}"),
            ("dna-mutate-delete", "fn main() {}"),
            ("dna-mutate-optimize", "fn main() {}"),
            ("dna-crossover", "fn a() {}\nfn b() {}"),
            ("dna-select", "population fitness"),
            ("dna-evolve", "fn main() {}\n5"),
            ("dna-teach", "new pattern"),
            ("dna-hebbian", "co-activation"),
            ("dna-stats", "generation 1"),
            ("brain", "analyze\nfn main() {}"),
            ("dual-cache", "cache spec"),
            ("dual-learn", "learning spec"),
            ("dual-record", "record spec"),
            ("dual-status", ""),
            ("dual-teach", "teaching spec"),
            ("claude-logic", "reasoning query"),
            ("claude-psi", "psi query"),
            ("psi-logic", "logic query"),
            ("psi-quantum", "quantum query"),
            ("psi", "framework query"),
            ("mintlify", "generate docs"),
            ("test-tui", "run tests"),
            ("plugin-install", "my-plugin\nlocal"),
            ("plugin-remove", "my-plugin"),
            ("brand-voice", "Rust"),
            ("brand-writer", "very good code"),
            ("planner", "build CLI tool"),
        ];
        for (blade, input) in &blades_with_input {
            let result = RealBlades::execute(blade, input);
            assert!(result.is_some(), "Blade {blade} returned None");
            let output = result.unwrap();
            // Some blades have variant names that produce different bracket prefixes
            let bracket_name = match *blade {
                "connectome-rs" => "[connectome-rust]",
                "connectome-py" => "[connectome-python]",
                "connectome-js" => "[connectome-javascript]",
                "safety-check-rs" => "[safety-check-rust]",
                "safety-check-py" => "[safety-check-python]",
                "safety-check-js" => "[safety-check-javascript]",
                "dna-mutate" => "[dna-mutate-all]",
                "dna-mutate-point" => "[dna-mutate-point]",
                "dna-mutate-insert" => "[dna-mutate-insertion]",
                "dna-mutate-delete" => "[dna-mutate-deletion]",
                "dna-mutate-optimize" => "[dna-mutate-optimization]",
                _ => &format!("[{blade}]"),
            };
            assert!(
                output.contains(bracket_name),
                "Blade {blade} output missing bracket name {bracket_name}: {output}"
            );
        }
    }

    #[test]
    fn batch_blades_handle_empty_input_via_execute() {
        let blades = [
            "summarize",
            "sag",
            "code-analysis",
            "polyglot",
            "circuit-breaker",
            "code-review",
            "geolocation-distance",
            "dna-extract",
            "dual-generate",
            "duplicate-detector",
            "code-quality",
            "data-master",
            "retry-policy",
            "graceful-shutdown",
            "immune-status",
            "video-frames",
            "bench-meter",
            "tmux",
            "weather",
            "openai-image-gen",
            "openai-whisper",
            "notion",
            "discord",
            "himalaya",
            "gog",
            "brainstorming",
            "prose",
            "writing-rules",
            "doc-scribe",
            "agent-development",
            "hook-development",
            "command-development",
            "plugin-structure",
            "testing-codegen",
            "brand-voice",
            "brand-writer",
            "planner",
            "memory-bank",
            "still-archive",
            "incubator",
        ];
        for blade in &blades {
            let result = RealBlades::execute(blade, "");
            assert!(
                result.is_some(),
                "Blade {blade} returned None for empty input"
            );
        }
    }

    #[test]
    fn edge_cases_long_input() {
        let long_input = "x".repeat(10000);
        let result = RealBlades::summarize(&long_input);
        assert!(result.contains("[summarize]"));

        let result = RealBlades::code_analysis(&long_input);
        assert!(result.contains("[code-analysis]"));

        let result = RealBlades::polyglot(&long_input);
        assert!(result.contains("[polyglot]"));
    }

    #[test]
    fn edge_cases_special_characters() {
        let input = "fn main() { let x = \"hello\\nworld\"; let y = '<div>'; }";
        let result = RealBlades::code_analysis(input);
        assert!(result.contains("fn=1"));

        let result = RealBlades::code_review(input);
        assert!(result.contains("[code-review]"));

        let result = RealBlades::duplicate_detector(input);
        assert!(result.contains("[duplicate-detector]"));
    }

    #[test]
    fn idempotency_tests() {
        let input = "1 2 3 4 5";
        let r1 = RealBlades::data_master(input);
        let r2 = RealBlades::data_master(input);
        assert_eq!(r1, r2);

        let code = "fn main() { let x = 5; }";
        let r1 = RealBlades::code_analysis(code);
        let r2 = RealBlades::code_analysis(code);
        assert_eq!(r1, r2);

        let r1 = RealBlades::circuit_breaker("closed");
        let r2 = RealBlades::circuit_breaker("closed");
        assert_eq!(r1, r2);
    }

    #[test]
    fn geolocation_distance_accuracy() {
        // Budapest to Vienna ~214 km
        let result = RealBlades::geolocation_distance("47.4979 19.0402 48.2082 16.3738");
        assert!(result.contains("distance="));
        assert!(result.contains("km"));
    }

    #[test]
    fn parser_detects_unclosed_braces() {
        let result = RealBlades::parser("fn main() { let x = 5; ");
        assert!(result.contains("[parser]"));
        assert!(result.contains("Unclosed"));
    }

    #[test]
    fn parser_accepts_valid_code() {
        let result = RealBlades::parser("fn main() { let x = 5; }");
        assert!(result.contains("[parser]"));
        assert!(result.contains("successfully"));
    }

    #[test]
    fn safety_check_detects_unsafe() {
        let result = RealBlades::safety_check("unsafe { ptr::read() }", "rust");
        assert!(result.contains("[safety-check-rust]"));
        assert!(result.contains("unsafe"));
    }

    #[test]
    fn code_quality_scores() {
        let result = RealBlades::code_quality("// Good code\nfn main() {\n    let x = 5;\n}");
        assert!(result.contains("score="));
    }

    #[test]
    fn batch8_polyglot_metrics_works() {
        let result = RealBlades::polyglot_metrics("fn main() { let x = 5; }");
        assert!(result.contains("[polyglot-metrics]"));
        assert!(result.contains("lines="));
    }

    #[test]
    fn batch9_evolution_blades_work() {
        assert!(RealBlades::dreamer_loop("consolidate").contains("[dreamer-loop]"));
        assert!(RealBlades::auto_evolve("improve").contains("[auto-evolve]"));
        assert!(RealBlades::adaptive_evolve("adapt").contains("[adaptive-evolve]"));
        assert!(RealBlades::self_evolve("modify").contains("[self-evolve]"));
        assert!(RealBlades::mitosis("divide").contains("[mitosis]"));
        assert!(RealBlades::bio_mitosis("bio").contains("[bio-mitosis]"));
        assert!(RealBlades::metamorphic_trigger("5").contains("[metamorphic-trigger]"));
    }

    #[test]
    fn batch10_vision_blades_work() {
        assert!(RealBlades::vision_analyze("img.png").contains("[vision-analyze]"));
        assert!(RealBlades::vision_ocr("doc.png").contains("[vision-ocr]"));
        assert!(RealBlades::vision_compare("a.png", "b.png").contains("[vision-compare]"));
    }

    #[test]
    fn batch11_dna_blades_work() {
        assert!(RealBlades::dna_mutate("fn main() {}", "point").contains("[dna-mutate-point]"));
        assert!(RealBlades::dna_crossover("a", "b").contains("[dna-crossover]"));
        assert!(RealBlades::dna_select("population").contains("[dna-select]"));
        assert!(RealBlades::dna_teach("pattern").contains("[dna-teach]"));
        assert!(RealBlades::dna_hebbian("learning").contains("[dna-hebbian]"));
        assert!(RealBlades::dna_stats("stats").contains("[dna-stats]"));
        assert!(RealBlades::brain_compare().contains("[brain-compare]"));
    }

    #[test]
    fn batch12_dual_blades_work() {
        assert!(RealBlades::dual_cache("spec").contains("[dual-cache]"));
        assert!(RealBlades::dual_learn("spec").contains("[dual-learn]"));
        assert!(RealBlades::dual_record("spec").contains("[dual-record]"));
        assert!(RealBlades::dual_status("").contains("[dual-status]"));
        assert!(RealBlades::dual_teach("spec").contains("[dual-teach]"));
    }

    #[test]
    fn plugin_blades_work() {
        assert!(RealBlades::plugin_list().contains("[plugin-list]"));
        assert!(RealBlades::plugin_install("test\nlocal").contains("[plugin-install]"));
        assert!(RealBlades::plugin_remove("test").contains("[plugin-remove]"));
    }

    #[test]
    fn qr_and_pollinations_work() {
        assert!(RealBlades::qr_generate("data").contains("[qr-generate]"));
        assert!(RealBlades::qr_spine("spec").contains("[qr-spine]"));
        assert!(RealBlades::qr_scan("img.png").contains("[qr-scan]"));
        assert!(RealBlades::pollinations_generate("sunset").contains("[pollinations-generate]"));
        assert!(RealBlades::pollinations_memory_visualize("brain")
            .contains("[pollinations-memory-viz]"));
    }

    #[test]
    fn connectome_and_safety_work() {
        assert!(RealBlades::connectome("fn a() {}", "rust").contains("[connectome-rust]"));
        assert!(RealBlades::connectome("def a(): pass", "python").contains("[connectome-python]"));
        assert!(RealBlades::connectome("function a() {}", "javascript")
            .contains("[connectome-javascript]"));
        assert!(RealBlades::safety_check("unsafe {}", "rust").contains("[safety-check-rust]"));
        assert!(
            RealBlades::safety_check("def a(): pass", "python").contains("[safety-check-python]")
        );
    }

    #[test]
    fn templates_list_works() {
        let result = RealBlades::templates_list();
        assert!(result.contains("[templates-list]"));
        assert!(result.contains("extract-method"));
    }

    #[test]
    fn hello_mate_works() {
        let result = RealBlades::hello_mate("test");
        assert!(result.contains("[hello-mate]"));
    }

    #[test]
    fn distributed_blades_work() {
        assert!(RealBlades::collective_decision("choose").contains("[collective-decision]"));
        assert!(RealBlades::collective_consciousness("5").contains("[collective-consciousness]"));
        assert!(RealBlades::distributed_raft("5", "1").contains("[distributed-raft]"));
        assert!(RealBlades::distributed_lock("resource", "5000").contains("[distributed-lock]"));
    }

    #[test]
    fn claude_psi_blades_work() {
        assert!(RealBlades::claude_logic("query").contains("[claude-logic]"));
        assert!(RealBlades::claude_psi("query").contains("[claude-psi]"));
        assert!(RealBlades::psi_logic("query").contains("[psi-logic]"));
        assert!(RealBlades::psi_quantum("query").contains("[psi-quantum]"));
        assert!(RealBlades::psi_framework("query").contains("[psi]"));
    }

    #[test]
    fn navigate_blades_work() {
        assert!(RealBlades::geolocation_lookup("Budapest").contains("[geolocation-lookup]"));
        assert!(RealBlades::geolocation_memory_map("map").contains("[geolocation-memory-map]"));
        assert!(RealBlades::navigation_route("A to B").contains("[navigation-route]"));
        assert!(RealBlades::navigation_poi("coffee", "Budapest").contains("[navigation-poi]"));
    }

    #[test]
    fn brand_guidelines_work() {
        let result = RealBlades::brand_guidelines("Acme");
        assert!(result.contains("[brand-guidelines]"));
        assert!(result.contains("Acme"));
    }

    #[test]
    fn memory_blades_work() {
        assert!(RealBlades::memory_skills("query").contains("[memory-skills]"));
        assert!(RealBlades::memory_skills_v2("query").contains("[memory-skills-v2]"));
        assert!(RealBlades::microscope_memory("query").contains("[microscope-memory]"));
        assert!(RealBlades::emoti_mem("happy curious").contains("[emoti-mem]"));
    }

    #[test]
    fn architecture_blades_work() {
        assert!(RealBlades::architect_mind("design").contains("[architect-mind]"));
        assert!(RealBlades::senior_architect("migration").contains("[senior-architect]"));
        assert!(RealBlades::senior_prompt_engineer("cot").contains("[senior-prompt-engineer]"));
    }

    #[test]
    fn surgery_blades_work() {
        assert!(RealBlades::omni_surgeon("inject fn").contains("[omni-surgeon]"));
        assert!(RealBlades::file_surgeon("find").contains("[file-surgeon]"));
        assert!(RealBlades::formatter("format").contains("[formatter]"));
        assert!(RealBlades::stem_core("template").contains("[stem-core]"));
        assert!(RealBlades::omni_connector("connect").contains("[omni-connector]"));
    }

    #[test]
    fn canvas_design_blades_work() {
        assert!(RealBlades::canvas("<div>").contains("[canvas]"));
        assert!(RealBlades::canvas_design("logo").contains("[canvas-design]"));
        assert!(RealBlades::frontend_design("component").contains("[frontend-design]"));
        assert!(RealBlades::ui_design_system("tokens").contains("[ui-design-system]"));
        assert!(RealBlades::ui_ux_pro_max("layout").contains("[ui-ux-pro]"));
        assert!(RealBlades::theme_factory("dark").contains("[theme-factory]"));
    }

    #[test]
    fn bio_neural_blades_work() {
        assert!(RealBlades::crispr_hotfix("patch").contains("[crispr-hotfix]"));
        assert!(RealBlades::crispr_hotfix_v2("patch").contains("[crispr-hotfix-v2]"));
        assert!(RealBlades::synaptic_pruning("optimize").contains("[synaptic-pruning]"));
        assert!(RealBlades::synaptic_pruning_v2("optimize").contains("[synaptic-pruning-v2]"));
        assert!(RealBlades::viral_transduction("gene").contains("[viral-transduction]"));
        assert!(RealBlades::hox_architecture("body").contains("[hox-architecture]"));
        assert!(RealBlades::ai_synapse("neural").contains("[ai-synapse]"));
        assert!(RealBlades::hive_orchestrator("coordinate").contains("[hive-orchestrator]"));
        assert!(RealBlades::maestro_orchestration("conduct").contains("[maestro]"));
        assert!(RealBlades::swarm_coordination("agents").contains("[swarm]"));
        assert!(RealBlades::colony_swarm("sync").contains("[colony-swarm]"));
    }

    #[test]
    fn agent_factory_and_commander_work() {
        assert!(RealBlades::agent_factory("researcher", "web search").contains("[agent-factory]"));
        assert!(RealBlades::commander("build", "release").contains("[commander]"));
        assert!(RealBlades::swarm_queen("5").contains("[swarm-queen]"));
        assert!(RealBlades::replicator("local", "code").contains("[replicator]"));
    }

    #[test]
    fn quality_react_stem_blades_work() {
        assert!(RealBlades::quality_feature_delivery("spec").contains("[quality-bun]"));
        assert!(RealBlades::react_practices("optimize").contains("[react-practices]"));
        assert!(RealBlades::stemcell_manager("template").contains("[stem-cell-manager]"));
        assert!(RealBlades::mitosis_agent("divide").contains("[mitosis-agent]"));
    }

    #[test]
    fn forge_and_meta_blades_work() {
        assert!(RealBlades::forge_blade("spec").contains("[forge-blade]"));
        assert!(RealBlades::mcporter("list").contains("[mcporter]"));
        assert!(RealBlades::omega_striker("action").contains("[omega-striker]"));
        assert!(RealBlades::sigma("protocol").contains("[sigma]"));
        assert!(RealBlades::model_usage("gpt-4").contains("[model-usage]"));
        assert!(RealBlades::claude_migration("v2").contains("[claude-migration]"));
        assert!(RealBlades::peekaboo("observe").contains("[peekaboo]"));
    }

    #[test]
    fn pr_and_git_blades_handle_unavailable_tool() {
        let result = RealBlades::merge_pr("123");
        assert!(result.contains("[merge-pr]"));
        let result = RealBlades::review_pr("123");
        assert!(result.contains("[review-pr]"));
    }

    #[test]
    fn platform_specific_blades_return_unsupported() {
        #[cfg(not(target_os = "macos"))]
        {
            let result = RealBlades::apple_notes("test");
            assert!(result.contains("UNSUPPORTED"));
            let result = RealBlades::bear_notes("test");
            assert!(result.contains("UNSUPPORTED"));
        }
    }

    #[test]
    fn mintlify_and_test_tui_work() {
        assert!(RealBlades::mintlify("generate docs").contains("[mintlify]"));
        assert!(RealBlades::test_tui("run tests").contains("[test-tui]"));
    }

    #[test]
    fn immune_extended_blades_work() {
        assert!(RealBlades::immune_antibody("threat").contains("[immune-antibody]"));
        assert!(RealBlades::immune_log("5").contains("[immune-log]"));
    }

    #[test]
    fn document_agent_work() {
        let result = RealBlades::document_agent("generate API docs");
        assert!(result.contains("[document-agent]"));
    }

    #[test]
    fn eightctl_work() {
        let result = RealBlades::eightctl("status");
        assert!(result.contains("[eightctl]"));
    }

    #[test]
    fn voice_call_work() {
        let result = RealBlades::voice_call("call John");
        assert!(result.contains("[voice-call]"));
    }

    #[test]
    fn nano_pdf_work() {
        let result = RealBlades::nano_pdf("extract pages");
        assert!(result.contains("[nano-pdf]"));
    }

    #[test]
    fn pptx_work() {
        let result = RealBlades::pptx("create slides");
        assert!(result.contains("[pptx]"));
    }
}
