mod common;

use anyhow::Result;
use aria2_mcp_rs::server::handler::McpHandler;
use aria2_mcp_rs::tools::registry::ToolRegistry;
use aria2_mcp_rs::Config;
use common::Aria2Container;
use mcp_sdk_rs::server::ServerHandler;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_mcp_handler_tools_list() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
    let handler = McpHandler::new(registry, Arc::new(client));

    let result = handler.handle_method("tools/list", None).await?;
    let tools = result["tools"]
        .as_array()
        .expect("Result should have tools array");

    assert!(!tools.is_empty());
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"manage_downloads"));
    assert!(names.contains(&"monitor_queue"));
    assert!(names.contains(&"inspect_download"));
    assert!(names.contains(&"configure_aria2"));

    Ok(())
}

#[tokio::test]
async fn test_mcp_handler_tools_call() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
    let handler = McpHandler::new(registry, Arc::new(client));

    let params = serde_json::json!({
        "name": "manage_downloads",
        "arguments": {
            "action": "add",
            "uris": ["https://p3terx.com"]
        }
    });

    let result = handler.handle_method("tools/call", Some(params)).await?;
    let content = result["content"]
        .as_array()
        .expect("Result should have content array");
    let text = content[0]["text"]
        .as_str()
        .expect("Content should have text");

    assert!(text.contains("gid"));

    Ok(())
}

#[tokio::test]
async fn test_mcp_handler_unknown_method() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
    let handler = McpHandler::new(registry, Arc::new(client));

    let result = handler.handle_method("unknown/method", None).await;
    assert!(result.is_err());

    Ok(())
}
