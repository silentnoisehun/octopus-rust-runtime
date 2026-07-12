use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[serde(default)]
    id: Option<Value>,
    #[serde(default)]
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

struct IncomingMessage {
    payload: String,
    framed: bool,
}

/// Rendered canonical capability list. The MCP `octopus_capabilities` tool and
/// the CLI `capabilities` command both return exactly this string.
pub fn render_capabilities_for_mcp() -> String {
    crate::render_capabilities()
}

pub fn run() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    loop {
        let message = match read_message(&mut reader) {
            Ok(Some(message)) => message,
            Ok(None) => break,
            Err(error) => {
                let response = error_response(Value::Null, -32700, &format!("Read error: {error}"));
                let _ = write_message(&mut writer, &response, true);
                continue;
            }
        };

        let request = match serde_json::from_str::<JsonRpcRequest>(&message.payload) {
            Ok(request) => request,
            Err(error) => {
                let response =
                    error_response(Value::Null, -32700, &format!("Parse error: {error}"));
                let _ = write_message(&mut writer, &response, message.framed);
                continue;
            }
        };

        let id = request.id.unwrap_or(Value::Null);
        let response = match request.method.as_str() {
            "initialize" => initialize_response(id),
            "initialized" => continue,
            "ping" => success_response(id, json!({})),
            "tools/list" => tools_list_response(id),
            "tools/call" => match request.params.as_ref() {
                Some(params) => handle_tool_call(id, params),
                None => error_response(id, -32602, "Missing params"),
            },
            _ => error_response(id, -32601, &format!("Method not found: {}", request.method)),
        };

        let _ = write_message(&mut writer, &response, message.framed);
    }
}

fn handle_tool_call(id: Value, params: &Value) -> Value {
    let tool_name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let result: Result<crate::ExecutionOutcome, String> = match tool_name {
        "octopus_list" => Ok(crate::ExecutionOutcome::completed(crate::list().join("\n"))),
        "octopus_capabilities" => Ok(crate::ExecutionOutcome::completed(
            crate::render_capabilities(),
        )),
        "octopus_run" => run_tool(arguments, crate::run_outcome),
        "octopus_arm" => run_tool(arguments, crate::run_arm_outcome),
        "octopus_pipeline" => run_tool(arguments, crate::run_pipeline_outcome),
        "octopus_status" => match arguments.get("root_id").and_then(Value::as_str) {
            Some(root_id) => Ok(crate::orch_status(root_id)),
            None => Err("Missing root_id".to_string()),
        },
        "octopus_resume" => match arguments.get("root_id").and_then(Value::as_str) {
            Some(root_id) => Ok(crate::orch_resume(root_id)),
            None => Err("Missing root_id".to_string()),
        },
        "octopus_retry" => match arguments.get("arm_id").and_then(Value::as_str) {
            Some(arm_id) => Ok(crate::orch_retry(arm_id)),
            None => Err("Missing arm_id".to_string()),
        },
        "octopus_cancel" => match arguments.get("root_id").and_then(Value::as_str) {
            Some(root_id) => Ok(crate::orch_cancel(root_id)),
            None => Err("Missing root_id".to_string()),
        },
        "octopus_orphans" => Ok(crate::orch_orphans()),
        _ => Err(format!("Unknown tool: {tool_name}")),
    };

    match result {
        Ok(outcome) => tool_outcome_response(id, outcome),
        Err(error) => error_response(id, -32602, &error),
    }
}

fn run_tool<F>(arguments: Value, execute: F) -> Result<crate::ExecutionOutcome, String>
where
    F: FnOnce(&str, &str) -> crate::ExecutionOutcome,
{
    let first = arguments
        .get("blade")
        .or_else(|| arguments.get("spec"))
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing blade/spec".to_string())?;
    let prompt = arguments
        .get("prompt")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing prompt".to_string())?;
    Ok(execute(first, prompt))
}

fn tool_outcome_response(id: Value, outcome: crate::ExecutionOutcome) -> Value {
    success_response(
        id,
        json!({
            "content": [{ "type": "text", "text": outcome.output }],
            "isError": outcome.is_failed(),
            "_meta": {
                "status": outcome.status.as_str(),
                "code": outcome.code
            }
        }),
    )
}

