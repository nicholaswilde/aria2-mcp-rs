use aria2_mcp_rs::aria2::Aria2Client;
use aria2_mcp_rs::config::Config;
use aria2_mcp_rs::prompts::PromptRegistry;
use aria2_mcp_rs::resources::ResourceRegistry;
use aria2_mcp_rs::server::mcp::{handle_request, JsonRpcRequest, McpState};
use aria2_mcp_rs::tools::ToolRegistry;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_stdio_flow_initialize_and_tools() {
    let state = Arc::new(RwLock::new(McpState::new(false)));
    let config = Config::default();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let prompt_registry = Arc::new(RwLock::new(PromptRegistry::default()));
    let client = Arc::new(Aria2Client::new(config.clone()));
    let clients = vec![client];

    // 1. Initialize
    let req_init = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: None,
        id: Some(aria2_mcp_rs::server::mcp::RequestId::Number(1)),
    };

    let res_init = handle_request(
        req_init,
        Arc::clone(&state),
        Arc::clone(&registry),
        Arc::clone(&resource_registry),
        Arc::clone(&prompt_registry),
        &clients,
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(
        res_init.id,
        Some(aria2_mcp_rs::server::mcp::RequestId::Number(1))
    );
    assert!(res_init.result.is_some());

    // 2. List Tools
    let req_tools = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/list".to_string(),
        params: None,
        id: Some(aria2_mcp_rs::server::mcp::RequestId::Number(2)),
    };

    let res_tools = handle_request(
        req_tools,
        Arc::clone(&state),
        Arc::clone(&registry),
        Arc::clone(&resource_registry),
        Arc::clone(&prompt_registry),
        &clients,
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(
        res_tools.id,
        Some(aria2_mcp_rs::server::mcp::RequestId::Number(2))
    );
    let result_val = res_tools.result.unwrap();
    let tools = result_val["tools"].as_array().unwrap();
    assert!(!tools.is_empty());
}

#[tokio::test]
async fn test_stdio_notifications_queuing() {
    let state = Arc::new(RwLock::new(McpState::new(false)));

    {
        let mut state_guard = state.write().await;
        state_guard.queue_notification("test/event", json!({"foo": "bar"}));
    }

    let mut state_guard = state.write().await;
    let notif = state_guard.pop_notification().unwrap();
    assert_eq!(notif.method, "test/event");
    assert_eq!(notif.params.unwrap()["foo"], "bar");
}
