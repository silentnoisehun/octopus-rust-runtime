//! CLI command dispatch for Wave Echo

use super::llm::LlmPool;
use super::reflex::ReflexLoop;
use super::spine::{SpineBus, WaveEventType};
use super::store::WaveStore;
use super::template::WaveTemplate;
use super::wave::WaveEngine;
use crate::ExecutionOutcome;
use std::env;
use std::path::{Path, PathBuf};

fn default_wv3_path() -> PathBuf {
    env::var_os("OCTOPUS_WAVE_VAULT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/wave_vault.wv3"))
}

pub fn handle_wave_echo_command(args: &[String]) -> ExecutionOutcome {
    if args.is_empty() {
        return ExecutionOutcome::completed(render_help());
    }

    let sub = args[0].as_str();
    let rest = &args[1..];
    let store_path = default_wv3_path();

    match sub {
        "list" => cmd_list(&store_path),
        "vault-search" => {
            if rest.is_empty() {
                return ExecutionOutcome::failed(
                    "wave_echo_usage",
                    "usage: wave-echo vault-search <query>",
                );
            }
            cmd_vault_search(&store_path, &rest.join(" "))
        }
        "vault-get" => {
            if rest.is_empty() {
                return ExecutionOutcome::failed(
                    "wave_echo_usage",
                    "usage: wave-echo vault-get <id>",
                );
            }
            let id: usize = match rest[0].parse() {
                Ok(id) => id,
                Err(_) => {
                    return ExecutionOutcome::failed("wave_echo_usage", "invalid template id")
                }
            };
            cmd_vault_get(&store_path, id)
        }
        "activate" => {
            if rest.is_empty() {
                return ExecutionOutcome::failed(
                    "wave_echo_usage",
                    "usage: wave-echo activate <id> [--task <task>] [--exec]",
                );
            }
            let id: usize = match rest[0].parse() {
                Ok(id) => id,
                Err(_) => {
                    return ExecutionOutcome::failed("wave_echo_usage", "invalid template id")
                }
            };
            let (task, name, port, path, exec) = parse_activate_args(&rest[1..]);
            cmd_activate(&store_path, id, &task, &name, &port, &path, exec)
        }
        "activate-multi" => {
            if rest.is_empty() {
                return ExecutionOutcome::failed(
                    "wave_echo_usage",
                    "usage: wave-echo activate-multi <ids> [--task <task>] [--exec]",
                );
            }
            let ids_str = &rest[0];
            let ids: Vec<usize> = ids_str
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            let (task, name, port, path, exec) = parse_activate_args(&rest[1..]);
            cmd_activate_multi(&store_path, &ids, &task, &name, &port, &path, exec)
        }
        "project-init" => {
            if rest.is_empty() {
                return ExecutionOutcome::failed(
                    "wave_echo_usage",
                    "usage: wave-echo project-init <name> [--deps <deps>] [--port <port>]",
                );
            }
            let name = &rest[0];
            let (deps, port) = parse_project_init_args(&rest[1..]);
            cmd_project_init(&store_path, name, &deps, &port)
        }
        "add-native" => {
            if rest.is_empty() {
                return ExecutionOutcome::failed(
                    "wave_echo_usage",
                    "usage: wave-echo add-native <file.txt>",
                );
            }
            cmd_add_native(&store_path, &rest[0])
        }
        "hot" => {
            let count: usize = rest.first().and_then(|s| s.parse().ok()).unwrap_or(10);
            cmd_hot(&store_path, count)
        }
        "plan" => {
            if rest.is_empty() {
                return ExecutionOutcome::failed("wave_echo_usage", "usage: wave-echo plan <task>");
            }
            cmd_plan(&store_path, &rest.join(" "))
        }
        "feedback" => {
            if rest.is_empty() {
                return ExecutionOutcome::failed(
                    "wave_echo_usage",
                    "usage: wave-echo feedback <id> [--fail]",
                );
            }
            let id: usize = match rest[0].parse() {
                Ok(id) => id,
                Err(_) => {
                    return ExecutionOutcome::failed("wave_echo_usage", "invalid template id")
                }
            };
            let fail = rest.iter().any(|arg| arg == "--fail");
            cmd_feedback(&store_path, id, !fail)
        }
        "dream" => {
            let decay: f64 = rest.first().and_then(|s| s.parse().ok()).unwrap_or(0.97);
            cmd_dream(&store_path, decay)
        }
        "status" => cmd_status(),
        "pump" => cmd_pump(),
        "llm-gen" => {
            if rest.is_empty() {
                return ExecutionOutcome::failed(
                    "wave_echo_usage",
                    "usage: wave-echo llm-gen <prompt> [--arms <n>]",
                );
            }
            let prompt = rest[0].clone();
            let arms: usize = parse_flag_val(&rest[1..], "--arms")
                .and_then(|s| s.parse().ok())
                .unwrap_or(8);
            cmd_llm_gen(&store_path, &prompt, arms)
        }
        _ => ExecutionOutcome::failed(
            "wave_echo_unknown",
            format!("unknown wave-echo command: {sub}"),
        ),
    }
}

