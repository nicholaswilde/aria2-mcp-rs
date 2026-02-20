mod common;

use anyhow::Result;
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_aria2_client_get_option() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();

    // Add a download to get a GID
    let uris = vec!["https://www.google.com".to_string()];
    let gid = client.add_uri(uris, None).await?;

    // Get options for the download
    let options = client.get_option(&gid).await?;
    
    // Check if we got some options back (e.g., dir or max-connection-per-server)
    assert!(options.is_object());
    
    Ok(())
}

#[tokio::test]
async fn test_aria2_client_change_option() -> Result<()> {
    let container = Aria2Container::new().await?;
    let client = container.client();

    // Add a download to get a GID
    let uris = vec!["https://www.google.com".to_string()];
    let gid = client.add_uri(uris, None).await?;

    // Change option
    let new_limit = "10K";
    let options = json!({
        "max-download-limit": new_limit
    });
    
    client.change_option(&gid, options).await?;

    // Verify the change
    let current_options = client.get_option(&gid).await?;
    let limit = current_options["max-download-limit"].as_str().unwrap();
    
    // Aria2 converts 10K to bytes
    assert_eq!(limit, "10240");
    
    Ok(())
}
