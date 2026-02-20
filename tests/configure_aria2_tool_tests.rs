mod common;

use anyhow::Result;
use aria2_mcp_rs::{ConfigureAria2Tool, McpeTool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_configure_aria2_tool_get_global() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ConfigureAria2Tool;

    let args = json!({
        "action": "get_global"
    });

    let result = tool.run(&client, args).await?;

    // Check if result contains global options (e.g., max-overall-download-limit)
    assert!(result.is_object());
    assert!(result.get("max-overall-download-limit").is_some());

    Ok(())
}

#[tokio::test]
async fn test_configure_aria2_tool_change_global() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ConfigureAria2Tool;

    let args = json!({
        "action": "change_global",
        "options": {
            "max-overall-download-limit": "10K"
        }
    });

    tool.run(&client, args).await?;

    // Verify change
    let global_stat = client.get_global_option().await?;
    assert_eq!(global_stat["max-overall-download-limit"], "10240");

    Ok(())
}

#[tokio::test]
async fn test_configure_aria2_tool_local_options() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ConfigureAria2Tool;

    // Add download
    let gid = client
        .add_uri(vec!["https://www.google.com".to_string()], None)
        .await?;

    // Change local option
    let args_change = json!({
        "action": "change_local",
        "gid": gid,
        "options": {
            "max-download-limit": "5K"
        }
    });
    tool.run(&client, args_change).await?;

    // Get local option
    let args_get = json!({
        "action": "get_local",
        "gid": gid
    });
    let result = tool.run(&client, args_get).await?;
    assert_eq!(result["max-download-limit"], "5120");

    Ok(())
}
