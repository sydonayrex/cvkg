use cvkg_cli::mcp::handle_request;

#[test]
fn mcp_tools_list_exposes_component_registry() {
    let response = handle_request(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#)
        .expect("valid MCP request");
    assert!(response.contains("Button"), "response missing Button");
    assert!(response.contains("component_registry"), "missing tool name");
}

#[test]
fn mcp_rejects_unknown_methods() {
    let response = handle_request(r#"{"jsonrpc":"2.0","id":2,"method":"nope"}"#)
        .expect("unknown method still returns JSON-RPC error");
    assert!(response.contains("Method not found"));
}
