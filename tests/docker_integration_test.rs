mod common;

use common::Aria2Container;
use anyhow::Result;

#[tokio::test]
async fn test_container_starts_and_is_reachable() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let version = client.get_version().await?;
    println!("aria2 version: {}", version);
    assert!(!version.is_empty());
    Ok(())
}
