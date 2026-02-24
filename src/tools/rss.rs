use crate::aria2::Aria2Client;
use crate::config::{RSSFeed, RSSFilter};
use crate::tools::registry::McpeTool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use rss::Channel;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::{self, Duration};

pub async fn start_rss_monitoring(client: Arc<Aria2Client>) -> Result<()> {
    let mut interval = time::interval(Duration::from_secs(600)); // Every 10 minutes

    loop {
        interval.tick().await;
        log::debug!("Checking RSS feeds for instance {}...", client.name);

        let feeds = {
            let config = client.config();
            let config_guard = config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            config_guard.rss_config.feeds.clone()
        };

        for (idx, mut feed) in feeds.into_iter().enumerate() {
            if let Err(e) = process_feed(&client, &mut feed).await {
                log::error!("Error processing RSS feed '{}': {}", feed.name, e);
            } else {
                // Update history in config
                if let Ok(mut config_guard) = client.config().write() {
                    if let Some(f) = config_guard.rss_config.feeds.get_mut(idx) {
                        f.download_history = feed.download_history;
                    }
                }
            }
        }
    }
}

pub async fn process_feed(client: &Aria2Client, feed: &mut RSSFeed) -> Result<()> {
    let content = reqwest::get(&feed.url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;

    for item in channel.items() {
        let title = item.title().unwrap_or("Unknown Title");
        let link = item.link().and_then(|l| {
            if l.is_empty() {
                item.enclosure().map(|e| e.url())
            } else {
                Some(l)
            }
        });

        let id = item
            .guid()
            .map(|g| g.value())
            .or(item.link())
            .unwrap_or(title);

        if feed.has_downloaded(id) {
            continue;
        }

        if let Some(url) = link {
            if matches_filters(title, &feed.filters) {
                log::info!("RSS Match: Adding download '{}' from {}", title, feed.name);
                match client.add_uri(vec![url.to_string()], None).await {
                    Ok(gid) => {
                        log::info!("Added RSS download. GID: {}", gid);
                        feed.mark_downloaded(id.to_string());
                    }
                    Err(e) => {
                        log::error!("Failed to add RSS download '{}': {}", title, e);
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn matches_filters(title: &str, filters: &[RSSFilter]) -> bool {
    if filters.is_empty() {
        return true;
    }

    for filter in filters {
        match filter {
            RSSFilter::Keyword(k) => {
                if title.to_lowercase().contains(&k.to_lowercase()) {
                    return true;
                }
            }
            RSSFilter::Regex(r) => {
                if let Ok(re) = regex::RegexBuilder::new(r).case_insensitive(true).build() {
                    if re.is_match(title) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub struct AddRssFeedTool;

#[async_trait]
impl McpeTool for AddRssFeedTool {
    fn name(&self) -> String {
        "add_rss_feed".to_string()
    }

    fn description(&self) -> String {
        "Add a new RSS feed to monitor".to_string()
    }

    fn schema(&self) -> Result<Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL of the RSS feed"
                },
                "name": {
                    "type": "string",
                    "description": "A friendly name for the feed"
                },
                "filters": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional filters (keywords or 'regex:pattern')"
                }
            },
            "required": ["url", "name"]
        }))
    }

    async fn run(&self, client: &Aria2Client, args: Value) -> Result<Value> {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .context("Missing 'url'")?
            .to_string();
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .context("Missing 'name'")?
            .to_string();

        let mut filters = Vec::new();
        if let Some(f_arr) = args.get("filters").and_then(|v| v.as_array()) {
            for f in f_arr {
                if let Some(f_str) = f.as_str() {
                    if let Some(stripped) = f_str.strip_prefix("regex:") {
                        filters.push(RSSFilter::Regex(stripped.to_string()));
                    } else {
                        filters.push(RSSFilter::Keyword(f_str.to_string()));
                    }
                }
            }
        }

        let feed = RSSFeed {
            url: url.clone(),
            name: name.clone(),
            filters,
            download_history: std::collections::HashSet::new(),
        };

        let config = client.config();
        let mut config_guard = config
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to write config: {}", e))?;
        config_guard.rss_config.feeds.push(feed);

        Ok(json!({
            "status": "success",
            "message": format!("Added RSS feed '{}' ({})", name, url)
        }))
    }
}

pub struct ListRssFeedsTool;

#[async_trait]
impl McpeTool for ListRssFeedsTool {
    fn name(&self) -> String {
        "list_rss_feeds".to_string()
    }

    fn description(&self) -> String {
        "List all monitored RSS feeds".to_string()
    }

    fn schema(&self) -> Result<Value> {
        Ok(json!({
            "type": "object",
            "properties": {}
        }))
    }

    async fn run(&self, client: &Aria2Client, _args: Value) -> Result<Value> {
        let config = client.config();
        let config_guard = config
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;

        Ok(json!({
            "feeds": config_guard.rss_config.feeds
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_matches_filters_internal() {
        let filters = vec![
            RSSFilter::Keyword("work".to_string()),
            RSSFilter::Regex("test.*123".to_string()),
        ];

        assert!(matches_filters("This is WORK", &filters));
        assert!(matches_filters("testing something 123", &filters));
        assert!(!matches_filters("other", &filters));
        assert!(matches_filters("any", &[]));

        // Invalid regex
        let filters = vec![RSSFilter::Regex("[".to_string())];
        assert!(!matches_filters("any", &filters));
    }

    #[tokio::test]
    async fn test_process_feed_mock() {
        let mock_server = MockServer::start().await;

        // RSS Feed Mock
        let rss_content = r#"<?xml version="1.0" encoding="UTF-8"?>
            <rss version="2.0">
            <channel>
                <title>Test Feed</title>
                <item>
                    <title>Test Item 1</title>
                    <link>http://example.com/item1</link>
                    <guid>item1</guid>
                </item>
                <item>
                    <title>Other Item</title>
                    <link>http://example.com/item2</link>
                    <guid>item2</guid>
                </item>
            </channel>
            </rss>"#;

        Mock::given(method("GET"))
            .and(path("/rss"))
            .respond_with(ResponseTemplate::new(200).set_body_string(rss_content))
            .mount(&mock_server)
            .await;

        // Aria2 Mock
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0",
                "id": "1",
                "result": "gid123"
            })))
            .mount(&mock_server)
            .await;

        let mut config = Config::default();
        config.instances = vec![crate::config::Aria2Instance {
            name: "test".to_string(),
            rpc_url: format!("{}/jsonrpc", mock_server.uri()),
            rpc_secret: None,
        }];
        let client = Aria2Client::new_with_instance(config.clone(), config.instances[0].clone());

        let mut feed = RSSFeed {
            url: format!("{}/rss", mock_server.uri()),
            name: "test_feed".to_string(),
            filters: vec![RSSFilter::Keyword("Test".to_string())],
            download_history: std::collections::HashSet::new(),
        };

        process_feed(&client, &mut feed).await.unwrap();

        assert!(feed.has_downloaded("item1"));
        assert!(!feed.has_downloaded("item2"));
    }

    #[tokio::test]
    async fn test_process_feed_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rss"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = Config::default();
        let client = Aria2Client::new(config);

        let mut feed = RSSFeed {
            url: format!("{}/rss", mock_server.uri()),
            name: "test_feed".to_string(),
            filters: vec![],
            download_history: std::collections::HashSet::new(),
        };

        let result = process_feed(&client, &mut feed).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_rss_feed_tool() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let tool = AddRssFeedTool;
        let args = json!({
            "url": "http://example.com/rss",
            "name": "test",
            "filters": ["keyword", "regex:.*"]
        });
        let result = tool.run(&client, args).await.unwrap();
        assert_eq!(result["status"], "success");

        let config = client.config();
        let config_guard = config.read().unwrap();
        assert_eq!(config_guard.rss_config.feeds.len(), 1);
        assert_eq!(config_guard.rss_config.feeds[0].name, "test");
    }

    #[tokio::test]
    async fn test_list_rss_feeds_tool_empty() {
        let config = Config::default();
        let client = Aria2Client::new(config);
        let tool = ListRssFeedsTool;
        let result = tool.run(&client, json!({})).await.unwrap();
        assert_eq!(result["feeds"].as_array().unwrap().len(), 0);
    }
}
