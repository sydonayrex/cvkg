//! Minimal MCP discovery server for the component registry.
//!
//! Reads one JSON-RPC request per stdin line and writes one response per line.
//! Only discovery is implemented; resource reads and tool calls are out of
//! scope for this slice.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};

use crate::component_registry::ComponentRegistry;

/// Handle one line-delimited MCP request.
pub fn handle_request(line: &str) -> Result<String, String> {
    let request: Value =
        serde_json::from_str(line).map_err(|error| format!("invalid JSON-RPC request: {error}"))?;
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");

    if method != "tools/list" {
        return Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": "Method not found" },
        })
        .to_string());
    }

    let registry =
        ComponentRegistry::load_default().map_err(|error| format!("registry error: {error}"))?;
    let components: Vec<&str> = registry
        .entries
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    Ok(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": [{
                "name": "component_registry",
                "description": "Lists CVKG component names available for discovery",
                "components": components,
            }],
        },
    })
    .to_string())
}

/// Serve line-delimited requests until stdin closes.
pub fn serve_stdin() -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in BufReader::new(stdin.lock()).lines() {
        let line = line.map_err(|error| format!("stdin error: {error}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let response = handle_request(&line)?;
        writeln!(out, "{response}").map_err(|error| format!("stdout error: {error}"))?;
    }
    Ok(())
}
