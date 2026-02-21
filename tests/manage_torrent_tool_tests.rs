mod common;

use anyhow::Result;
use aria2_mcp_rs::{ManageTorrentTool, McpeTool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_manage_torrent_name() -> Result<()> {
    let tool = ManageTorrentTool;
    assert_eq!(tool.name(), "manage_torrent");
    Ok(())
}

#[tokio::test]
async fn test_manage_torrent_schema() -> Result<()> {
    let tool = ManageTorrentTool;
    let schema = tool.schema()?;
    assert_eq!(schema["type"], "object");
    Ok(())
}

#[tokio::test]
async fn test_manage_torrent_get_peers_error() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ManageTorrentTool;

    // Test with invalid GID
    let args = json!({
        "action": "getPeers",
        "gid": "invalid_gid"
    });

    let result = tool.run(&client, args).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_manage_torrent_change_files_error() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ManageTorrentTool;

    // Test with invalid GID
    let args = json!({
        "action": "changeFiles",
        "gid": "invalid_gid",
        "selectedFiles": "1,2,3"
    });

    let result = tool.run(&client, args).await;
    assert!(result.is_err());

    Ok(())
}
