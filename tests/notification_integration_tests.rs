mod common;

use anyhow::Result;
use aria2_mcp_rs::aria2::notifications::Aria2Event;
use common::Aria2Container;
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_notification_flow_with_container() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    
    let (tx, mut rx) = mpsc::channel(100);
    client.start_notifications(tx).await?;
    
    // Give it a moment to connect
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Trigger an event: add a download
    client.add_uri(vec!["https://example.com/file.zip".to_string()], None).await?;
    
    // We expect a DownloadStart event
    let notification = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await?;
    let notification = notification.expect("Should have received a notification");
    
    assert_eq!(notification.method, Aria2Event::DownloadStart);
    
    // Map to MCP and verify
    let mcp = notification.to_mcp_notification();
    assert_eq!(mcp["method"], "notifications/aria2/event");
    assert_eq!(mcp["params"]["event"], "download_start");
    assert!(mcp["params"]["gid"].as_str().unwrap().len() > 0);

    Ok(())
}
