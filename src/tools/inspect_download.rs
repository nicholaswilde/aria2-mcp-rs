use crate::aria2::Aria2Client;
use anyhow::Result;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use super::registry::McpeTool;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InspectDownloadParams {
    /// The GID of the download to inspect.
    pub gid: String,
    /// The action to perform.
    pub action: InspectAction,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InspectAction {
    /// Get detailed state, progress, and error info for a GID.
    Status,
    /// List all files in a task (useful for torrents).
    Files,
    /// List URIs associated with a task.
    Uris,
}

#[derive(Debug, schemars::JsonSchema)]
pub struct InspectDownloadTool;

#[async_trait]
impl McpeTool for InspectDownloadTool {
    fn name(&self) -> String {
        "inspect_download".to_string()
    }

    fn description(&self) -> String {
        "Inspect a download".to_string()
    }

    fn schema(&self) -> Result<serde_json::Value> {
        Ok(schemars::schema_for!(InspectDownloadParams).into())
    }

    async fn run(
        &self,
        client: &Aria2Client,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let params: InspectDownloadParams = serde_json::from_value(params)?;
        match params.action {
            InspectAction::Status => {
                let status = client.tell_status(&params.gid).await?;
                Ok(json!({ "status": status }))
            }
            InspectAction::Files => {
                let files = client.get_files(&params.gid).await?;
                Ok(json!({ "files": files }))
            }
            InspectAction::Uris => {
                let uris = client.get_uris(&params.gid).await?;
                Ok(json!({ "uris": uris }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aria2::Aria2Client;
    use crate::config::Config;

    #[tokio::test]
    async fn test_inspect_download_name() {
        let tool = InspectDownloadTool;
        assert_eq!(tool.name(), "inspect_download");
    }

    #[tokio::test]
    async fn test_inspect_download_run_status_error() {
        let tool = InspectDownloadTool;
        let client = Aria2Client::new(Config::default());
        let args = json!({ "gid": "dummy", "action": "status" });
        let result = tool.run(&client, args).await;
        assert!(result.is_err());
    }
}
