mod common;

use anyhow::Result;
use aria2_mcp_rs::{AddRssFeedTool, ListRssFeedsTool, McpeTool};
use common::Aria2Container;
use serde_json::json;

#[tokio::test]
async fn test_add_rss_feed_tool() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let tool = AddRssFeedTool;

    let args = json!({
        "url": "https://example.com/rss",
        "name": "Example Feed",
        "filters": ["keyword1", "regex:.*keyword2.*"]
    });

    let result = tool.run(&client, args).await?;

    assert_eq!(result["status"], "success");
    assert!(result["message"].as_str().unwrap().contains("Example Feed"));

    Ok(())
}

#[tokio::test]
async fn test_list_rss_feeds_tool() -> Result<()> {
    if !common::should_run_docker_tests() {
        return Ok(());
    }
    let container = Aria2Container::new().await?;
    let client = container.client();
    let add_tool = AddRssFeedTool;
    let list_tool = ListRssFeedsTool;

    // Add a feed first
    let args = json!({
        "url": "https://example.com/rss",
        "name": "Example Feed"
    });
    add_tool.run(&client, args).await?;

    let result = list_tool.run(&client, json!({})).await?;

    let feeds = result["feeds"].as_array().unwrap();
    assert_eq!(feeds.len(), 1);
    assert_eq!(feeds[0]["name"], "Example Feed");
    assert_eq!(feeds[0]["url"], "https://example.com/rss");

    Ok(())
}

#[test]
fn test_matches_filters_internal() {
    use aria2_mcp_rs::config::RSSFilter;
    use aria2_mcp_rs::tools::rss::matches_filters;

    let filters = vec![
        RSSFilter::Keyword("ubuntu".to_string()),
        RSSFilter::Regex(".*debian.*".to_string()),
    ];

    assert!(matches_filters("Ubuntu 24.04 Release", &filters));
    assert!(matches_filters("Debian 12 ISO", &filters));
    assert!(!matches_filters("Fedora 40", &filters));
    assert!(matches_filters("Any Title", &[]));
}
