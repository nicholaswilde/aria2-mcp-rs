use mcp_sdk_rs::server::ServerHandler;
use std::sync::Arc;
use tokio::sync::RwLock;

use aria2_mcp_rs::aria2::Aria2Client;
use aria2_mcp_rs::config::Config;
use aria2_mcp_rs::server::handler::McpHandler;
use aria2_mcp_rs::tools::registry::ToolRegistry;

#[tokio::test]
async fn test_manage_all_instances_tool_exists() {
    let config = Config::default();
    let registry = ToolRegistry::new(&config);
    
    let tool = registry.get_tool("manage_all_instances");
    assert!(tool.is_some(), "manage_all_instances tool should be registered");
}

#[tokio::test]
async fn test_manage_all_instances_schema() {
    let config = Config::default();
    let registry = ToolRegistry::new(&config);
    let tool = registry.get_tool("manage_all_instances").unwrap();
    let schema = tool.schema().unwrap();
    
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["action"].is_object());
    assert!(schema["required"].as_array().unwrap().contains(&serde_json::json!("action")));
}
