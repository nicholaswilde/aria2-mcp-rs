use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::aria2::Aria2Client;
use crate::tools::registry::McpeTool;

pub struct ListDownloadFilesTool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDownloadFilesArgs {
    /// Relative path within the download directory
    pub path: String,
    /// Maximum depth for listing (optional)
    pub max_depth: Option<u32>,
}

#[async_trait]
impl McpeTool for ListDownloadFilesTool {
    fn name(&self) -> String {
        "list_download_files".to_string()
    }

    fn description(&self) -> String {
        "List files and directories within a specified path relative to the download directory."
            .to_string()
    }

    fn schema(&self) -> Result<Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path within the download directory"
                },
                "maxDepth": {
                    "type": "integer",
                    "description": "Maximum depth for recursive listing (optional, default is 1)",
                    "minimum": 1
                }
            },
            "required": ["path"]
        }))
    }

    async fn run(&self, _client: &Aria2Client, _args: Value) -> Result<Value> {
        // Implementation will come in next phase
        Ok(json!({ "status": "todo" }))
    }
}
