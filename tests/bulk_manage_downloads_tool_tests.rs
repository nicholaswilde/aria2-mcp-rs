mod common;

use anyhow::Result;
use aria2_mcp_rs::{BulkManageDownloadsTool, McpeTool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_bulk_manage_downloads_pause() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = BulkManageDownloadsTool;

    // Add multiple downloads
    let gid1 = client
        .add_uri(vec!["https://example.com/1".to_string()], None)
        .await?;
    let gid2 = client
        .add_uri(vec!["https://example.com/2".to_string()], None)
        .await?;

    let args = json!({
        "action": "pause",
        "gids": [gid1.clone(), gid2.clone()]
    });

    let result = tool.run(&client, args).await?;

    assert_eq!(
        result.get("success_count").and_then(|v| v.as_u64()),
        Some(2)
    );
    assert_eq!(
        result.get("failure_count").and_then(|v| v.as_u64()),
        Some(0)
    );

    // Verify both are paused
    let status1 = client.tell_status(&gid1).await?;
    let status2 = client.tell_status(&gid2).await?;
    assert_eq!(status1["status"], "paused");
    assert_eq!(status2["status"], "paused");

    Ok(())
}

#[tokio::test]
async fn test_bulk_manage_downloads_resume() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = BulkManageDownloadsTool;

    // Add multiple downloads and pause them
    let gid1 = client
        .add_uri(vec!["https://example.com/1".to_string()], None)
        .await?;
    let gid2 = client
        .add_uri(vec!["https://example.com/2".to_string()], None)
        .await?;
    client.pause(&gid1).await?;
    client.pause(&gid2).await?;

    let args = json!({
        "action": "resume",
        "gids": [gid1.clone(), gid2.clone()]
    });

    let result = tool.run(&client, args).await?;

    assert_eq!(
        result.get("success_count").and_then(|v| v.as_u64()),
        Some(2)
    );
    assert_eq!(
        result.get("failure_count").and_then(|v| v.as_u64()),
        Some(0)
    );

    // Verify both are no longer paused (could be waiting or active)
    let status1 = client.tell_status(&gid1).await?;
    let status2 = client.tell_status(&gid2).await?;
    assert_ne!(status1["status"], "paused");
    assert_ne!(status2["status"], "paused");

    Ok(())
}

#[tokio::test]
async fn test_bulk_manage_downloads_remove() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = BulkManageDownloadsTool;

    // Add multiple downloads
    let gid1 = client
        .add_uri(vec!["https://example.com/1".to_string()], None)
        .await?;
    let gid2 = client
        .add_uri(vec!["https://example.com/2".to_string()], None)
        .await?;

    let args = json!({
        "action": "remove",
        "gids": [gid1.clone(), gid2.clone()]
    });

    let result = tool.run(&client, args).await?;

    assert_eq!(
        result.get("success_count").and_then(|v| v.as_u64()),
        Some(2)
    );

    // Verify they are removed or in stopped status
    let result1 = client.tell_status(&gid1).await;
    let result2 = client.tell_status(&gid2).await;

    // tell_status on removed gid might return error or status removed
    if let Ok(status) = result1 {
        assert!(status["status"] == "removed" || status["status"] == "complete");
    }
    if let Ok(status) = result2 {
        assert!(status["status"] == "removed" || status["status"] == "complete");
    }

    Ok(())
}
