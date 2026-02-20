mod common;

use anyhow::Result;
use common::Aria2Container;

#[tokio::test]
async fn test_aria2_client_get_version() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();
    let version = client.get_version().await?;
    assert!(!version.is_empty());
    Ok(())
}
