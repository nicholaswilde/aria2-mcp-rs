mod common;

use anyhow::Result;
use aria2_mcp_rs::{McpeTool, SearchDownloadsTool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_mcp_search_downloads_by_query() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = SearchDownloadsTool;

    // Add a download with a specific URI
    let query_uri = "https://p3terx.com/search-test".to_string();
    let uris = vec![query_uri.clone()];
    let _gid = client.add_uri(uris, None).await?;

    // Search by query
    let args = json!({
        "query": "search-test"
    });

    let result = tool.run(&client, args).await?;
    assert!(result.is_array());
    let results = result.as_array().unwrap();
    assert!(!results.is_empty());

    // Check if the found item contains the query in its URIs
    let found = results.iter().any(|item| {
        if let Some(files) = item.get("files") {
            files.as_array().unwrap().iter().any(|file| {
                file.get("uris")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|uri| {
                        uri.get("uri")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .contains("search-test")
                    })
            })
        } else {
            false
        }
    });
    assert!(found, "Should find the download with 'search-test' in URI");

    Ok(())
}

#[tokio::test]
async fn test_mcp_search_downloads_by_status() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = SearchDownloadsTool;

    // Add a paused download
    let uris = vec!["https://p3terx.com/paused-test".to_string()];
    let mut options = serde_json::Map::new();
    options.insert("pause".to_string(), serde_json::json!("true"));
    let _gid = client
        .add_uri(uris, Some(serde_json::Value::Object(options)))
        .await?;

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Search by status
    let args = json!({
        "status": "paused"
    });

    let result = tool.run(&client, args).await?;
    assert!(result.is_array());
    let results = result.as_array().unwrap();
    assert!(!results.is_empty());

    // Check if all found items have the correct status
    for item in results {
        let status = item.get("status").unwrap().as_str().unwrap();
        assert!(status == "paused" || status == "waiting");
    }

    Ok(())
}

#[tokio::test]
async fn test_mcp_search_downloads_with_keys() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = SearchDownloadsTool;

    // Add a download
    let uris = vec!["https://p3terx.com/keys-test".to_string()];
    let _gid = client.add_uri(uris, None).await?;

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Search with specific keys
    let args = json!({
        "query": "keys-test",
        "keys": ["gid", "status"]
    });

    let result = tool.run(&client, args).await?;
    assert!(result.is_array());
    let results = result.as_array().unwrap();
    assert!(!results.is_empty());

    // Check if the found item only has the requested keys (aria2 might return more, but we should check at least these)
    let item = results.first().unwrap();
    assert!(item.get("gid").is_some());
    assert!(item.get("status").is_some());

    Ok(())
}