fn initialize_response(id: Value) -> Value {
    success_response(
        id,
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "octopus-runtime",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

fn tools_list_response(id: Value) -> Value {
    success_response(
        id,
        json!({
            "tools": [
                {
                    "name": "octopus_list",
                    "description": "List registered native blades and composite helpers.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    }
                },
                {
                    "name": "octopus_capabilities",
                    "description": "List every blade with its truthful execution mode.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    }
                },
                {
                    "name": "octopus_run",
                    "description": "Run one native blade through the standalone Rust runtime.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "blade": { "type": "string", "description": "Blade name" },
                            "prompt": { "type": "string", "description": "Prompt or payload" }
                        },
                        "required": ["blade", "prompt"]
                    }
                },
                {
                    "name": "octopus_arm",
                    "description": "Run a sequential composite arm such as pipeline-architect + rust-surgeon.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "spec": { "type": "string", "description": "Composite arm specification" },
                            "prompt": { "type": "string", "description": "Prompt or payload" }
                        },
                        "required": ["spec", "prompt"]
                    }
                },
                {
                    "name": "octopus_pipeline",
                    "description": "Run parallel composite arms separated by || under one octopus root.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "spec": { "type": "string", "description": "Pipeline specification" },
                            "prompt": { "type": "string", "description": "Prompt or payload" }
                        },
                        "required": ["spec", "prompt"]
                    }
                },
                {
                    "name": "octopus_status",
                    "description": "Get status of a root orchestration and its events.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "root_id": { "type": "string", "description": "Root ID to query" }
                        },
                        "required": ["root_id"]
                    }
                },
                {
                    "name": "octopus_resume",
                    "description": "Resume an interrupted orchestration root.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "root_id": { "type": "string", "description": "Root ID to resume" }
                        },
                        "required": ["root_id"]
                    }
                },
                {
                    "name": "octopus_retry",
                    "description": "Retry a failed or timed-out arm.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "arm_id": { "type": "string", "description": "Arm ID to retry" }
                        },
                        "required": ["arm_id"]
                    }
                },
                {
                    "name": "octopus_cancel",
                    "description": "Cancel a running orchestration root and all its arms.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "root_id": { "type": "string", "description": "Root ID to cancel" }
                        },
                        "required": ["root_id"]
                    }
                },
                {
                    "name": "octopus_orphans",
                    "description": "List orphaned arms that have no active root.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    }
                }
            ]
        }),
    )
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn read_message<R: BufRead + Read>(reader: &mut R) -> io::Result<Option<IncomingMessage>> {
    let first_line = loop {
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            let n = reader.read(&mut byte)?;
            if n == 0 {
                return Ok(None);
            }
            if byte[0] == b'\n' {
                break;
            }
            if byte[0] != b'\r' {
                buf.push(byte[0]);
            }
        }
        if !buf.is_empty() {
            break String::from_utf8(buf)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
        }
    };

    if first_line.starts_with('{') {
        return Ok(Some(IncomingMessage {
            payload: first_line,
            framed: false,
        }));
    }

    let mut content_length: Option<usize> = None;
    parse_header_line(&first_line, &mut content_length);

    let mut header_line = String::new();
    loop {
        header_line.clear();
        let bytes = reader.read_line(&mut header_line)?;
        if bytes == 0 {
            break;
        }
        if header_line == "\r\n" || header_line == "\n" {
            break;
        }
        parse_header_line(&header_line, &mut content_length);
    }

    let len = content_length.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Missing Content-Length header in framed MCP message",
        )
    })?;

    let mut body = vec![0u8; len];
    reader.read_exact(&mut body)?;
    let payload = String::from_utf8(body)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 JSON payload"))?;

    Ok(Some(IncomingMessage {
        payload,
        framed: true,
    }))
}

fn parse_header_line(line: &str, content_length: &mut Option<usize>) {
    let lower = line.to_ascii_lowercase();
    if lower.starts_with("content-length:") {
        let value = line
            .split_once(':')
            .map(|(_, value)| value.trim())
            .and_then(|value| value.parse::<usize>().ok());
        if let Some(length) = value {
            *content_length = Some(length);
        }
    }
}

fn write_message<W: Write>(writer: &mut W, response: &Value, framed: bool) -> io::Result<()> {
    if framed {
        let payload = response.to_string();
        write!(
            writer,
            "Content-Length: {}\r\n\r\n{}",
            payload.len(),
            payload
        )?;
    } else {
        writeln!(writer, "{}", response)?;
    }
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_execution_is_an_mcp_tool_error_result() {
        let response = tool_outcome_response(
            json!(1),
            crate::ExecutionOutcome::failed("blade_unavailable", "missing"),
        );
        assert_eq!(response["result"]["isError"], json!(true));
        assert_eq!(
            response["result"]["_meta"]["code"],
            json!("blade_unavailable")
        );
    }

    #[test]
    fn completed_execution_is_not_an_mcp_tool_error() {
        let response = tool_outcome_response(json!(1), crate::ExecutionOutcome::completed("ok"));
        assert_eq!(response["result"]["isError"], json!(false));
        assert_eq!(response["result"]["_meta"]["status"], json!("completed"));
    }

    #[test]
    fn octopus_status_tool_returns_is_error_for_missing_root() {
        let response = handle_tool_call(
            json!(1),
            &json!({"name": "octopus_status", "arguments": {"root_id": "nonexistent"}}),
        );
        assert_eq!(response["result"]["isError"], json!(true));
    }

    #[test]
    fn octopus_orphans_tool_returns_completed() {
        let response = handle_tool_call(
            json!(1),
            &json!({"name": "octopus_orphans", "arguments": {}}),
        );
        assert_eq!(response["result"]["isError"], json!(false));
    }

    #[test]
    fn octopus_status_tool_missing_root_id_returns_error() {
        let response = handle_tool_call(
            json!(1),
            &json!({"name": "octopus_status", "arguments": {}}),
        );
        assert_eq!(response["error"]["code"], json!(-32602));
    }

    #[test]
    fn octopus_retry_tool_missing_arm_id_returns_error() {
        let response =
            handle_tool_call(json!(1), &json!({"name": "octopus_retry", "arguments": {}}));
        assert_eq!(response["error"]["code"], json!(-32602));
    }

    #[test]
    fn unknown_tool_returns_method_not_found() {
        let response = handle_tool_call(json!(1), &json!({"name": "nonexistent", "arguments": {}}));
        assert_eq!(response["error"]["code"], json!(-32602));
    }
}
