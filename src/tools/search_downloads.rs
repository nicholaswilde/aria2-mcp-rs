use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::aria2::Aria2Client;
use crate::tools::registry::McpeTool;

pub struct SearchDownloadsTool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchDownloadsArgs {
    /// Substring to search for in filenames or URIs
    pub query: Option<String>,
    /// Filter by status: active, waiting, paused, error, complete, removed
    pub status: Option<String>,
    /// Keys to return for each match
    pub keys: Option<Vec<String>>,
}

#[async_trait]
impl McpeTool for SearchDownloadsTool {
    fn name(&self) -> String {
        "search_downloads".to_string()
    }

    fn description(&self) -> String {
        "Search and filter downloads by filename, URI, or status".to_string()
    }

    fn schema(&self) -> Result<Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Substring to search for in filenames or URIs"
                },
                "status": {
                    "type": "string",
                    "enum": ["active", "waiting", "paused", "error", "complete", "removed"],
                    "description": "Filter by status"
                },
                "keys": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional keys to return for each task"
                }
            }
        }))
    }

    async fn run(&self, client: &Aria2Client, args: Value) -> Result<Value> {
        let args: SearchDownloadsArgs = serde_json::from_value(args)?;
        
        let mut all_downloads = Vec::new();
        
        match args.status.as_deref() {
            Some("active") => {
                let active = client.tell_active(args.keys.clone()).await?;
                if let Some(arr) = active.as_array() {
                    all_downloads.extend(arr.clone());
                }
            }
            Some("waiting") | Some("paused") => {
                let waiting = client.tell_waiting(0, 1000, args.keys.clone()).await?;
                if let Some(arr) = waiting.as_array() {
                    all_downloads.extend(arr.clone());
                }
            }
            Some("error") | Some("complete") | Some("removed") => {
                let stopped = client.tell_stopped(0, 1000, args.keys.clone()).await?;
                if let Some(arr) = stopped.as_array() {
                    all_downloads.extend(arr.clone());
                }
            }
            _ => {
                let active = client.tell_active(args.keys.clone()).await?;
                if let Some(arr) = active.as_array() {
                    all_downloads.extend(arr.clone());
                }
                let waiting = client.tell_waiting(0, 1000, args.keys.clone()).await?;
                if let Some(arr) = waiting.as_array() {
                    all_downloads.extend(arr.clone());
                }
                let stopped = client.tell_stopped(0, 1000, args.keys.clone()).await?;
                if let Some(arr) = stopped.as_array() {
                    all_downloads.extend(arr.clone());
                }
            }
        }

        let filtered = self.filter_downloads(all_downloads, &args);
        Ok(json!(filtered))
    }
}

impl SearchDownloadsTool {
    fn filter_downloads(&self, downloads: Vec<Value>, args: &SearchDownloadsArgs) -> Vec<Value> {
        downloads.into_iter().filter(|item| {
            // Filter by status if provided (paused is special as it's within waiting)
            if let Some(status_filter) = &args.status {
                let item_status = item.get("status").and_then(|s| s.as_str()).unwrap_or("");
                if status_filter == "paused" {
                    let paused = item.get("paused").and_then(|p| {
                        p.as_str().map(|s| s == "true")
                            .or_else(|| p.as_bool())
                    }).unwrap_or(false);
                    if item_status != "waiting" || !paused {
                        return false;
                    }
                } else if item_status != status_filter {
                    return false;
                }
            }
            
            // Filter by query if provided
            if let Some(query) = &args.query {
                let query = query.to_lowercase();
                
                // Check files for path/name
                let files_match = if let Some(files) = item.get("files").and_then(|f| f.as_array()) {
                    files.iter().any(|file| {
                        file.get("path").and_then(|p| p.as_str()).map(|p| p.to_lowercase().contains(&query)).unwrap_or(false) ||
                        file.get("uris").and_then(|u| u.as_array()).map(|uris| {
                            uris.iter().any(|uri| {
                                uri.get("uri").and_then(|u| u.as_str()).map(|u| u.to_lowercase().contains(&query)).unwrap_or(false)
                            })
                        }).unwrap_or(false)
                    })
                } else {
                    false
                };
                
                if files_match {
                    return true;
                }
                
                // Check bittorrent name if available
                if let Some(bt) = item.get("bittorrent") {
                    if let Some(info) = bt.get("info") {
                        if let Some(name) = info.get("name").and_then(|n| n.as_str()) {
                            if name.to_lowercase().contains(&query) {
                                return true;
                            }
                        }
                    }
                }
                
                return false;
            }
            
            true
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aria2::Aria2Client;
    use crate::config::Config;

    #[tokio::test]
    async fn test_search_downloads_name() {
        let tool = SearchDownloadsTool;
        assert_eq!(tool.name(), "search_downloads");
    }

    #[tokio::test]
    async fn test_search_downloads_schema() {
        let tool = SearchDownloadsTool;
        let schema = tool.schema().unwrap();
        assert_eq!(schema["type"], "object");
    }

    #[tokio::test]
    async fn test_search_downloads_run_error() {
        let tool = SearchDownloadsTool;
        let client = Aria2Client::new(Config::default());
        // Should fail because it can't connect to aria2
        let args = json!({ "status": "active" });
        let result = tool.run(&client, args).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_downloads_by_status() {
        let tool = SearchDownloadsTool;
        let downloads = vec![
            json!({ "gid": "1", "status": "active" }),
            json!({ "gid": "2", "status": "waiting", "paused": "false" }),
            json!({ "gid": "3", "status": "waiting", "paused": "true" }),
            json!({ "gid": "4", "status": "complete" }),
        ];

        // Filter by active
        let args = SearchDownloadsArgs { query: None, status: Some("active".to_string()), keys: None };
        let results = tool.filter_downloads(downloads.clone(), &args);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["gid"], "1");

        // Filter by paused
        let args = SearchDownloadsArgs { query: None, status: Some("paused".to_string()), keys: None };
        let results = tool.filter_downloads(downloads.clone(), &args);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["gid"], "3");
    }

    #[test]
    fn test_filter_downloads_by_query_path() {
        let tool = SearchDownloadsTool;
        let downloads = vec![
            json!({ 
                "gid": "1", 
                "files": [{ "path": "/downloads/movie.mp4", "uris": [] }] 
            }),
            json!({ 
                "gid": "2", 
                "files": [{ "path": "/downloads/other.txt", "uris": [] }] 
            }),
        ];

        let args = SearchDownloadsArgs { query: Some("movie".to_string()), status: None, keys: None };
        let results = tool.filter_downloads(downloads, &args);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["gid"], "1");
    }

    #[test]
    fn test_filter_downloads_by_query_uri() {
        let tool = SearchDownloadsTool;
        let downloads = vec![
            json!({ 
                "gid": "1", 
                "files": [{ "path": "", "uris": [{ "uri": "https://example.com/file.zip" }] }] 
            }),
        ];

        let args = SearchDownloadsArgs { query: Some("example".to_string()), status: None, keys: None };
        let results = tool.filter_downloads(downloads, &args);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["gid"], "1");
    }

    #[test]
    fn test_filter_downloads_by_query_bt_name() {
        let tool = SearchDownloadsTool;
        let downloads = vec![
            json!({ 
                "gid": "1", 
                "bittorrent": { "info": { "name": "Linux ISO" } }
            }),
        ];

        let args = SearchDownloadsArgs { query: Some("linux".to_string()), status: None, keys: None };
        let results = tool.filter_downloads(downloads, &args);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["gid"], "1");
    }
}
