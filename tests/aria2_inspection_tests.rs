mod common;

use anyhow::Result;
use common::Aria2Container;

#[tokio::test]
async fn test_get_files() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();

    // Add a download to have something to inspect
    let uris = vec!["https://example.com/file.txt".to_string()];
    let gid = client.add_uri(uris, None).await?;

    let files = client.get_files(&gid).await?;
    assert!(files.is_array());
    assert!(!files.as_array().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_uris() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();

    // Add a download to have something to inspect
    let uris = vec!["https://example.com/file.txt".to_string()];
    let gid = client.add_uri(uris, None).await?;

    let uris_resp = client.get_uris(&gid).await?;
    assert!(uris_resp.is_array());
    assert!(!uris_resp.as_array().unwrap().is_empty());

    Ok(())
}
