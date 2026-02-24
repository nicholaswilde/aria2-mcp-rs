use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::aria2::Aria2Client;
use crate::config::{RSSFeed, RSSFilter};
use crate::tools::registry::McpeTool;

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
