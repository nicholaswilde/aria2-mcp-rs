use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

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

        let file = std::fs::File::open(&self.log_path)
            .map_err(|e| anyhow::anyhow!("Failed to read log file '{}': {}", self.log_path, e))?;

        let mut reader = std::io::BufReader::new(file);
        use std::io::{Read, Seek, SeekFrom};

        // Seek to the end
        let file_size = reader.seek(SeekFrom::End(0))?;
        if file_size == 0 {
            return Ok(serde_json::json!(""));
        }

        // We'll read in chunks from the end to find the last N lines
        let chunk_size = 1024;
        let mut pos = file_size;
        let mut buffer = Vec::new();
        let mut line_count = 0;

        while line_count <= self.lines && pos > 0 {
            let to_read = std::cmp::min(pos, chunk_size as u64);
            pos -= to_read;
            reader.seek(SeekFrom::Start(pos))?;

            let mut chunk = vec![0; to_read as usize];
            reader.read_exact(&mut chunk)?;

            // Prepend chunk to buffer
            let mut new_buffer = chunk;
            new_buffer.extend_from_slice(&buffer);
            buffer = new_buffer;

            // Count newlines in buffer (from the end)
            line_count = 0;
            for &byte in buffer.iter().rev() {
                if byte == b'\n' {
                    line_count += 1;
                }
            }
        }

        // Convert buffer to lines and take the last N
        let content = String::from_utf8_lossy(&buffer);
        let mut all_lines: Vec<&str> = content.lines().collect();
        if all_lines.len() > self.lines {
            all_lines = all_lines.drain(all_lines.len() - self.lines..).collect();
        }

        Ok(serde_json::json!(all_lines.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_recent_logs_resource_read() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "Line 1")?;
        writeln!(file, "Line 2")?;
        writeln!(file, "Line 3")?;
        writeln!(file, "Line 4")?;
        writeln!(file, "Line 5")?;

        let log_path = file.path().to_str().unwrap().to_string();
        let resource = RecentLogsResource::new(log_path, 3);

        let client = Aria2Client::new(crate::config::Config::default());
        let result = resource.read(&client).await?;

        let expected = "Line 3\nLine 4\nLine 5";
        assert_eq!(result.as_str().unwrap(), expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_recent_logs_resource_small_file() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "Line 1")?;
        writeln!(file, "Line 2")?;

        let log_path = file.path().to_str().unwrap().to_string();
        let resource = RecentLogsResource::new(log_path, 10);

        let client = Aria2Client::new(crate::config::Config::default());
        let result = resource.read(&client).await?;

        let expected = "Line 1\nLine 2";
        assert_eq!(result.as_str().unwrap(), expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_recent_logs_resource_empty_file() -> Result<()> {
        let file = NamedTempFile::new()?;

        let log_path = file.path().to_str().unwrap().to_string();
        let resource = RecentLogsResource::new(log_path, 10);

        let client = Aria2Client::new(crate::config::Config::default());
        let result = resource.read(&client).await?;

        assert_eq!(result.as_str().unwrap(), "");

        Ok(())
    }

    #[tokio::test]
    async fn test_recent_logs_resource_file_not_found() -> Result<()> {
        let resource = RecentLogsResource::new("non_existent_file.log".to_string(), 10);
        let client = Aria2Client::new(crate::config::Config::default());
        let result = resource.read(&client).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read log file"));

        Ok(())
    }
}
