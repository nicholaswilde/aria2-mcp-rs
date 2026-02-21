use super::McpeTool;
use crate::Aria2Client;
use anyhow::{Context, Result};
use serde_json::json;

pub struct ConfigureAria2Tool;

#[async_trait::async_trait]
impl McpeTool for ConfigureAria2Tool {
    fn name(&self) -> String {
        "configure_aria2".to_string()
    }

    fn description(&self) -> String {
        "Retrieve or update aria2 configuration (global or per-download)".to_string()
    }

    fn schema(&self) -> Result<serde_json::Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["get_global", "change_global", "get_local", "change_local"],
                    "description": "The configuration action to perform"
                },
                "gid": {
                    "type": "string",
                    "description": "The GID of the download (required for get_local/change_local)"
                },
                "options": {
                    "type": "object",
                    "description": "Key-value pairs of options to set (required for change_global/change_local)"
                }
            },
            "required": ["action"]
        }))
    }

    async fn run(
        &self,
        client: &Aria2Client,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .context("Missing or invalid 'action' argument")?;

        match action {
            "get_global" => client.get_global_option().await,
            "change_global" => {
                let options = args
                    .get("options")
                    .context("Missing 'options' for change_global")?
                    .clone();
                client.change_global_option(options).await?;
                Ok(json!({"status": "success", "message": "Global options updated"}))
            }
            "get_local" => {
                let gid = args
                    .get("gid")
                    .and_then(|v| v.as_str())
                    .context("Missing 'gid' for get_local")?;
                client.get_option(gid).await
            }
            "change_local" => {
                let gid = args
                    .get("gid")
                    .and_then(|v| v.as_str())
                    .context("Missing 'gid' for change_local")?;
                let options = args
                    .get("options")
                    .context("Missing 'options' for change_local")?
                    .clone();
                client.change_option(gid, options).await?;
                Ok(
                    json!({"status": "success", "message": format!("Options updated for GID {}", gid)}),
                )
            }
            _ => Err(anyhow::anyhow!("Unknown action: {}", action)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_configure_aria2_name() {
        let tool = ConfigureAria2Tool;
        assert_eq!(tool.name(), "configure_aria2");
    }

    #[tokio::test]
    async fn test_configure_aria2_run_missing_action() {
        let tool = ConfigureAria2Tool;
        let client = Aria2Client::new(Config::default());
        let args = json!({});
        let result = tool.run(&client, args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_configure_aria2_run_unknown_action() {
        let tool = ConfigureAria2Tool;
        let client = Aria2Client::new(Config::default());
        let args = json!({ "action": "unknown" });
        let result = tool.run(&client, args).await;
        assert!(result.is_err());
    }
}