fn cmd_list(store_path: &Path) -> ExecutionOutcome {
    let mut templates = match WaveStore::load_wv3(store_path) {
        Ok(t) => t,
        Err(e) => return ExecutionOutcome::failed("wv3_load_failed", e),
    };

    templates.sort_by(|a, b| b.amplitude.partial_cmp(&a.amplitude).unwrap());

    let mut out = format!("WAVE VAULT [{} templates]\n", templates.len());
    for t in templates {
        out.push_str(&format!(
            "#{:02} [{:.2}] {} (keywords: {}, tech: {})\n",
            t.id,
            t.amplitude,
            t.label,
            t.keywords.join(", "),
            t.tech_stack.join(",")
        ));
    }
    ExecutionOutcome::completed(out)
}

fn cmd_vault_search(store_path: &Path, query: &str) -> ExecutionOutcome {
    let templates = match WaveStore::load_wv3(store_path) {
        Ok(t) => t,
        Err(e) => return ExecutionOutcome::failed("wv3_load_failed", e),
    };

    let candidates: Vec<_> = templates
        .iter()
        .map(|t| {
            (
                t.id,
                t.keywords.clone(),
                t.amplitude,
                t.tech_stack.clone(),
                Vec::new(),
            )
        })
        .collect();

    let mut matches = Vec::new();
    for &(id, ref kw, amp, ref tech, ref pat) in &candidates {
        let score = WaveEngine::calculate_interference(query, kw, amp, tech, pat);
        if score > 0.10 {
            matches.push((id, score));
        }
    }

    matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut out = format!("WAVE SEARCH [query: \"{query}\"]\n");
    for (id, score) in matches {
        if let Some(t) = templates.iter().find(|t| t.id == id) {
            let resonant = if score >= 0.72 { " [RESONANT]" } else { "" };
            out.push_str(&format!(
                "#{:02} score={:.4}{} | {} (amp={:.2})\n",
                t.id, score, resonant, t.label, t.amplitude
            ));
        }
    }
    ExecutionOutcome::completed(out)
}

fn cmd_vault_get(store_path: &Path, id: usize) -> ExecutionOutcome {
    let templates = match WaveStore::load_wv3(store_path) {
        Ok(t) => t,
        Err(e) => return ExecutionOutcome::failed("wv3_load_failed", e),
    };

    if let Some(t) = templates.iter().find(|t| t.id == id) {
        ExecutionOutcome::completed(format!(
            "VAULT GET #{:02} [{}]\nlabel: {}\namplitude: {:.4}\nkeywords: {}\ntech: {}\n\n{}",
            t.id,
            t.label,
            t.label,
            t.amplitude,
            t.keywords.join(", "),
            t.tech_stack.join(", "),
            t.code
        ))
    } else {
        ExecutionOutcome::failed("template_not_found", format!("template #{id} not found"))
    }
}

fn cmd_activate(
    store_path: &Path,
    id: usize,
    task: &str,
    name: &str,
    port: &str,
    path: &str,
    exec: bool,
) -> ExecutionOutcome {
    let templates = match WaveStore::load_wv3(store_path) {
        Ok(t) => t,
        Err(e) => return ExecutionOutcome::failed("wv3_load_failed", e),
    };

    if let Some(t) = templates.iter().find(|t| t.id == id) {
        ReflexLoop::activate_template(t, task, name, port, path, exec)
    } else {
        ExecutionOutcome::failed("template_not_found", format!("template #{id} not found"))
    }
}

