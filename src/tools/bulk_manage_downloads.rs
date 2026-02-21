use anyhow::Result;
use async_trait::async_trait;
use futures_util::future::join_all;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::aria2::Aria2Client;
use crate::tools::registry::McpeTool;

pub struct BulkManageDownloadsTool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkManageDownloadsArgs {
    /// Action to perform: pause, resume, remove, forcePause, forceRemove
    pub action: String,
    /// List of GIDs to perform the action on
    pub gids: Vec<String>,
}

#[async_trait]
impl McpeTool for BulkManageDownloadsTool {
    fn name(&self) -> String {
        "bulk_manage_downloads".to_string()
    }

    fn description(&self) -> String {
        "Perform bulk actions on multiple aria2 downloads: pause, resume, remove, force-pause, force-remove".to_string()
    }

    fn schema(&self) -> Result<Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["pause", "resume", "remove", "forcePause", "forceRemove"],
                    "description": "Action to perform"
                },
                "gids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of GIDs of the downloads"
                }
            },
            "required": ["action", "gids"]
        }))
    }

    async fn run(&self, client: &Aria2Client, args: Value) -> Result<Value> {
        let args: BulkManageDownloadsArgs = serde_json::from_value(args)?;

        let futures = args.gids.iter().map(|gid| {
            let client = client.clone();
            let action = args.action.clone();
            let gid = gid.clone();
            async move {
                let result: Result<()> = match action.as_str() {
                    "pause" => client.pause(&gid).await,
                    "resume" => client.unpause(&gid).await,
                    "remove" => client.remove(&gid).await,
                    "forcePause" => client.force_pause(&gid).await,
                    "forceRemove" => client.force_remove(&gid).await,
                    _ => Err(anyhow::anyhow!("Unknown action: {}", action)),
                };
                (gid, result)
            }
        });

        let results = join_all(futures).await;

        let mut success_count = 0;
        let mut failure_count = 0;
        let mut details = Vec::new();

        for (gid, res) in results {
            match res {
                Ok(_) => {
                    success_count += 1;
                    details.push(json!({ "gid": gid, "status": "success" }));
                }
                Err(e) => {
                    failure_count += 1;
                    details
                        .push(json!({ "gid": gid, "status": "error", "message": e.to_string() }));
                }
            }
        }

        Ok(json!({
            "success_count": success_count,
            "failure_count": failure_count,
            "results": details
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_bulk_manage_downloads_name() {
        let tool = BulkManageDownloadsTool;
        assert_eq!(tool.name(), "bulk_manage_downloads");
    }

    #[tokio::test]
    async fn test_bulk_manage_downloads_schema() {
        let tool = BulkManageDownloadsTool;
        let schema = tool.schema().unwrap();
        assert_eq!(schema["type"], "object");
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("action")));
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("gids")));
    }

    #[tokio::test]
    async fn test_bulk_manage_downloads_run_error() {
        let tool = BulkManageDownloadsTool;
        let client = Aria2Client::new(Config::default());
        // Should fail because it can't connect to aria2 (or rather, the actions will fail)
        let args = json!({
            "action": "pause",
            "gids": ["1"]
        });
        let result = tool.run(&client, args).await.unwrap();
        assert_eq!(result["success_count"], 0);
        assert_eq!(result["failure_count"], 1);
    }

    #[tokio::test]
    async fn test_bulk_manage_downloads_invalid_action() {
        let tool = BulkManageDownloadsTool;
        let client = Aria2Client::new(Config::default());
        let args = json!({
            "action": "invalid",
            "gids": ["1"]
        });
        let result = tool.run(&client, args).await.unwrap();
        assert_eq!(result["success_count"], 0);
        assert_eq!(result["failure_count"], 1);
        assert!(result["results"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Unknown action"));
    }
}
