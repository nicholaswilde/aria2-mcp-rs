mod common;

use anyhow::Result;
use aria2_mcp_rs::resources::ResourceRegistry;
use aria2_mcp_rs::server::handler::McpHandler;
use aria2_mcp_rs::tools::registry::ToolRegistry;
use aria2_mcp_rs::Config;
use common::Aria2Container;
use mcp_sdk_rs::server::ServerHandler;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_mcp_handler_resources_list() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let handler = McpHandler::new(registry, resource_registry, vec![Arc::new(client.clone())]);

    let result = handler.handle_method("resources/list", None).await?;
    let resources = result["resources"]
        .as_array()
        .expect("Result should have resources array");

    assert!(!resources.is_empty());
    let uris: Vec<&str> = resources
        .iter()
        .map(|r| r["uri"].as_str().unwrap())
        .collect();
    assert!(uris.contains(&"aria2://status/global"));
    assert!(uris.contains(&"aria2://downloads/active"));
    assert!(uris.contains(&"aria2://logs/recent"));

    Ok(())
}

#[tokio::test]
async fn test_mcp_handler_resources_read_global_status() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let handler = McpHandler::new(registry, resource_registry, vec![Arc::new(client.clone())]);

    let params = serde_json::json!({
        "uri": "aria2://status/global"
    });

    let result = handler
        .handle_method("resources/read", Some(params))
        .await?;
    let contents = result["contents"]
        .as_array()
        .expect("Result should have contents array");
    let text = contents[0]["text"]
        .as_str()
        .expect("Content should have text");

    let status: serde_json::Value = serde_json::from_str(text)?;
    assert!(status[0].get("stats").is_some());
    assert_eq!(status[0]["instance"], "default");

    Ok(())
}

#[tokio::test]
async fn test_mcp_handler_resources_read_active_downloads() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let handler = McpHandler::new(registry, resource_registry, vec![Arc::new(client.clone())]);

    // Add a download to make it active
    client
        .add_uri(vec!["https://p3terx.com".to_string()], None)
        .await?;

    let params = serde_json::json!({
        "uri": "aria2://downloads/active"
    });

    let result = handler
        .handle_method("resources/read", Some(params))
        .await?;
    let contents = result["contents"]
        .as_array()
        .expect("Result should have contents array");
    let text = contents[0]["text"]
        .as_str()
        .expect("Content should have text");

    let active: serde_json::Value = serde_json::from_str(text)?;
    assert!(active[0].get("data").is_some());
    assert!(!active[0]["data"].as_array().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_mcp_handler_resources_read_not_found() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let handler = McpHandler::new(registry, resource_registry, vec![Arc::new(client.clone())]);

    let params = serde_json::json!({
        "uri": "aria2://unknown"
    });

    let result = handler.handle_method("resources/read", Some(params)).await;
    assert!(result.is_err());

    Ok(())
}