fn cmd_activate_multi(
    store_path: &Path,
    ids: &[usize],
    task: &str,
    name: &str,
    port: &str,
    path: &str,
    exec: bool,
) -> ExecutionOutcome {
    let templates = match WaveStore::load_wv3(store_path) {
        Ok(t) => t,
        Err(e) => return ExecutionOutcome::failed("wv3_load_failed", e),
    };

    let selected: Vec<_> = templates
        .into_iter()
        .filter(|t| ids.contains(&t.id))
        .collect();
    if selected.is_empty() {
        return ExecutionOutcome::failed(
            "templates_not_found",
            "none of the requested template IDs were found",
        );
    }

    ReflexLoop::activate_multi(&selected, task, name, port, path, exec)
}

fn cmd_project_init(store_path: &Path, name: &str, deps: &str, port: &str) -> ExecutionOutcome {
    let templates = match WaveStore::load_wv3(store_path) {
        Ok(t) => t,
        Err(_) => vec![
            WaveTemplate::new(
                1,
                "Cargo.toml",
                vec![],
                "WRITE||Cargo.toml@@@[package]\nname = \"{{name}}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
                vec![],
                0.95,
            ),
            WaveTemplate::new(
                2,
                "src/main.rs",
                vec![],
                "WRITE||src/main.rs@@@fn main() {\n    println!(\"Running {{name}} on port {{port}}\");\n}\n",
                vec![],
                0.95,
            ),
        ],
    };

    let task = format!("init project {name} with deps {deps}");
    ReflexLoop::activate_multi(&templates, &task, name, port, "src/main.rs", true)
}

fn cmd_add_native(store_path: &Path, txt_file: &str) -> ExecutionOutcome {
    match WaveStore::add_native_file(store_path, txt_file) {
        Ok(added) => ExecutionOutcome::completed(format!(
            "ADD NATIVE: added {added} new templates to {}",
            store_path.display()
        )),
        Err(e) => ExecutionOutcome::failed("add_native_failed", e),
    }
}

fn cmd_hot(store_path: &Path, count: usize) -> ExecutionOutcome {
    let mut templates = match WaveStore::load_wv3(store_path) {
        Ok(t) => t,
        Err(e) => return ExecutionOutcome::failed("wv3_load_failed", e),
    };

    templates.sort_by(|a, b| b.amplitude.partial_cmp(&a.amplitude).unwrap());
    let top: Vec<_> = templates.into_iter().take(count).collect();

    let mut out = format!("HOT ARMS [top {count}]\n");
    for t in top {
        out.push_str(&format!("#{:02} [{:.4}] {}\n", t.id, t.amplitude, t.label));
    }
    ExecutionOutcome::completed(out)
}

fn cmd_plan(store_path: &Path, task: &str) -> ExecutionOutcome {
    let templates = match WaveStore::load_wv3(store_path) {
        Ok(t) => t,
        Err(e) => return ExecutionOutcome::failed("wv3_load_failed", e),
    };

    let candidates: Vec<_> = templates
        .iter()
        .map(|t| {
            (
                t.id,
                t.keywords.clone(),
                t.amplitude,
                t.tech_stack.clone(),
                Vec::new(),
            )
        })
        .collect();

    if let Some(m) = WaveEngine::find_best_match(task, &candidates) {
        if let Some(t) = templates.iter().find(|t| t.id == m.template_id) {
            return ExecutionOutcome::completed(format!(
                "WAVE PLAN [resonant score={:.4}]\nrecommended_arm: #{:02} {}\ncode_preview:\n{}",
                m.score, t.id, t.label, t.code
            ));
        }
    }

    ExecutionOutcome::completed(format!(
        "WAVE PLAN: no resonant template found for \"{task}\" (score < 0.72)"
    ))
}

fn cmd_feedback(store_path: &Path, id: usize, success: bool) -> ExecutionOutcome {
    match ReflexLoop::apply_feedback(id, success, store_path) {
        Ok(msg) => ExecutionOutcome::completed(msg),
        Err(e) => ExecutionOutcome::failed("feedback_failed", e),
    }
}

fn cmd_dream(store_path: &Path, decay: f64) -> ExecutionOutcome {
    let mut templates = match WaveStore::load_wv3(store_path) {
        Ok(t) => t,
        Err(e) => return ExecutionOutcome::failed("wv3_load_failed", e),
    };

    for t in &mut templates {
        t.apply_decay(decay);
    }

    if let Ok(spine) = SpineBus::default_bus() {
        spine.push_event(WaveEventType::Dream, decay as f32, "dream_decay_pass");
    }

    if let Err(e) = WaveStore::save_wv3(store_path, &templates) {
        return ExecutionOutcome::failed("dream_save_failed", e);
    }

    ExecutionOutcome::completed(format!(
        "WAVE DREAM: applied decay factor {decay:.4} across {} templates",
        templates.len()
    ))
}

