use mcp_sdk_rs::server::ServerHandler;
use std::sync::Arc;
use tokio::sync::RwLock;

use aria2_mcp_rs::aria2::Aria2Client;
use aria2_mcp_rs::config::Config;
use aria2_mcp_rs::resources::ResourceRegistry;
use aria2_mcp_rs::server::handler::McpHandler;
use aria2_mcp_rs::tools::registry::ToolRegistry;

#[tokio::test]
async fn test_mcp_handler_routing_default() {
    let config = Config::default();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let client1 = Arc::new(Aria2Client::new(config.clone()));
    let client2 = Arc::new(Aria2Client::new(config.clone()));
    let handler = McpHandler::new(
        registry,
        resource_registry,
        vec![client1.clone(), client2.clone()],
    );

    // Call a tool without instance argument, should use client1
    let params = serde_json::json!({
        "name": "check_health",
        "arguments": {}
    });

    // We can't easily check which client was used without mocking Aria2Client
    // but we can check if it compiles and runs.
    let _ = handler.handle_method("tools/call", Some(params)).await;
}

#[tokio::test]
async fn test_mcp_handler_routing_specific_instance() {
    let config = Config::default();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let client1 = Arc::new(Aria2Client::new(config.clone()));
    let client2 = Arc::new(Aria2Client::new(config.clone()));
    let handler = McpHandler::new(
        registry,
        resource_registry,
        vec![client1.clone(), client2.clone()],
    );

    // Call a tool with instance 1, should use client2
    let params = serde_json::json!({
        "name": "check_health",
        "arguments": {
            "instance": 1
        }
    });

    let _ = handler.handle_method("tools/call", Some(params)).await;
}

#[tokio::test]
async fn test_mcp_handler_routing_invalid_instance() {
    let config = Config::default();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let client1 = Arc::new(Aria2Client::new(config.clone()));
    let handler = McpHandler::new(registry, resource_registry, vec![client1.clone()]);

    // Call a tool with non-existent instance 1
    let params = serde_json::json!({
        "name": "check_health",
        "arguments": {
            "instance": 1
        }
    });

    let result = handler.handle_method("tools/call", Some(params)).await;
    assert!(result.is_err());
    // Should return a clear error about invalid instance
}
