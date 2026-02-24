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
