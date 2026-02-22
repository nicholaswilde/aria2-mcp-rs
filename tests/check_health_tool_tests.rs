mod common;

use anyhow::Result;
use aria2_mcp_rs::{CheckHealthTool, McpeTool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_check_health_name() -> Result<()> {
    let tool = CheckHealthTool;
    assert_eq!(tool.name(), "check_health");
    Ok(())
}

#[tokio::test]
async fn test_check_health_schema() -> Result<()> {
    let tool = CheckHealthTool;
    let schema = tool.schema()?;
    assert_eq!(schema["type"], "object");
    Ok(())
}

#[tokio::test]
async fn test_check_health_basic() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = CheckHealthTool;

    let args = json!({});
    let result = tool.run(&client, args).await?;

    assert!(result.get("summary").is_some());
    assert!(result.get("issues").is_some());

    Ok(())
}

#[tokio::test]
async fn test_check_health_with_stalled() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = CheckHealthTool;

    // Add a download that will likely be stalled (invalid URI)
    let uris = vec!["http://invalid.domain/file.zip".to_string()];
    let _gid = client.add_uri(uris, None).await?;

    // Wait a bit for it to be processed
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let args = json!({});
    let result = tool.run(&client, args).await?;

    let issues = result["issues"].as_array().unwrap();
    // It should identify at least one issue if it's stalled or has errors
    assert!(!issues.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_check_health_multi_instance() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = std::sync::Arc::new(container.client());
    let tool = CheckHealthTool;

    let args = json!({});
    let result = tool.run_multi(&[client.clone()], args).await?;

    // Multi-instance health should return a list of results
    assert!(result.get("results").is_some());
    let results = result["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["instance"], client.name);

    Ok(())
}
