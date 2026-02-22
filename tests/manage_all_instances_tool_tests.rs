mod common;

use aria2_mcp_rs::config::Config;
use aria2_mcp_rs::tools::manage_all_instances::ManageAllInstancesTool;
use aria2_mcp_rs::tools::registry::{McpeTool, ToolRegistry};
use common::Aria2Container;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_manage_all_instances_tool_exists() {
    let config = Config::default();
    let registry = ToolRegistry::new(&config);

    let tool = registry.get_tool("manage_all_instances");
    assert!(
        tool.is_some(),
        "manage_all_instances tool should be registered"
    );
}

#[tokio::test]
async fn test_manage_all_instances_schema() {
    let config = Config::default();
    let registry = ToolRegistry::new(&config);
    let tool = registry.get_tool("manage_all_instances").unwrap();
    let schema = tool.schema().unwrap();

    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["action"].is_object());
    assert!(schema["required"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("action")));
}

#[tokio::test]
async fn test_manage_all_instances_run_multi() -> anyhow::Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = Arc::new(container.client());
    let tool = ManageAllInstancesTool;

    // Test pause
    let args = json!({"action": "pause"});
    let result = tool.run_multi(std::slice::from_ref(&client), args).await?;
    assert_eq!(result["action"], "pause");
    assert_eq!(result["results"][0]["status"], "ok");

    // Test resume
    let args = json!({"action": "resume"});
    let result = tool.run_multi(std::slice::from_ref(&client), args).await?;
    assert_eq!(result["action"], "resume");
    assert_eq!(result["results"][0]["status"], "ok");

    // Test purge
    let args = json!({"action": "purge"});
    let result = tool.run_multi(std::slice::from_ref(&client), args).await?;
    assert_eq!(result["action"], "purge");
    assert_eq!(result["results"][0]["status"], "ok");

    Ok(())
}

#[tokio::test]
async fn test_manage_all_instances_error_cases() -> Result<(), Box<dyn std::error::Error>> {
    let tool = ManageAllInstancesTool;
    let config = Config::default();
    let client = Arc::new(aria2_mcp_rs::aria2::Aria2Client::new(config));

    // Test run (single client) - should error
    let res = tool.run(&client, json!({})).await;
    assert!(res.is_err());

    // Test run_multi with missing action
    let res = tool
        .run_multi(std::slice::from_ref(&client), json!({}))
        .await;
    assert!(res.is_err());

    // Test run_multi with invalid action
    let res = tool
        .run_multi(std::slice::from_ref(&client), json!({"action": "invalid"}))
        .await;
    assert!(res.is_ok()); // The tool returns a result list with errors, not a Result::Err
    assert_eq!(res.unwrap()["results"][0]["status"], "error");

    Ok(())
}
