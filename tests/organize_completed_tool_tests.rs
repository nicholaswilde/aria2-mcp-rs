mod common;

use anyhow::Result;
use aria2_mcp_rs::{OrganizeCompletedTool, McpeTool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_organize_completed_name() -> Result<()> {
    let tool = OrganizeCompletedTool;
    assert_eq!(tool.name(), "organize_completed");
    Ok(())
}

#[tokio::test]
async fn test_organize_completed_schema() -> Result<()> {
    let tool = OrganizeCompletedTool;
    let schema = tool.schema()?;
    assert_eq!(schema["type"], "object");
    Ok(())
}

#[tokio::test]
async fn test_organize_completed_run_todo() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = OrganizeCompletedTool;

    let args = json!({});
    let result = tool.run(&client, args).await;
    
    // This should fail because it calls todo!()
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not yet implemented"));
    
    Ok(())
}
