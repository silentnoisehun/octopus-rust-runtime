use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputType {
    FilePath,
    Text,
    Hash,
    Command,
    Json,
    Any,
}

impl fmt::Display for InputType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::FilePath => "file_path",
            Self::Text => "text",
            Self::Hash => "hash",
            Self::Command => "command",
            Self::Json => "json",
            Self::Any => "any",
        };
        formatter.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputType {
    Text,
    Json,
    Structured,
    Any,
}

impl fmt::Display for OutputType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Text => "text",
            Self::Json => "json",
            Self::Structured => "structured",
            Self::Any => "any",
        };
        formatter.write_str(value)
    }
}

#[derive(Debug, Clone)]
pub struct InputContract {
    pub field: &'static str,
    pub input_type: InputType,
    pub required: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct OutputContract {
    pub output_type: OutputType,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct CapabilityContract {
    pub version: &'static str,
    pub group: &'static str,
    pub input: Vec<InputContract>,
    pub output: OutputContract,
    pub deprecated: bool,
    pub deprecation_message: Option<&'static str>,
}

impl CapabilityContract {
    pub fn validate_input(&self, prompt: &str) -> Result<(), String> {
        if self.input.is_empty() {
            return Ok(());
        }

        let required_fields: Vec<_> = self.input.iter().filter(|c| c.required).collect();
        if required_fields.is_empty() {
            return Ok(());
        }

        for field_contract in &required_fields {
            match field_contract.input_type {
                InputType::FilePath => {
                    if prompt.trim().is_empty() {
                        return Err(format!(
                            "Missing required field '{}': {}",
                            field_contract.field, field_contract.description
                        ));
                    }
                    let path = prompt.trim();
                    if path.contains(['\r', '\n']) {
                        return Err(format!(
                            "Field '{}' cannot contain newlines",
                            field_contract.field
                        ));
                    }
                }
                InputType::Text => {
                    if prompt.trim().is_empty() {
                        return Err(format!(
                            "Missing required field '{}': {}",
                            field_contract.field, field_contract.description
                        ));
                    }
                }
                InputType::Hash => {
                    let parts: Vec<&str> = prompt.splitn(3, '|').collect();
                    let hash = parts.get(1).unwrap_or(&"").trim();
                    if hash.is_empty() || (!hash.eq_ignore_ascii_case("NEW") && hash.len() != 64) {
                        return Err(format!(
                            "Field '{}' must be a 64-character SHA256 hash or 'NEW'",
                            field_contract.field
                        ));
                    }
                }
                InputType::Command => {
                    if prompt.trim().is_empty() {
                        return Err(format!(
                            "Missing required command in '{}': {}",
                            field_contract.field, field_contract.description
                        ));
                    }
                }
                InputType::Json => {
                    let trimmed = prompt.trim();
                    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
                        return Err(format!(
                            "Field '{}' must be valid JSON object",
                            field_contract.field
                        ));
                    }
                }
                InputType::Any => {}
            }
        }

        Ok(())
    }

    pub fn render(&self) -> String {
        let mut parts = vec![
            format!("version:{}", self.version),
            format!("group:{}", self.group),
        ];

        if self.deprecated {
            parts.push("deprecated:true".to_string());
            if let Some(msg) = self.deprecation_message {
                parts.push(format!("deprecation_msg:{msg}"));
            }
        }

        for input in &self.input {
            let required = if input.required {
                "required"
            } else {
                "optional"
            };
            parts.push(format!(
                "input:{}:{}:{}:{}",
                input.field, input.input_type, required, input.description
            ));
        }

        parts.push(format!(
            "output:{}:{}",
            self.output.output_type, self.output.description
        ));

        parts.join("\t")
    }
}

pub fn get_contract(name: &str) -> Option<CapabilityContract> {
    match name {
        "code-reader" => Some(CapabilityContract {
            version: "1.2",
            group: "local",
            input: vec![InputContract {
                field: "path",
                input_type: InputType::FilePath,
                required: true,
                description: "File path to read (no newlines)",
            }],
            output: OutputContract {
                output_type: OutputType::Structured,
                description: "LOCAL READ with path, bytes, analysis, content",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "code-writer" => Some(CapabilityContract {
            version: "1.2",
            group: "local",
            input: vec![
                InputContract {
                    field: "path",
                    input_type: InputType::FilePath,
                    required: true,
                    description: "Target file path",
                },
                InputContract {
                    field: "expected_hash",
                    input_type: InputType::Hash,
                    required: true,
                    description: "Expected SHA256 hash or NEW",
                },
                InputContract {
                    field: "content",
                    input_type: InputType::Text,
                    required: true,
                    description: "File content to write",
                },
            ],
            output: OutputContract {
                output_type: OutputType::Structured,
                description: "LOCAL WRITE with path, bytes, sha256, backup",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "diagnostics" => Some(CapabilityContract {
            version: "1.2",
            group: "local",
            input: vec![InputContract {
                field: "path",
                input_type: InputType::FilePath,
                required: false,
                description: "File path to analyze (no newlines)",
            }],
            output: OutputContract {
                output_type: OutputType::Structured,
                description: "LOCAL READ with path, bytes, diagnostics analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "git-nexus" => Some(CapabilityContract {
            version: "1.2",
            group: "local",
            input: vec![InputContract {
                field: "path",
                input_type: InputType::FilePath,
                required: false,
                description: "Repository path (default: current directory)",
            }],
            output: OutputContract {
                output_type: OutputType::Structured,
                description: "Git status with branch and short status",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "github" => Some(CapabilityContract {
            version: "1.5",
            group: "external",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: false,
                description:
                    "repo-view <owner/repo> | pr-list <owner/repo> | issue-list <owner/repo>",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "GitHub CLI output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "github-manager" => Some(CapabilityContract {
            version: "1.5",
            group: "external",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: false,
                description: "repo-list | pr-list | issue-list | run-list",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "GitHub CLI output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "pipeline-architect" => Some(CapabilityContract {
            version: "1.2",
            group: "composite",
            input: vec![InputContract {
                field: "prompt",
                input_type: InputType::Text,
                required: true,
                description: "Boundary contract specification",
            }],
            output: OutputContract {
                output_type: OutputType::Structured,
                description: "Boundary contract description",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "rust-surgeon" => Some(CapabilityContract {
            version: "1.2",
            group: "composite",
            input: vec![InputContract {
                field: "prompt",
                input_type: InputType::Text,
                required: true,
                description: "Surgical replacement specification",
            }],
            output: OutputContract {
                output_type: OutputType::Structured,
                description: "Surgical replacement output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "summarize" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "text",
                input_type: InputType::Text,
                required: false,
                description: "Text to summarize",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Extractive summary",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "sag" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "query ||| text format",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Search results with occurrence counts",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "code-analysis" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to analyze",
            }],
            output: OutputContract {
                output_type: OutputType::Structured,
                description: "Code metrics (lines, functions, structs, etc.)",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "polyglot" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to detect language",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Detected language with confidence",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "circuit-breaker" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "state",
                input_type: InputType::Text,
                required: true,
                description: "closed|open|half-open",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Circuit breaker state",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "code-review" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to review",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Code review findings",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "geolocation-distance" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "coords",
                input_type: InputType::Text,
                required: true,
                description: "lat1 lon1 lat2 lon2",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Distance in kilometers",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-extract" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to extract DNA from",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Extracted functions, structs, traits",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dual-generate" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "prompt",
                input_type: InputType::Text,
                required: true,
                description: "Generation prompt",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Generated code in two languages",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "duplicate-detector" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to check for duplicates",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Duplicate line detection",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "code-quality" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to analyze quality",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Quality score and metrics",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "data-master" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "data",
                input_type: InputType::Text,
                required: true,
                description: "Space-separated numbers",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Statistical summary",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "retry-policy" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "config",
                input_type: InputType::Text,
                required: false,
                description: "max_retries delay_ms",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Retry policy configuration",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "graceful-shutdown" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![InputContract {
                field: "timeout",
                input_type: InputType::Text,
                required: false,
                description: "Timeout in milliseconds",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Shutdown configuration",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "immune-status" => Some(CapabilityContract {
            version: "2.0",
            group: "algorithm",
            input: vec![],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "System immune status",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "video-frames" => Some(CapabilityContract {
            version: "2.1",
            group: "process",
            input: vec![InputContract {
                field: "video",
                input_type: InputType::FilePath,
                required: true,
                description: "Video file path and optional output dir",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Frame extraction status",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "bench-meter" => Some(CapabilityContract {
            version: "2.1",
            group: "algorithm",
            input: vec![InputContract {
                field: "iterations",
                input_type: InputType::Text,
                required: false,
                description: "Number of iterations (default: 1000)",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Benchmark results",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "tmux" => Some(CapabilityContract {
            version: "2.1",
            group: "process",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: false,
                description: "list-sessions|new-session|kill-session|send-keys",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Tmux command output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "weather" => Some(CapabilityContract {
            version: "2.1",
            group: "external",
            input: vec![InputContract {
                field: "city",
                input_type: InputType::Text,
                required: true,
                description: "City name",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Weather information",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "openai-image-gen" => Some(CapabilityContract {
            version: "2.2",
            group: "external",
            input: vec![InputContract {
                field: "prompt",
                input_type: InputType::Text,
                required: true,
                description: "Image description",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Generated image URL",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "openai-whisper" => Some(CapabilityContract {
            version: "2.2",
            group: "external",
            input: vec![InputContract {
                field: "audio",
                input_type: InputType::FilePath,
                required: true,
                description: "Audio file path",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Transcription text",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "notion" => Some(CapabilityContract {
            version: "2.2",
            group: "external",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Search query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Notion search results",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "discord" => Some(CapabilityContract {
            version: "2.2",
            group: "external",
            input: vec![InputContract {
                field: "message",
                input_type: InputType::Text,
                required: true,
                description: "Message to send",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Message delivery status",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "himalaya" => Some(CapabilityContract {
            version: "2.2",
            group: "external",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: true,
                description: "inbox|send <to> <subject> <body>",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Email operation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "gog" => Some(CapabilityContract {
            version: "2.2",
            group: "external",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: true,
                description: "gmail|calendar|drive",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Google service result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "brainstorming" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "topic",
                input_type: InputType::Text,
                required: true,
                description: "Topic to brainstorm",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Structured brainstorming output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "prose" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "text",
                input_type: InputType::Text,
                required: true,
                description: "Text to analyze",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Prose quality analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "writing-rules" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "style",
                input_type: InputType::Text,
                required: true,
                description: "technical|creative|formal|casual",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Writing rules for style",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "doc-scribe" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code or text to document",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Generated documentation",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "agent-development" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "type",
                input_type: InputType::Text,
                required: true,
                description: "researcher|coder|reviewer",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Agent template",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "hook-development" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "type",
                input_type: InputType::Text,
                required: true,
                description: "pre-commit|post-commit|pre-push",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Hook template",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "command-development" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "name",
                input_type: InputType::Text,
                required: true,
                description: "Command name",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Command template",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "plugin-structure" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "name",
                input_type: InputType::Text,
                required: true,
                description: "Plugin name",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Plugin directory structure",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "testing-codegen" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to generate tests for",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Generated test templates",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "brand-voice" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "brand",
                input_type: InputType::Text,
                required: true,
                description: "Brand name",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Brand voice analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "brand-writer" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "content",
                input_type: InputType::Text,
                required: true,
                description: "Content to rewrite",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Brand-optimized content",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "planner" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "goal",
                input_type: InputType::Text,
                required: true,
                description: "Goal to plan for",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Structured plan",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "memory-bank" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "topic",
                input_type: InputType::Text,
                required: true,
                description: "Topic to store",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Memory structure",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "still-archive" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Search query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Archive search results",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "incubator" => Some(CapabilityContract {
            version: "2.3",
            group: "meta",
            input: vec![InputContract {
                field: "module",
                input_type: InputType::Text,
                required: true,
                description: "Module name",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Incubation status",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        // Phase 5 contracts
        "web-research" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Search query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Search results",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "audio-diagnostics" => Some(CapabilityContract {
            version: "2.4",
            group: "process",
            input: vec![InputContract {
                field: "path",
                input_type: InputType::FilePath,
                required: true,
                description: "Audio file path",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Audio diagnostics",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "sherpa-onnx-tts" => Some(CapabilityContract {
            version: "2.4",
            group: "process",
            input: vec![InputContract {
                field: "text",
                input_type: InputType::Text,
                required: true,
                description: "Text to speak",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "TTS output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "tts-voice" => Some(CapabilityContract {
            version: "2.4",
            group: "process",
            input: vec![InputContract {
                field: "text",
                input_type: InputType::Text,
                required: true,
                description: "Text to speak",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Voice output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "stt-ear" => Some(CapabilityContract {
            version: "2.4",
            group: "process",
            input: vec![InputContract {
                field: "audio",
                input_type: InputType::FilePath,
                required: true,
                description: "Audio file path",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Transcription",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "mermaid-agent" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "diagram",
                input_type: InputType::Text,
                required: true,
                description: "Mermaid diagram code",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Diagram render info",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "1password" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: true,
                description: "item-get|item-list",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "1Password result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "canvas" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "html",
                input_type: InputType::Text,
                required: true,
                description: "HTML content",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Canvas render status",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "canvas-design" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Design specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Design output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "frontend-design" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Component specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Frontend component",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "ui-design-system" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Design system spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Design system output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "ui-ux-pro" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "UI/UX specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "UI/UX output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "theme-factory" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Theme specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Theme output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "brand-guidelines" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "brand",
                input_type: InputType::Text,
                required: true,
                description: "Brand name",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Brand guidelines",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "document-agent" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "task",
                input_type: InputType::Text,
                required: true,
                description: "Documentation task",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Documentation output",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "memory-skills" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Memory query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Memory search results",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "memory-skills-v2" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Memory query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Enhanced memory results",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "microscope-memory" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Memory query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Microscope memory results",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "emoti-mem" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "text",
                input_type: InputType::Text,
                required: true,
                description: "Text for emotional analysis",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "21D emotion vector",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "architect-mind" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Architecture question",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Architecture analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "senior-architect" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "task",
                input_type: InputType::Text,
                required: true,
                description: "Architecture task",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Architecture analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "senior-prompt-engineer" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "task",
                input_type: InputType::Text,
                required: true,
                description: "Prompt engineering task",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Prompt optimization",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "omni-surgeon" => Some(CapabilityContract {
            version: "2.4",
            group: "composite",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Surgery specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Surgery result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "file-surgeon" => Some(CapabilityContract {
            version: "2.4",
            group: "composite",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "File operation spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "File operation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "formatter" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Format specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Format result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "stem-core" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Template specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Code generation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "omni-connector" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Connection specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Connection result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "parser" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to parse",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Parse result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "type-inference" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to analyze",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Type inference result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "lint-rules" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to lint",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Lint results",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "crispr-hotfix" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Hotfix specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Hotfix result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "crispr-hotfix-v2" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Hotfix specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Hotfix result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "synaptic-pruning" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Optimization spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Pruning result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "synaptic-pruning-v2" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Optimization spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Pruning result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "viral-transduction" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Gene therapy spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Transduction result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "hox-architecture" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Architecture spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Architecture result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "ai-synapse" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Neural connection spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Synapse result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "hive-orchestrator" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Orchestration spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Orchestration result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "maestro" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Orchestration spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Orchestration result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "swarm" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Swarm spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Swarm result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "colony-swarm" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Colony spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Colony result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "quality-bun" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Delivery spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Delivery result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "react-practices" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "React optimization spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Optimization result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "stem-cell-manager" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Template spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Differentiation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "mitosis-agent" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Mitosis spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Mitosis result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "blogwatcher" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "url",
                input_type: InputType::Text,
                required: true,
                description: "Blog URL to monitor",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Blog status",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "peekaboo" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Observation spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Observation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "merge-pr" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "pr",
                input_type: InputType::Text,
                required: true,
                description: "PR number or URL",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Merge result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "merge-pr-v1" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "pr",
                input_type: InputType::Text,
                required: true,
                description: "PR number or URL",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Merge result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "review-pr" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "pr",
                input_type: InputType::Text,
                required: true,
                description: "PR number or URL",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Review result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "eightctl" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: true,
                description: "Control command",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Control result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "clawhub" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: true,
                description: "search|install|list",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "ClawHub result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "wacli" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: true,
                description: "WhatsApp CLI command",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "WhatsApp result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "goplaces" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Place search query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Places results",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "local-places" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Local place query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Local places results",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "web-extractor" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "url",
                input_type: InputType::Text,
                required: true,
                description: "URL to extract",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Extracted content",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "lobster-scraper" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "url",
                input_type: InputType::Text,
                required: true,
                description: "URL to scrape",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Scraped content",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "nano-pdf" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "PDF operation spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "PDF operation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "pptx" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Presentation operation",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Presentation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "turborepo" => Some(CapabilityContract {
            version: "2.4",
            group: "process",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: true,
                description: "Turborepo command",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Turborepo result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "voice-call" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Voice call specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Voice call result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "forge-blade" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Blade specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Forge result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "mcporter" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: true,
                description: "MCP server command",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "MCP result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "apple-notes" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: true,
                description: "Apple Notes command",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Apple Notes result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "bear-notes" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "command",
                input_type: InputType::Command,
                required: true,
                description: "Bear Notes command",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Bear Notes result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "hello-mate" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "message",
                input_type: InputType::Text,
                required: false,
                description: "Greeting message",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Greeting response",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "omega-striker" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Action specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Action result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "sigma" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Protocol specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Protocol result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "model-usage" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Model usage query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Usage statistics",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "claude-migration" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Migration specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Migration analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "ast-refactor" => Some(CapabilityContract {
            version: "2.4",
            group: "composite",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Refactoring specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Refactoring result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "connectome" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to analyze",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Connection analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "connectome-rs" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Rust code to analyze",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Rust connection analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "connectome-py" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Python code to analyze",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Python connection analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "connectome-js" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "JavaScript code to analyze",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "JavaScript connection analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "safety-check" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to safety check",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Safety analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "safety-check-py" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Python code to safety check",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Python safety analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "safety-check-js" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "JavaScript code to safety check",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "JavaScript safety analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "polyglot-metrics" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to analyze",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Language metrics",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "polyglot-convert" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Conversion specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Conversion result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "immune-antibody" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "threat",
                input_type: InputType::Text,
                required: true,
                description: "Threat description",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Antibody response",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "immune-log" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "count",
                input_type: InputType::Text,
                required: false,
                description: "Number of entries",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Immune log entries",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "plugin-list" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Installed plugins",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "plugin-install" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Plugin name and source",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Install result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "plugin-remove" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "name",
                input_type: InputType::Text,
                required: true,
                description: "Plugin name",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Remove result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dreamer-loop" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Dream specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Dream result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "auto-evolve" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Evolution specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Evolution result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "adaptive-evolve" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Adaptation specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Adaptation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "self-evolve" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Self-evolution specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Self-evolution result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "mitosis" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Mitosis specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Mitosis result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "bio-mitosis" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Bio-mitosis specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Bio-mitosis result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "metamorphic-trigger" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "generations",
                input_type: InputType::Text,
                required: false,
                description: "Number of generations",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Metamorphic result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "omnicoder" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Mode and code",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Omni-coder result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "agent-factory" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Agent type and capabilities",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Agent blueprint",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "commander" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Command and arguments",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Command result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "swarm-queen" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "count",
                input_type: InputType::Text,
                required: false,
                description: "Number of workers",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Swarm queen result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "replicator" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Target and code",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Replication result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "vision-analyze" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "path",
                input_type: InputType::FilePath,
                required: true,
                description: "Image file path",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Vision analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "vision-compare" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "paths",
                input_type: InputType::Text,
                required: true,
                description: "Two image paths",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Comparison result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "vision-ocr" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "path",
                input_type: InputType::FilePath,
                required: true,
                description: "Image file path",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "OCR result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "geolocation-lookup" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Location query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Coordinates",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "geolocation-memory-map" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Map specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Memory map",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "navigation-route" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "route",
                input_type: InputType::Text,
                required: true,
                description: "Origin and destination",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Route calculation",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "navigation-poi" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "POI query and location",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "POI results",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "collective-decision" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Decision specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Decision result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "collective-consciousness" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "nodes",
                input_type: InputType::Text,
                required: false,
                description: "Number of nodes",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Consciousness link",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "distributed-raft" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Nodes and ID",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Raft consensus",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "distributed-lock" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Resource and timeout",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Lock result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "alan-self-code" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Code and instruction",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Self-coding result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "alan-learn" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Pattern and hours",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Learning result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "templates-refactor" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Template and code",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Refactoring result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "templates-list" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Available templates",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "pollinations-generate" => Some(CapabilityContract {
            version: "2.4",
            group: "external",
            input: vec![InputContract {
                field: "description",
                input_type: InputType::Text,
                required: true,
                description: "Image description",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Image URL",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "pollinations-memory-viz" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Visualization spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Visualization result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "qr-generate" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "data",
                input_type: InputType::Text,
                required: true,
                description: "Data to encode",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "QR code",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "qr-spine" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Spine specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Spine visualization",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "qr-scan" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "path",
                input_type: InputType::FilePath,
                required: true,
                description: "Image file path",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Scan result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "cryo-snap" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Snapshot specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Snapshot result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-mutate" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to mutate",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Mutation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-mutate-point" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to mutate",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Point mutation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-mutate-insert" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to mutate",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Insertion mutation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-mutate-delete" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to mutate",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Deletion mutation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-mutate-optimize" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "code",
                input_type: InputType::Text,
                required: true,
                description: "Code to mutate",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Optimization mutation result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-crossover" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "codes",
                input_type: InputType::Text,
                required: true,
                description: "Two code snippets",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Crossover result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-select" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "population",
                input_type: InputType::Text,
                required: true,
                description: "Population description",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Selection result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-evolve" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Code and generations",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Evolution result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-teach" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "pattern",
                input_type: InputType::Text,
                required: true,
                description: "Teaching pattern",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Teaching result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-hebbian" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Hebbian spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Hebbian learning result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dna-stats" => Some(CapabilityContract {
            version: "2.4",
            group: "algorithm",
            input: vec![InputContract {
                field: "stats",
                input_type: InputType::Text,
                required: true,
                description: "Population stats",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Stats result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "brain" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Mode and code",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Brain analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "brain-compare" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Brain comparison",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dual-cache" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Cache specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Cache result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dual-learn" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Learning specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Learning result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dual-record" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Record specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Record result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dual-status" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Dual generation status",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "dual-teach" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Teaching specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Teaching result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "claude-logic" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Logic query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Logic analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "claude-psi" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "PSI query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "PSI analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "psi-logic" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Logic query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "PSI logic result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "psi-quantum" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Quantum query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Quantum analysis",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "psi" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "query",
                input_type: InputType::Text,
                required: true,
                description: "Framework query",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "PSI framework result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "mintlify" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Documentation spec",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Mintlify docs",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        "test-tui" => Some(CapabilityContract {
            version: "2.4",
            group: "meta",
            input: vec![InputContract {
                field: "spec",
                input_type: InputType::Text,
                required: true,
                description: "Test specification",
            }],
            output: OutputContract {
                output_type: OutputType::Text,
                description: "Test result",
            },
            deprecated: false,
            deprecation_message: None,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_reader_contract_validates_empty_input() {
        let contract = get_contract("code-reader").unwrap();
        assert!(contract.validate_input("").is_err());
    }

    #[test]
    fn code_reader_contract_validates_newline_input() {
        let contract = get_contract("code-reader").unwrap();
        assert!(contract.validate_input("file\nwith\nnewlines").is_err());
    }

    #[test]
    fn code_reader_contract_validates_good_input() {
        let contract = get_contract("code-reader").unwrap();
        assert!(contract.validate_input("src/main.rs").is_ok());
    }

    #[test]
    fn code_writer_contract_validates_hash_format() {
        let contract = get_contract("code-writer").unwrap();
        assert!(contract.validate_input("path|bad|content").is_err());
        assert!(contract.validate_input("path|NEW|content").is_ok());
        assert!(contract
            .validate_input(&format!("path|{}|content", "a".repeat(64)))
            .is_ok());
    }

    #[test]
    fn github_contract_allows_empty_command() {
        let contract = get_contract("github").unwrap();
        assert!(contract.validate_input("").is_ok());
    }

    #[test]
    fn git_nexus_contract_allows_empty_path() {
        let contract = get_contract("git-nexus").unwrap();
        assert!(contract.validate_input("").is_ok());
    }

    #[test]
    fn contract_render_includes_version_and_group() {
        let contract = get_contract("code-reader").unwrap();
        let rendered = contract.render();
        assert!(rendered.contains("version:1.2"));
        assert!(rendered.contains("group:local"));
    }

    #[test]
    fn nonexistent_contract_returns_none() {
        assert!(get_contract("nonexistent-blade").is_none());
    }
}
