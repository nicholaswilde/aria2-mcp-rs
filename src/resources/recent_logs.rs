use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::fs;

use crate::aria2::Aria2Client;
use crate::resources::McpResource;

pub struct RecentLogsResource {
    log_path: String,
    lines: usize,
}

impl RecentLogsResource {
    pub fn new(log_path: String, lines: usize) -> Self {
        Self { log_path, lines }
    }
}

#[async_trait]
impl McpResource for RecentLogsResource {
    fn uri(&self) -> String {
        // For simplicity, URI doesn't include path/lines. These are configured when the resource is created.
        // In a more advanced setup, these could be query parameters in the URI.
        "aria2://logs/recent".to_string()
    }

    fn name(&self) -> String {
        "Recent Logs".to_string()
    }

    fn description(&self) -> Option<String> {
        Some(format!(
            "Last {} lines of the application log file at {}.",
            self.lines, self.log_path
        ))
    }

    fn mime_type(&self) -> Option<String> {
        Some("text/plain".to_string())
    }

    async fn read(&self, _client: &Aria2Client) -> Result<Value> {
        // This resource reads from the filesystem, not directly from an Aria2Client.
        // The client parameter is kept for trait consistency.

        let log_content = fs::read_to_string(&self.log_path)
            .map_err(|e| anyhow::anyhow!("Failed to read log file '{}': {}", self.log_path, e))?;

        // Collect the last N lines. Using .rev().take(N).rev() to maintain order.
        let lines: Vec<String> = log_content
            .lines()
            .rev()
            .take(self.lines)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(String::from)
            .collect();

        Ok(serde_json::json!(lines.join("\n")))
    }
}
