//! Reflex loop execution and feedback

use super::spine::{SpineBus, WaveEventType};
use super::store::WaveStore;
use super::template::{ArmType, WaveTemplate};
use crate::ExecutionOutcome;
use std::fs;
use std::path::Path;
use std::thread;

pub struct ReflexLoop;

impl ReflexLoop {
    pub fn activate_template(
        template: &WaveTemplate,
        task: &str,
        name: &str,
        port: &str,
        path: &str,
        exec: bool,
    ) -> ExecutionOutcome {
        let arm = template.parse_arm_type(task, name, port, path);

        // Push event to SpineBus
        if let Ok(spine) = SpineBus::default_bus() {
            spine.push_event(
                WaveEventType::Gen,
                template.amplitude as f32,
                &format!("activate:{}:{}", template.id, template.label),
            );
            spine.write_psi_slot(0, template.amplitude);
        }

        match arm {
            ArmType::Octopus { spec, prompt } => {
                if exec {
                    crate::run_pipeline_outcome(&spec, &prompt)
                } else {
                    ExecutionOutcome::completed(format!(
                        "REFLEX PLAN [Octopus]: spec={spec}\n\n{prompt}"
                    ))
                }
            }
            ArmType::Write { file, content } => {
                if exec {
                    if let Some(parent) = Path::new(&file).parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    match fs::write(&file, &content) {
                        Ok(()) => ExecutionOutcome::completed(format!(
                            "REFLEX ACTIVATED [Write]: file={file} ({} bytes)",
                            content.len()
                        )),
                        Err(e) => ExecutionOutcome::failed(
                            "reflex_write_failed",
                            format!("cannot write {file}: {e}"),
                        ),
                    }
                } else {
                    ExecutionOutcome::completed(format!(
                        "REFLEX PLAN [Write]: file={file}\n\n{content}"
                    ))
                }
            }
            ArmType::Generic { code } => ExecutionOutcome::completed(format!(
                "REFLEX CODE [{}]: amp={:.2}\n\n{code}",
                template.label, template.amplitude
            )),
        }
    }

    pub fn activate_multi(
        templates: &[WaveTemplate],
        task: &str,
        name: &str,
        port: &str,
        path: &str,
        exec: bool,
    ) -> ExecutionOutcome {
        let mut handles = Vec::new();

        for t in templates {
            let template = t.clone();
            let task = task.to_string();
            let name = name.to_string();
            let port = port.to_string();
            let path = path.to_string();

            handles.push(thread::spawn(move || {
                Self::activate_template(&template, &task, &name, &port, &path, exec)
            }));
        }

        let mut outputs = Vec::new();
        let mut failed = false;

        for h in handles {
            let res = h.join().unwrap_or_else(|_| {
                ExecutionOutcome::failed("thread_panic", "reflex thread panicked")
            });
            if res.is_failed() {
                failed = true;
            }
            outputs.push(res.output);
        }

        let summary = outputs.join("\n\n---\n\n");
        if failed {
            ExecutionOutcome::failed("reflex_multi_failed", summary)
        } else {
            ExecutionOutcome::completed(summary)
        }
    }

    pub fn apply_feedback(
        template_id: usize,
        success: bool,
        store_path: impl AsRef<Path>,
    ) -> Result<String, String> {
        let store_path = store_path.as_ref();
        let mut templates = WaveStore::load_wv3(store_path)?;

        let template = templates
            .iter_mut()
            .find(|t| t.id == template_id)
            .ok_or_else(|| format!("template ID {template_id} not found"))?;

        if success {
            template.apply_ltp();
        } else {
            template.apply_ltd();
        }

        let label = template.label.clone();
        let new_amp = template.amplitude;

        if let Ok(spine) = SpineBus::default_bus() {
            let event_type = if success {
                WaveEventType::Ltp
            } else {
                WaveEventType::Ltd
            };
            spine.push_event(
                event_type,
                new_amp as f32,
                &format!("feedback:{template_id}:{label}"),
            );
            spine.write_psi_slot(0, new_amp);
        }

        WaveStore::save_wv3(store_path, &templates)?;

        Ok(format!(
            "FEEDBACK APPLIED [{}]: id={} new_amplitude={:.4}",
            if success {
                "LTP (+0.02)"
            } else {
                "LTD (x0.95)"
            },
            template_id,
            new_amp
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_activate_template_and_feedback() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let t = WaveTemplate::new(
            1,
            "test-write",
            vec!["rust".to_string()],
            "WRITE||{{path}}@@@fn main() {}",
            vec!["rust".to_string()],
            0.80,
        );

        WaveStore::save_wv3(path, std::slice::from_ref(&t)).unwrap();

        let target = tempfile::NamedTempFile::new().unwrap();
        let target_path = target.path().to_string_lossy().to_string();

        let res = ReflexLoop::activate_template(&t, "task", "app", "8080", &target_path, true);
        assert!(!res.is_failed());
        assert_eq!(fs::read_to_string(&target_path).unwrap(), "fn main() {}");

        let fb = ReflexLoop::apply_feedback(1, true, path).unwrap();
        assert!(fb.contains("LTP"));
    }
}
