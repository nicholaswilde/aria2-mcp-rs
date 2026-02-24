use super::McpeTool;
use crate::aria2::Aria2Client;
use anyhow::{Context, Result};
use serde_json::json;

pub struct PurgePolicyTool;

#[async_trait::async_trait]
impl McpeTool for PurgePolicyTool {
    fn name(&self) -> String {
        "purge_policy".to_string()
    }

    fn description(&self) -> String {
        "View or update the automated queue purging policy".to_string()
    }

    fn schema(&self) -> Result<serde_json::Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["get_policy", "update_policy", "exclude_gid", "remove_exclusion"],
                    "description": "The action to perform"
                },
                "enabled": {
                    "type": "boolean",
                    "description": "Enable or disable automated purging"
                },
                "intervalSecs": {
                    "type": "integer",
                    "description": "How often to run the purge check (in seconds)"
                },
                "minAgeSecs": {
                    "type": "integer",
                    "description": "Minimum age of a stopped/errored download before it can be purged (in seconds)"
                },
                "gid": {
                    "type": "string",
                    "description": "GID to exclude from or remove from purging exclusions"
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

        let config = client.config();

        match action {
            "get_policy" => {
                let config_guard = config
                    .read()
                    .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
                Ok(json!({ "policy": config_guard.purge_config }))
            }
            "update_policy" => {
                let mut config_guard = config
                    .write()
                    .map_err(|e| anyhow::anyhow!("Failed to write config: {}", e))?;

                if let Some(enabled) = args.get("enabled").and_then(|v| v.as_bool()) {
                    config_guard.purge_config.enabled = enabled;
                }
                if let Some(interval) = args.get("intervalSecs").and_then(|v| v.as_u64()) {
                    config_guard.purge_config.interval_secs = interval;
                }
                if let Some(min_age) = args.get("minAgeSecs").and_then(|v| v.as_u64()) {
                    config_guard.purge_config.min_age_secs = min_age;
                }

                Ok(
                    json!({ "status": "success", "message": "Purge policy updated", "policy": config_guard.purge_config }),
                )
            }
            "exclude_gid" => {
                let gid = args
                    .get("gid")
                    .and_then(|v| v.as_str())
                    .context("Missing 'gid' for exclude_gid")?;

                let mut config_guard = config
                    .write()
                    .map_err(|e| anyhow::anyhow!("Failed to write config: {}", e))?;

                config_guard
                    .purge_config
                    .excluded_gids
                    .insert(gid.to_string());

                Ok(
                    json!({ "status": "success", "message": format!("GID {} excluded from purging", gid) }),
                )
            }
            "remove_exclusion" => {
                let gid = args
                    .get("gid")
                    .and_then(|v| v.as_str())
                    .context("Missing 'gid' for remove_exclusion")?;

                let mut config_guard = config
                    .write()
                    .map_err(|e| anyhow::anyhow!("Failed to write config: {}", e))?;

                let removed = config_guard.purge_config.excluded_gids.remove(gid);

                if removed {
                    Ok(
                        json!({ "status": "success", "message": format!("GID {} removed from purging exclusions", gid) }),
                    )
                } else {
                    Ok(
                        json!({ "status": "error", "message": format!("GID {} was not in exclusions list", gid) }),
                    )
                }
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
    async fn test_purge_policy_name() {
        let tool = PurgePolicyTool;
        assert_eq!(tool.name(), "purge_policy");
    }

    #[tokio::test]
    async fn test_purge_policy_run_get() {
        let tool = PurgePolicyTool;
        let client = Aria2Client::new(Config::default());
        let args = json!({ "action": "get_policy" });
        let result = tool.run(&client, args).await.unwrap();
        assert!(!result["policy"]["enabled"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_purge_policy_run_update() {
        let tool = PurgePolicyTool;
        let client = Aria2Client::new(Config::default());
        let args = json!({
            "action": "update_policy",
            "enabled": true,
            "intervalSecs": 100
        });
        let result = tool.run(&client, args).await.unwrap();
        assert!(result["policy"]["enabled"].as_bool().unwrap());
        assert_eq!(result["policy"]["interval_secs"], 100);
    }

    #[tokio::test]
    async fn test_purge_policy_exclude_gid() {
        let tool = PurgePolicyTool;
        let client = Aria2Client::new(Config::default());
        let args = json!({
            "action": "exclude_gid",
            "gid": "test-gid"
        });
        let result = tool.run(&client, args).await.unwrap();
        assert_eq!(result["status"], "success");

        let args = json!({ "action": "get_policy" });
        let result = tool.run(&client, args).await.unwrap();
        let exclusions = result["policy"]["excluded_gids"].as_array().unwrap();
        assert!(exclusions.contains(&json!("test-gid")));
    }
}
