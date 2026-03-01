use aria2_mcp_rs::config::Config;
use aria2_mcp_rs::prompts::PromptRegistry;
use aria2_mcp_rs::resources::ResourceRegistry;
use aria2_mcp_rs::server::mcp::{handle_request, JsonRpcRequest, McpState, RequestId};
use aria2_mcp_rs::tools::ToolRegistry;
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
        clients.clone(),
    )
    .await
    .unwrap()
    .unwrap();

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
        clients.clone(),
    )
    .await
    .unwrap()
    .unwrap();

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
        clients.clone(),
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(res.id, Some(RequestId::Number(99)));
    assert_eq!(res.error.unwrap().code, -32601);
}

#[tokio::test]
async fn test_handle_request_tools_call() {
    let state = Arc::new(RwLock::new(McpState::new(false)));
    let config = Config::default();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let prompt_registry = Arc::new(RwLock::new(PromptRegistry::default()));

    // Mock client that returns success for getVersion (used by some tools maybe?)
    // Actually we just need to test the response structure.
    // MonitorQueueTool with action stats should work if we can mock it.
    // But Aria2Client is not easily mockable without a server.
    // However, we can use a tool that doesn't need a client or fails gracefully.

    // Let's create a dummy tool and register it.

    struct DummyTool;
    #[async_trait::async_trait]
    impl aria2_mcp_rs::tools::registry::McpeTool for DummyTool {
        fn name(&self) -> String {
            "dummy".to_string()
        }
        fn description(&self) -> String {
            "dummy".to_string()
        }
        fn schema(&self) -> anyhow::Result<serde_json::Value> {
            Ok(json!({}))
        }
        async fn run(
            &self,
            _client: &aria2_mcp_rs::Aria2Client,
            _args: serde_json::Value,
        ) -> anyhow::Result<serde_json::Value> {
            Ok(json!({"status": "ok"}))
        }
    }

    {
        let mut reg = registry.write().await;
        reg.register(Arc::new(DummyTool));
    }

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "dummy",
            "arguments": {}
        })),
        id: Some(RequestId::Number(1)),
    };

    let res = handle_request(
        req,
        Arc::clone(&state),
        Arc::clone(&registry),
        Arc::clone(&resource_registry),
        Arc::clone(&prompt_registry),
        vec![Arc::new(aria2_mcp_rs::Aria2Client::new(Config::default()))],
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(res.id, Some(RequestId::Number(1)));
    let result = res.result.unwrap();
    assert!(result["content"].is_array());
    assert_eq!(result["content"][0]["type"], "text");
    assert!(result["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("ok"));
}

#[tokio::test]
async fn test_handle_request_resources_read() {
    let state = Arc::new(RwLock::new(McpState::new(false)));
    let config = Config::default();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let mut resource_registry = ResourceRegistry::default();
    let prompt_registry = Arc::new(RwLock::new(PromptRegistry::default()));

    struct DummyResource;
    #[async_trait::async_trait]
    impl aria2_mcp_rs::resources::McpResource for DummyResource {
        fn name(&self) -> String {
            "dummy".to_string()
        }
        fn uri(&self) -> String {
            "dummy://res".to_string()
        }
        fn description(&self) -> Option<String> {
            None
        }
        fn mime_type(&self) -> Option<String> {
            Some("text/plain".to_string())
        }
        async fn read(
            &self,
            _client: &aria2_mcp_rs::Aria2Client,
        ) -> anyhow::Result<serde_json::Value> {
            Ok(json!("content"))
        }
        async fn read_multi(
            &self,
            _clients: &[Arc<aria2_mcp_rs::Aria2Client>],
        ) -> anyhow::Result<serde_json::Value> {
            Ok(json!("content"))
        }
    }

    resource_registry.register(Arc::new(DummyResource));
    let resource_registry = Arc::new(RwLock::new(resource_registry));

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "resources/read".to_string(),
        params: Some(json!({
            "uri": "dummy://res"
        })),
        id: Some(RequestId::Number(1)),
    };

    let res = handle_request(
        req,
        Arc::clone(&state),
        Arc::clone(&registry),
        Arc::clone(&resource_registry),
        Arc::clone(&prompt_registry),
        vec![Arc::new(aria2_mcp_rs::Aria2Client::new(Config::default()))],
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(res.id, Some(RequestId::Number(1)));
    let result = res.result.unwrap();
    assert!(result["contents"].is_array());
    assert_eq!(result["contents"][0]["uri"], "dummy://res");
    assert_eq!(result["contents"][0]["mimeType"], "text/plain");
    assert_eq!(result["contents"][0]["text"], "content");
}
