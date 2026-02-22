use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::aria2::Aria2Client;
use crate::tools::registry::McpeTool;

pub struct ManageAllInstancesTool;

#[async_trait]
impl McpeTool for ManageAllInstancesTool {
    fn name(&self) -> String {
        "manage_all_instances".to_string()
    }

    fn description(&self) -> String {
        "Perform bulk operations (pause, resume, stop) on all configured aria2 instances at once.".to_string()
    }

    fn schema(&self) -> Result<Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["pause", "resume", "purge"],
                    "description": "The action to perform on all instances."
                }
            },
            "required": ["action"]
        }))
    }

    async fn run(&self, _client: &Aria2Client, _args: Value) -> Result<Value> {
        Err(anyhow::anyhow!("This tool requires multiple clients and cannot be run with a single client reference. Use McpHandler's multi-client routing."))
    }

    async fn run_multi(&self, clients: &[Arc<Aria2Client>], args: Value) -> Result<Value> {
        let action = args["action"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing action"))?;

        let mut results = Vec::new();

        for client in clients {
            let res = match action {
                "pause" => client.pause_all().await,
                "resume" => client.unpause_all().await,
                "purge" => client.purge_download_result().await,
                _ => Err(anyhow::anyhow!("Invalid action: {}", action)),
            };

            results.push(json!({
                "instance": client.name,
                "status": if res.is_ok() { "ok" } else { "error" },
                "message": res.err().map(|e| e.to_string()).unwrap_or_else(|| "Success".to_string())
            }));
        }

        Ok(json!({
            "action": action,
            "results": results
        }))
    }
}
