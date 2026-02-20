mod common;

use anyhow::Result;
use common::Aria2Container;

#[tokio::test]
async fn test_remove_download() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();

    let uris = vec!["https://p3terx.com".to_string()];
    let gid = client.add_uri(uris, None).await?;

    // Verify status is not removed
    let status = client.tell_status(&gid).await?;
    assert_eq!(status["gid"], gid);

    // Remove
    client.remove(&gid).await?;

    // Verify status is removed (aria2.tellStatus should return an error if it's completely gone,
    // or status 'removed' if it's in the stopped list)
    // Actually, aria2.remove moves it to stopped list unless it's already stopped.
    // aria2.remove(gid) -> removes from waiting/active and moves to stopped.

    let status = client.tell_status(&gid).await?;
    assert_eq!(status["status"], "removed");

    Ok(())
}

#[tokio::test]
async fn test_move_position() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();

    let mut options = serde_json::Map::new();
    options.insert("pause".to_string(), serde_json::json!("true"));
    let options = Some(serde_json::Value::Object(options));

    let uris1 = vec!["https://p3terx.com/1".to_string()];
    let _gid1 = client.add_uri(uris1, options.clone()).await?;

    let uris2 = vec!["https://p3terx.com/2".to_string()];
    let gid2 = client.add_uri(uris2, options).await?;

    // Move gid2 to the first position
    let new_pos = client.move_position(&gid2, 0, "POS_SET").await?;
    assert_eq!(new_pos, 0);

    Ok(())
}

#[tokio::test]
async fn test_force_pause_remove() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();

    let uris = vec!["https://p3terx.com".to_string()];
    let gid = client.add_uri(uris, None).await?;

    // Force Pause
    client.force_pause(&gid).await?;
    let status = client.tell_status(&gid).await?;
    assert_eq!(status["status"], "paused");

    // Force Remove
    client.force_remove(&gid).await?;
    let status = client.tell_status(&gid).await;
    // It might be in 'removed' status or completely gone ('not found' error)
    match status {
        Ok(s) => assert_eq!(s["status"], "removed"),
        Err(e) => assert!(e.to_string().contains("not found")),
    }

    Ok(())
}
