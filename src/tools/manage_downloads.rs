use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::aria2::Aria2Client;
use crate::tools::registry::Tool;

pub struct ManageDownloadsTool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageDownloadsArgs {
    /// Action to perform: add, pause, resume, remove, forcePause, forceRemove, move
    pub action: String,
    /// GID of the download (required for all actions except 'add')
    pub gid: Option<String>,
    /// URIs to add (required for 'add')
    pub uris: Option<Vec<String>>,
    /// New position (required for 'move')
    pub pos: Option<i32>,
    /// How to move: POS_SET, POS_CUR, POS_END (required for 'move')
    pub how: Option<String>,
    /// Options for adding the download
    pub options: Option<Value>,
}

#[async_trait]
impl Tool for ManageDownloadsTool {
    fn name(&self) -> &str {
        "manage_downloads"
    }

    fn description(&self) -> &str {
        "Monitor and manage aria2 downloads: add, pause, resume, remove, force-pause, force-remove, move"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["add", "pause", "resume", "remove", "forcePause", "forceRemove", "move"],
                    "description": "Action to perform"
                },
                "gid": {
                    "type": "string",
                    "description": "GID of the download"
                },
                "uris": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "URIs to add (for action='add')"
                },
                "pos": {
                    "type": "integer",
                    "description": "New position (for action='move')"
                },
                "how": {
                    "type": "string",
                    "enum": ["POS_SET", "POS_CUR", "POS_END"],
                    "description": "How to move (for action='move')"
                },
                "options": {
                    "type": "object",
                    "description": "Options for the added download (for action='add')"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, client: &Aria2Client, args: Value) -> Result<Value> {
        let args: ManageDownloadsArgs = serde_json::from_value(args)?;

        match args.action.as_str() {
            "add" => {
                let uris = args
                    .uris
                    .ok_or_else(|| anyhow::anyhow!("'uris' is required for action 'add'"))?;
                let gid = client.add_uri(uris, args.options).await?;
                Ok(json!({ "gid": gid }))
            }
            "pause" => {
                let gid = args
                    .gid
                    .ok_or_else(|| anyhow::anyhow!("'gid' is required for action 'pause'"))?;
                client.pause(&gid).await?;
                Ok(json!({ "status": "paused", "gid": gid }))
            }
            "resume" => {
                let gid = args
                    .gid
                    .ok_or_else(|| anyhow::anyhow!("'gid' is required for action 'resume'"))?;
                client.unpause(&gid).await?;
                Ok(json!({ "status": "resumed", "gid": gid }))
            }
            "remove" => {
                let gid = args
                    .gid
                    .ok_or_else(|| anyhow::anyhow!("'gid' is required for action 'remove'"))?;
                client.remove(&gid).await?;
                Ok(json!({ "status": "removed", "gid": gid }))
            }
            "forcePause" => {
                let gid = args
                    .gid
                    .ok_or_else(|| anyhow::anyhow!("'gid' is required for action 'forcePause'"))?;
                client.force_pause(&gid).await?;
                Ok(json!({ "status": "force-paused", "gid": gid }))
            }
            "forceRemove" => {
                let gid = args
                    .gid
                    .ok_or_else(|| anyhow::anyhow!("'gid' is required for action 'forceRemove'"))?;
                client.force_remove(&gid).await?;
                Ok(json!({ "status": "force-removed", "gid": gid }))
            }
            "move" => {
                let gid = args
                    .gid
                    .ok_or_else(|| anyhow::anyhow!("'gid' is required for action 'move'"))?;
                let pos = args
                    .pos
                    .ok_or_else(|| anyhow::anyhow!("'pos' is required for action 'move'"))?;
                let how = args
                    .how
                    .ok_or_else(|| anyhow::anyhow!("'how' is required for action 'move'"))?;
                let new_pos = client.move_position(&gid, pos, &how).await?;
                Ok(json!({ "newPosition": new_pos, "gid": gid }))
            }
            _ => Err(anyhow::anyhow!("Unknown action: {}", args.action)),
        }
    }
}
