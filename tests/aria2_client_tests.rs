use crate::common::Aria2Container;
use anyhow::Result;
mod common;

#[tokio::test]
async fn test_get_uris() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();

    // Add a download to get a GID
    let uris = vec!["http://example.org/file".to_string()];
    let gid = client.add_uri(uris, None).await?;

    let result = client.get_uris(&gid).await?;
    assert!(result.is_array());

    let uris_array = result.as_array().unwrap();
    assert!(uris_array
        .iter()
        .any(|uri| uri["uri"] == "http://example.org/file"));

    Ok(())
}
