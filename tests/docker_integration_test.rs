mod common;

use common::Aria2Container;
use anyhow::Result;

#[tokio::test]
async fn test_add_download() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    
    let uris = vec!["https://p3terx.com".to_string()];
    let gid = client.add_uri(uris, None).await?;
    
    assert!(!gid.is_empty());
    println!("added download with GID: {}", gid);
    
    // Verify status
    let status = client.tell_status(&gid).await?;
    assert_eq!(status["gid"], gid);
    let status_str = status["status"].as_str().unwrap();
    println!("download status: {}", status_str);
    assert!(status_str == "active" || status_str == "waiting" || status_str == "complete");
    
    Ok(())
}

#[tokio::test]
async fn test_container_starts_and_is_reachable() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let version = client.get_version().await?;
    println!("aria2 version: {}", version);
    assert!(!version.is_empty());
    Ok(())
}