fn cmd_status() -> ExecutionOutcome {
    if let Ok(spine) = SpineBus::default_bus() {
        let events = spine.read_events();
        let psi_0 = spine.read_psi_slot(0);
        ExecutionOutcome::completed(format!(
            "WAVE ECHO STATUS\nspine_path: {}\npsi_slot_0: {:.4}\nring_events: {}\n",
            spine.path.display(),
            psi_0,
            events.len()
        ))
    } else {
        ExecutionOutcome::failed("spine_unavailable", "cannot open spine mmap bus")
    }
}

fn cmd_pump() -> ExecutionOutcome {
    if let Ok(spine) = SpineBus::default_bus() {
        let events = spine.read_events();
        spine.push_event(WaveEventType::Pump, 1.0, "pump_cycle");
        ExecutionOutcome::completed(format!(
            "WAVE PUMP: processed {} events on ring buffer",
            events.len()
        ))
    } else {
        ExecutionOutcome::failed("spine_unavailable", "cannot open spine mmap bus")
    }
}

fn cmd_llm_gen(store_path: &Path, prompt: &str, arms: usize) -> ExecutionOutcome {
    let candidates = LlmPool::generate_candidates(prompt, arms);
    let mut templates = WaveStore::load_wv3(store_path).unwrap_or_default();
    let start_id = templates.len() + 1;

    let mut added = Vec::new();
    for (i, mut c) in candidates.into_iter().enumerate() {
        c.id = start_id + i;
        added.push(c.clone());
        templates.push(c);
    }

    if let Err(e) = WaveStore::save_wv3(store_path, &templates) {
        return ExecutionOutcome::failed("llm_gen_save_failed", e);
    }

    let mut out = format!("LLM-GEN [prompt: \"{prompt}\" | arms: {arms}]\n");
    for a in added {
        out.push_str(&format!("+#{:02} [{}]\n", a.id, a.label));
    }
    ExecutionOutcome::completed(out)
}

fn parse_activate_args(args: &[String]) -> (String, String, String, String, bool) {
    let task = parse_flag_val(args, "--task").unwrap_or_default();
    let name = parse_flag_val(args, "--name").unwrap_or_else(|| "mytool".to_string());
    let port = parse_flag_val(args, "--port").unwrap_or_else(|| "8080".to_string());
    let path = parse_flag_val(args, "--path").unwrap_or_else(|| "src/main.rs".to_string());
    let exec = args.iter().any(|arg| arg == "--exec");
    (task, name, port, path, exec)
}

fn parse_project_init_args(args: &[String]) -> (String, String) {
    let deps = parse_flag_val(args, "--deps").unwrap_or_default();
    let port = parse_flag_val(args, "--port").unwrap_or_else(|| "8080".to_string());
    (deps, port)
}

fn parse_flag_val(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == flag && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

fn render_help() -> String {
    "WAVE ECHO - Hullám-sablon kar-könyvtár és kódgenerátor\n\n\
     Subcommands:\n\
       list                             List all vault templates\n\
       vault-search <query>             GP-echo interference search\n\
       vault-get <id>                   View template code\n\
       activate <id> [--exec]           Activate template\n\
       activate-multi <ids> [--exec]    Activate multiple templates concurrently\n\
       project-init <name>              Initialize project structure\n\
       add-native <file.txt>            Add native text templates\n\
       hot [count]                      Top templates by amplitude\n\
       plan <task>                      Recommended template plan\n\
       feedback <id> [--fail]           LTP / LTD learning feedback\n\
       dream [decay]                    Amplitude decay pass\n\
       status                           Spine status report\n\
       pump                             Pump ring buffer events\n\
       llm-gen <prompt> [--arms N]      Generate templates via free model pool\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_wave_echo_cmd_list_and_search() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let t = WaveTemplate::new(
            1,
            "rust-http-server",
            vec!["rust".to_string(), "http".to_string(), "axum".to_string()],
            "OCTO||summarize || code-analysis@@@rust http server axum tokio",
            vec!["axum".to_string()],
            0.90,
        );

        WaveStore::save_wv3(path, &[t]).unwrap();

        let res = cmd_vault_search(path, "rust http axum");
        assert!(!res.is_failed());
        assert!(res.output.contains("rust-http-server"));
    }
}
