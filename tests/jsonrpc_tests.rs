use aria2_mcp_rs::server::mcp::{JsonRpcRequest, JsonRpcResponse, JsonRpcError, RequestId, McpState, handle_request};
use aria2_mcp_rs::tools::ToolRegistry;
use aria2_mcp_rs::resources::ResourceRegistry;
use aria2_mcp_rs::prompts::PromptRegistry;
use aria2_mcp_rs::config::Config;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_deserialize_request_with_id() {
    let data = json!({
        "jsonrpc": "2.0",
        "method": "test_method",
        "params": {"key": "value"},
        "id": 1
    });
    let req: JsonRpcRequest = serde_json::from_value(data).unwrap();
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "test_method");
    assert_eq!(req.params.unwrap()["key"], "value");
    assert_eq!(req.id, Some(RequestId::Number(1)));
}

#[test]
fn test_mcp_state_notifications() {
    let mut state = McpState::new(false);
    state.queue_notification("test_event", json!({"info": "data"}));
    let notif = state.pop_notification().unwrap();
    assert_eq!(notif.method, "test_event");
}

#[tokio::test]
async fn test_handle_request_initialize() {
    let state = Arc::new(RwLock::new(McpState::new(false)));
    let config = Config::default();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let prompt_registry = Arc::new(RwLock::new(PromptRegistry::default()));
    let clients = vec![];

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: None,
        id: Some(RequestId::Number(1)),
    };

    let res = handle_request(
        req,
        Arc::clone(&state),
        Arc::clone(&registry),
        Arc::clone(&resource_registry),
        Arc::clone(&prompt_registry),
        &clients
    ).await.unwrap().unwrap();

    assert_eq!(res.id, Some(RequestId::Number(1)));
    assert_eq!(res.result.unwrap()["serverInfo"]["name"], "aria2-mcp-rs");
    
    let state_guard = state.read().await;
    assert!(state_guard.initialized);
}

#[tokio::test]
async fn test_handle_request_tools_list() {
    let state = Arc::new(RwLock::new(McpState::new(false)));
    let config = Config::default();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let prompt_registry = Arc::new(RwLock::new(PromptRegistry::default()));
    let clients = vec![];

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/list".to_string(),
        params: None,
        id: Some(RequestId::String("req-1".to_string())),
    };

    let res = handle_request(
        req,
        Arc::clone(&state),
        Arc::clone(&registry),
        Arc::clone(&resource_registry),
        Arc::clone(&prompt_registry),
        &clients
    ).await.unwrap().unwrap();

    assert_eq!(res.id, Some(RequestId::String("req-1".to_string())));
    assert!(res.result.unwrap()["tools"].is_array());
}

#[tokio::test]
async fn test_handle_request_method_not_found() {
    let state = Arc::new(RwLock::new(McpState::new(false)));
    let config = Config::default();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let prompt_registry = Arc::new(RwLock::new(PromptRegistry::default()));
    let clients = vec![];

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "unknown_method".to_string(),
        params: None,
        id: Some(RequestId::Number(99)),
    };

    let res = handle_request(
        req,
        Arc::clone(&state),
        Arc::clone(&registry),
        Arc::clone(&resource_registry),
        Arc::clone(&prompt_registry),
        &clients
    ).await.unwrap().unwrap();

    assert_eq!(res.id, Some(RequestId::Number(99)));
    assert_eq!(res.error.unwrap().code, -32601);
}
