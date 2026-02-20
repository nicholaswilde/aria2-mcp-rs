mod common;

use anyhow::Result;
use aria2_mcp_rs::{ManageDownloadsTool, Tool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_mcp_manage_downloads_add() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ManageDownloadsTool;

    let args = json!({
        "action": "add",
        "uris": ["https://p3terx.com"]
    });

    let result = tool.execute(&client, args).await?;
    let gid = result["gid"].as_str().expect("GID should be in result");

    // Verify status
    let status = client.tell_status(gid).await?;
    assert_eq!(status["gid"], gid);

    Ok(())
}

#[tokio::test]
async fn test_mcp_manage_downloads_pause_resume() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ManageDownloadsTool;

    // Add
    let args_add = json!({
        "action": "add",
        "uris": ["https://p3terx.com"]
    });
    let result_add = tool.execute(&client, args_add).await?;
    let gid = result_add["gid"].as_str().expect("GID should be in result");

    // Pause
    let args_pause = json!({
        "action": "pause",
        "gid": gid
    });
    tool.execute(&client, args_pause).await?;
    let status = client.tell_status(gid).await?;
    assert_eq!(status["status"], "paused");

    // Resume
    let args_resume = json!({
        "action": "resume",
        "gid": gid
    });
    tool.execute(&client, args_resume).await?;
    let status = client.tell_status(gid).await?;
    assert!(status["status"] == "active" || status["status"] == "waiting");

    Ok(())
}

#[tokio::test]
async fn test_mcp_manage_downloads_remove() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = ManageDownloadsTool;

    let args_add = json!({
        "action": "add",
        "uris": ["https://p3terx.com"]
    });
    let result_add = tool.execute(&client, args_add).await?;
    let gid = result_add["gid"].as_str().expect("GID should be in result");

    // Remove
    let args_remove = json!({
        "action": "remove",
        "gid": gid
    });
    tool.execute(&client, args_remove).await?;
    let status = client.tell_status(gid).await?;
    assert_eq!(status["status"], "removed");

    Ok(())
}
