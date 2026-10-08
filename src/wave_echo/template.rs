//! Wave template and LTP/LTD learning methods

#[derive(Debug, Clone)]
pub enum ArmType {
    Octopus { spec: String, prompt: String },
    Write { file: String, content: String },
    Generic { code: String },
}

#[derive(Debug, Clone)]
pub struct WaveTemplate {
    pub id: usize,
    pub label: String,
    pub keywords: Vec<String>,
    pub code: String,
    pub tech_stack: Vec<String>,
    pub amplitude: f64,
}

impl WaveTemplate {
    pub fn new(
        id: usize,
        label: impl Into<String>,
        keywords: Vec<String>,
        code: impl Into<String>,
        tech_stack: Vec<String>,
        amplitude: f64,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            keywords,
            code: code.into(),
            tech_stack,
            amplitude: amplitude.clamp(0.1, 1.0),
        }
    }

    /// Long-Term Potentiation (+0.02)
    pub fn apply_ltp(&mut self) {
        self.amplitude = (self.amplitude + 0.02).min(1.0);
    }

    /// Long-Term Depression (x 0.95)
    pub fn apply_ltd(&mut self) {
        self.amplitude = (self.amplitude * 0.95).max(0.1);
    }

    /// Decay
    pub fn apply_decay(&mut self, factor: f64) {
        self.amplitude = (self.amplitude * factor).clamp(0.1, 1.0);
    }

    /// Parse arm execution type
    pub fn parse_arm_type(&self, task: &str, name: &str, port: &str, path: &str) -> ArmType {
        let rendered = self.render(task, name, port, path);
        if let Some(rest) = rendered.strip_prefix("OCTO||") {
            if let Some((spec, prompt)) = rest.split_once("@@@") {
                return ArmType::Octopus {
                    spec: spec.trim().to_string(),
                    prompt: prompt.trim().to_string(),
                };
            }
        } else if let Some(rest) = rendered.strip_prefix("WRITE||") {
            if let Some((file, content)) = rest.split_once("@@@") {
                return ArmType::Write {
                    file: file.trim().to_string(),
                    content: content.to_string(),
                };
            }
        }
        ArmType::Generic { code: rendered }
    }

    /// Substitute placeholders: {{task}}, {{name}}, {{port}}, {{path}}
    pub fn render(&self, task: &str, name: &str, port: &str, path: &str) -> String {
        let mut rendered = self.code.replace("\\n", "\n");
        rendered = rendered.replace("{{task}}", task);
        rendered = rendered.replace("{{name}}", name);
        rendered = rendered.replace("{{port}}", port);
        rendered = rendered.replace("{{path}}", path);
        rendered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_ltp_ltd() {
        let mut template = WaveTemplate::new(
            1,
            "test",
            vec!["rust".to_string()],
            "fn main() {}",
            vec!["rust".to_string()],
            0.80,
        );

        template.apply_ltp();
        assert!((template.amplitude - 0.82).abs() < 1e-6);

        template.apply_ltd();
        assert!((template.amplitude - 0.779).abs() < 1e-6);
    }

    #[test]
    fn test_arm_type_parsing() {
        let octo = WaveTemplate::new(
            1,
            "octo",
            vec![],
            "OCTO||summarize || code-analysis@@@{{task}}",
            vec![],
            0.9,
        );

        match octo.parse_arm_type("my task", "mytool", "8080", "src/main.rs") {
            ArmType::Octopus { spec, prompt } => {
                assert_eq!(spec, "summarize || code-analysis");
                assert_eq!(prompt, "my task");
            }
            _ => panic!("expected Octopus arm"),
        }

        let write = WaveTemplate::new(
            2,
            "write",
            vec![],
            "WRITE||{{path}}@@@fn main() { println!(\"{{name}}\"); }",
            vec![],
            0.9,
        );

        match write.parse_arm_type("task", "mytool", "8080", "src/main.rs") {
            ArmType::Write { file, content } => {
                assert_eq!(file, "src/main.rs");
                assert!(content.contains("mytool"));
            }
            _ => panic!("expected Write arm"),
        }
    }
}
