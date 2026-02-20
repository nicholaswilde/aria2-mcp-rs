mod common;

use anyhow::Result;
use aria2_mcp_rs::{MonitorQueueTool, Tool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_mcp_monitor_queue_stats() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = MonitorQueueTool;

    let args = json!({
        "action": "stats"
    });

    let result = tool.execute(&client, args).await?;
    assert!(result.get("numActive").is_some());

    Ok(())
}

#[tokio::test]
async fn test_mcp_monitor_queue_active() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = MonitorQueueTool;

    // Add a download
    let uris = vec!["https://p3terx.com".to_string()];
    let _gid = client.add_uri(uris, None).await?;

    let args = json!({
        "action": "active"
    });

    let result = tool.execute(&client, args).await?;
    assert!(result.is_array());
    assert!(!result.as_array().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_mcp_monitor_queue_waiting() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = MonitorQueueTool;

    // Add a paused download
    let uris = vec!["https://p3terx.com/waiting".to_string()];
    let mut options = serde_json::Map::new();
    options.insert("pause".to_string(), serde_json::json!("true"));
    let _gid = client
        .add_uri(uris, Some(serde_json::Value::Object(options)))
        .await?;

    let args = json!({
        "action": "waiting"
    });

    let result = tool.execute(&client, args).await?;
    assert!(result.is_array());
    assert!(!result.as_array().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_mcp_monitor_queue_stopped() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = MonitorQueueTool;

    // Add and remove a download to have it in stopped
    let uris = vec!["https://p3terx.com/stopped".to_string()];
    let gid = client.add_uri(uris, None).await?;
    client.remove(&gid).await?;

    let args = json!({
        "action": "stopped"
    });

    let result = tool.execute(&client, args).await?;
    assert!(result.is_array());
    assert!(!result.as_array().unwrap().is_empty());

    Ok(())
}
