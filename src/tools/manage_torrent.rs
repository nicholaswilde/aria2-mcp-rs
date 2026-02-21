use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::aria2::Aria2Client;
use crate::tools::registry::McpeTool;

pub struct ManageTorrentTool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageTorrentArgs {
    /// Action to perform: getPeers, changeFiles, addTrackers
    pub action: String,
    /// GID of the download
    pub gid: String,
    /// Selected files (comma-separated indices, for changeFiles)
    pub selected_files: Option<String>,
    /// Trackers (comma-separated URIs, for addTrackers)
    pub trackers: Option<String>,
}

#[async_trait]
impl McpeTool for ManageTorrentTool {
    fn name(&self) -> String {
        "manage_torrent".to_string()
    }

    fn description(&self) -> String {
        "Manage BitTorrent-specific settings: get peers, select files, and update trackers"
            .to_string()
    }

    fn schema(&self) -> Result<Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["getPeers", "changeFiles", "addTrackers"],
                    "description": "Action to perform"
                },
                "gid": {
                    "type": "string",
                    "description": "GID of the torrent download"
                },
                "selectedFiles": {
                    "type": "string",
                    "description": "Comma-separated list of file indices to download (e.g., '1,2,5')"
                },
                "trackers": {
                    "type": "string",
                    "description": "Comma-separated list of tracker URIs"
                }
            },
            "required": ["action", "gid"]
        }))
    }

    async fn run(&self, client: &Aria2Client, args: Value) -> Result<Value> {
        let args: ManageTorrentArgs = serde_json::from_value(args)?;

        match args.action.as_str() {
            "getPeers" => {
                let peers = client.get_peers(&args.gid).await?;
                Ok(peers)
            }
            "changeFiles" => {
                let files = args
                    .selected_files
                    .ok_or_else(|| anyhow::anyhow!("selectedFiles is required for changeFiles"))?;
                client
                    .change_option(&args.gid, json!({ "select-file": files }))
                    .await?;
                Ok(json!({ "status": "success", "gid": args.gid, "selectedFiles": files }))
            }
            "addTrackers" => {
                let trackers = args
                    .trackers
                    .ok_or_else(|| anyhow::anyhow!("trackers is required for addTrackers"))?;
                client
                    .change_option(&args.gid, json!({ "bt-tracker": trackers }))
                    .await?;
                Ok(json!({ "status": "success", "gid": args.gid, "trackers": trackers }))
            }
            _ => Err(anyhow::anyhow!("Unknown action: {}", args.action)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_manage_torrent_name() {
        let tool = ManageTorrentTool;
        assert_eq!(tool.name(), "manage_torrent");
    }

    #[tokio::test]
    async fn test_manage_torrent_schema() {
        let tool = ManageTorrentTool;
        let schema = tool.schema().unwrap();
        assert_eq!(schema["type"], "object");
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("action")));
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("gid")));
    }

    #[tokio::test]
    async fn test_manage_torrent_run_error() {
        let tool = ManageTorrentTool;
        let client = Aria2Client::new(Config::default());
        let args = json!({
            "action": "getPeers",
            "gid": "1"
        });
        let result = tool.run(&client, args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_manage_torrent_missing_args() {
        let tool = ManageTorrentTool;
        let client = Aria2Client::new(Config::default());

        // Missing selectedFiles for changeFiles
        let args = json!({
            "action": "changeFiles",
            "gid": "1"
        });
        let result = tool.run(&client, args).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("selectedFiles is required"));

        // Missing trackers for addTrackers
        let args = json!({
            "action": "addTrackers",
            "gid": "1"
        });
        let result = tool.run(&client, args).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("trackers is required"));
    }
}
