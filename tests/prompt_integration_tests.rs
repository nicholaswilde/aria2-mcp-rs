mod common;

use anyhow::Result;
use aria2_mcp_rs::prompts::PromptRegistry;
use aria2_mcp_rs::resources::ResourceRegistry;
use aria2_mcp_rs::server::handler::McpHandler;
use aria2_mcp_rs::tools::registry::ToolRegistry;
use aria2_mcp_rs::Config;
use common::Aria2Container;
use mcp_sdk_rs::server::ServerHandler;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_mcp_handler_prompts_list() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let prompt_registry = Arc::new(RwLock::new(PromptRegistry::default()));
    let handler = McpHandler::new(
        registry,
        resource_registry,
        prompt_registry,
        vec![Arc::new(client.clone())],
    );

    let result = handler.handle_method("prompts/list", None).await?;
    let prompts = result["prompts"]
        .as_array()
        .expect("Result should have prompts array");

    assert!(!prompts.is_empty());
    let names: Vec<&str> = prompts
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"diagnose-download"));
    assert!(names.contains(&"optimize-schedule"));

    Ok(())
}

#[tokio::test]
async fn test_mcp_handler_prompts_get_diagnose() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let prompt_registry = Arc::new(RwLock::new(PromptRegistry::default()));
    let handler = McpHandler::new(
        registry,
        resource_registry,
        prompt_registry,
        vec![Arc::new(client.clone())],
    );

    let params = serde_json::json!({
        "name": "diagnose-download",
        "arguments": {
            "gid": "12345"
        }
    });

    let result = handler.handle_method("prompts/get", Some(params)).await?;
    let messages = result["messages"]
        .as_array()
        .expect("Result should have messages array");

    assert_eq!(messages.len(), 3);
    assert!(messages[0]["content"]["text"]
        .as_str()
        .unwrap()
        .contains("12345"));
    assert_eq!(
        messages[2]["content"]["resource"]["uri"],
        "aria2://logs/recent"
    );

    Ok(())
}

#[tokio::test]
async fn test_mcp_handler_prompts_get_optimize() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let prompt_registry = Arc::new(RwLock::new(PromptRegistry::default()));
    let handler = McpHandler::new(
        registry,
        resource_registry,
        prompt_registry,
        vec![Arc::new(client.clone())],
    );

    let params = serde_json::json!({
        "name": "optimize-schedule"
    });

    let result = handler.handle_method("prompts/get", Some(params)).await?;
    let messages = result["messages"]
        .as_array()
        .expect("Result should have messages array");

    assert_eq!(messages.len(), 1);
    assert!(messages[0]["content"]["text"]
        .as_str()
        .unwrap()
        .contains("optimize"));

    Ok(())
}
