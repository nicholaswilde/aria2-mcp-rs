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
    client
        .add_uri(vec!["https://example.com/file.zip".to_string()], None)
        .await?;

    // We expect a DownloadStart event
    let notification = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await?;
    let notification = notification.expect("Should have received a notification");

    assert_eq!(notification.method, Aria2Event::DownloadStart);

    // Map to MCP and verify
    let mcp = notification.to_mcp_notification();
    assert_eq!(mcp["method"], "notifications/aria2/event");
    assert_eq!(mcp["params"]["event"], "download_start");
    assert!(!mcp["params"]["gid"].as_str().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_notification_flow_multi_instance() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container1 = Aria2Container::new().await?;
    let container2 = Aria2Container::new().await?;

    let mut client1 = container1.client();
    client1.name = "box1".to_string();

    let mut client2 = container2.client();
    client2.name = "box2".to_string();

    let (tx, mut rx) = mpsc::channel(100);
    client1.start_notifications(tx.clone()).await?;
    client2.start_notifications(tx).await?;

    // Give them a moment to connect
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Trigger event on box1
    client1
        .add_uri(vec!["https://example.com/file1.zip".to_string()], None)
        .await?;
    let n1 = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await?
        .unwrap();
    assert_eq!(n1.method, Aria2Event::DownloadStart);

    // Trigger event on box2
    client2
        .add_uri(vec!["https://example.com/file2.zip".to_string()], None)
        .await?;
    let n2 = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await?
        .unwrap();
    assert_eq!(n2.method, Aria2Event::DownloadStart);

    Ok(())
}
