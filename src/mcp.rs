use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::commands::build;
use crate::config::ProjectFile;
use crate::device;
use crate::error::{Error, Result};

const PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Deserialize)]
struct Request {
    #[allow(dead_code)]
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

pub fn run() -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut output = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.map_err(|error| Error::Message(error.to_string()))?;
        if line.trim().is_empty() {
            continue;
        }

        let request: Request = match serde_json::from_str(&line) {
            Ok(request) => request,
            Err(error) => {
                write_message(&mut output, &error_response(Value::Null, -32700, error.to_string()))?;
                continue;
            }
        };

        if let Some(response) = handle(request) {
            write_message(&mut output, &response)?;
        }
    }
    Ok(())
}

fn write_message(output: &mut impl Write, message: &Value) -> Result<()> {
    serde_json::to_writer(&mut *output, message)
        .map_err(|error| Error::Message(error.to_string()))?;
    output
        .write_all(b"\n")
        .and_then(|_| output.flush())
        .map_err(|error| Error::Message(error.to_string()))
}

fn handle(request: Request) -> Option<Value> {
    let id = request.id?;
    let result = match request.method.as_str() {
        "initialize" => json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "kpdk", "version": env!("CARGO_PKG_VERSION") }
        }),
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tool_definitions() }),
        "tools/call" => return Some(tool_call_response(id, &request.params)),
        _ => return Some(error_response(id, -32601, "method not found".into())),
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "get_supported_devices",
            "description": "List Padauk devices supported by this kpdk build and their SDCC architectures.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "build_project",
            "description": "Build an existing kpdk project with the locally installed SDK.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Path to the project directory containing pdk.toml." },
                    "release": { "type": "boolean", "default": false }
                },
                "required": ["project"],
                "additionalProperties": false
            }
        }
    ])
}

fn tool_call_response(id: Value, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
    let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
    let result = match call_tool(name, &arguments) {
        Ok(value) => json!({
            "content": [{ "type": "text", "text": value.to_string() }],
            "structuredContent": value,
            "isError": false
        }),
        Err(error) => json!({
            "content": [{ "type": "text", "text": error.to_string() }],
            "isError": true
        }),
    };
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn call_tool(name: &str, arguments: &Value) -> Result<Value> {
    match name {
        "get_supported_devices" => Ok(json!({
            "devices": device::supported_devices().iter().map(|item| json!({
                "name": item.name,
                "architecture": item.architecture.sdcc_target(),
                "otp": item.otp
            })).collect::<Vec<_>>()
        })),
        "build_project" => {
            let project = arguments
                .get("project")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::Message("build_project requires string argument `project`".into()))?;
            let release = arguments.get("release").and_then(Value::as_bool).unwrap_or(false);
            let root = PathBuf::from(project);
            let config = ProjectFile::load(&root)?;
            let artifacts = build::run(&root, release)?;
            Ok(json!({
                "success": true,
                "device": config.project.device.to_ascii_uppercase(),
                "release": release,
                "artifacts": {
                    "ihx": absolute(&artifacts.ihx),
                    "bin": absolute(&artifacts.bin)
                },
                "diagnostics": []
            }))
        }
        _ => Err(Error::Message(format!("unknown tool `{name}`"))),
    }
}

fn absolute(path: &std::path::Path) -> String {
    path.canonicalize().unwrap_or_else(|_| path.to_owned()).to_string_lossy().into_owned()
}

fn error_response(id: Value, code: i32, message: String) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_both_tools() {
        let tools = tool_definitions();
        assert_eq!(tools.as_array().unwrap().len(), 2);
        assert!(tools.to_string().contains("build_project"));
    }

    #[test]
    fn returns_supported_devices() {
        let result = call_tool("get_supported_devices", &json!({})).unwrap();
        assert!(result.to_string().contains("PFS154"));
        assert!(result.to_string().contains("pdk14"));
    }
}
