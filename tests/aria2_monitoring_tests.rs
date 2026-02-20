mod common;

use anyhow::Result;
use common::Aria2Container;

#[tokio::test]
async fn test_tell_active() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();

    // Add a download
    let uris = vec!["https://p3terx.com".to_string()];
    let _gid = client.add_uri(uris, None).await?;

    // tell_active
    let active = client.tell_active(None).await?;
    assert!(!active.as_array().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_tell_waiting() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();

    // Add a paused download
    let uris = vec!["https://p3terx.com/waiting".to_string()];
    let mut options = serde_json::Map::new();
    options.insert("pause".to_string(), serde_json::json!("true"));
    let _gid = client
        .add_uri(uris, Some(serde_json::Value::Object(options)))
        .await?;

    // tell_waiting
    let waiting = client.tell_waiting(0, 10, None).await?;
    assert!(!waiting.as_array().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_tell_stopped() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();

    // Add and remove a download to have it in stopped
    let uris = vec!["https://p3terx.com/stopped".to_string()];
    let gid = client.add_uri(uris, None).await?;
    client.remove(&gid).await?;

    // tell_stopped
    let stopped = client.tell_stopped(0, 10, None).await?;
    assert!(!stopped.as_array().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_global_stat() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();

    let stats = client.get_global_stat().await?;
    assert!(stats.get("numActive").is_some());

    Ok(())
}
