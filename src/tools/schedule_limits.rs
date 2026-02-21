use super::McpeTool;
use crate::aria2::Aria2Client;
use crate::config::{BandwidthProfile, BandwidthSchedule};
use anyhow::{Context, Result};
use serde_json::json;

pub struct ScheduleLimitsTool;

#[async_trait::async_trait]
impl McpeTool for ScheduleLimitsTool {
    fn name(&self) -> String {
        "schedule_limits".to_string()
    }

    fn description(&self) -> String {
        "Manage bandwidth speed profiles and schedules".to_string()
    }

    fn schema(&self) -> Result<serde_json::Value> {
        Ok(json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list_profiles", "set_profile", "add_profile", "remove_profile", "list_schedules", "add_schedule", "remove_schedule"],
                    "description": "The action to perform"
                },
                "profile_name": {
                    "type": "string",
                    "description": "The name of the speed profile"
                },
                "max_download": {
                    "type": "string",
                    "description": "Max overall download limit (e.g., '1M', '500K', '0' for unlimited)"
                },
                "max_upload": {
                    "type": "string",
                    "description": "Max overall upload limit"
                },
                "schedule": {
                    "type": "object",
                    "properties": {
                        "day": {
                            "type": "string",
                            "enum": ["daily", "mon", "tue", "wed", "thu", "fri", "sat", "sun"],
                            "description": "Day of the week"
                        },
                        "start_time": {
                            "type": "string",
                            "description": "Start time in HH:MM format"
                        },
                        "end_time": {
                            "type": "string",
                            "description": "End time in HH:MM format"
                        }
                    },
                    "required": ["day", "start_time", "end_time"]
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
            "list_profiles" => {
                let config = client.config();
                let config_guard = config
                    .read()
                    .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
                let profiles = config_guard.bandwidth_profiles.clone();
                Ok(json!({ "profiles": profiles }))
            }
            "add_profile" => {
                let name = args
                    .get("profile_name")
                    .and_then(|v| v.as_str())
                    .context("Missing 'profile_name'")?;
                let max_download = args
                    .get("max_download")
                    .and_then(|v| v.as_str())
                    .context("Missing 'max_download'")?;
                let max_upload = args
                    .get("max_upload")
                    .and_then(|v| v.as_str())
                    .context("Missing 'max_upload'")?;

                let profile = BandwidthProfile {
                    max_download: max_download.to_string(),
                    max_upload: max_upload.to_string(),
                };

                let config = client.config();
                let mut config_guard = config
                    .write()
                    .map_err(|e| anyhow::anyhow!("Failed to write config: {}", e))?;
                config_guard
                    .bandwidth_profiles
                    .insert(name.to_string(), profile);

                Ok(json!({ "status": "success", "message": format!("Profile '{}' added", name) }))
            }
            "remove_profile" => {
                let name = args
                    .get("profile_name")
                    .and_then(|v| v.as_str())
                    .context("Missing 'profile_name'")?;

                let config = client.config();
                let mut config_guard = config
                    .write()
                    .map_err(|e| anyhow::anyhow!("Failed to write config: {}", e))?;
                config_guard.bandwidth_profiles.remove(name);

                Ok(json!({ "status": "success", "message": format!("Profile '{}' removed", name) }))
            }
            "list_schedules" => {
                let config = client.config();
                let config_guard = config
                    .read()
                    .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
                let schedules = config_guard.bandwidth_schedules.clone();
                Ok(json!({ "schedules": schedules }))
            }
            "add_schedule" => {
                let profile_name = args
                    .get("profile_name")
                    .and_then(|v| v.as_str())
                    .context("Missing 'profile_name'")?;
                let schedule_val = args.get("schedule").context("Missing 'schedule'")?;
                let day = schedule_val
                    .get("day")
                    .and_then(|v| v.as_str())
                    .context("Missing 'day' in schedule")?;
                let start_time = schedule_val
                    .get("start_time")
                    .and_then(|v| v.as_str())
                    .context("Missing 'start_time' in schedule")?;
                let end_time = schedule_val
                    .get("end_time")
                    .and_then(|v| v.as_str())
                    .context("Missing 'end_time' in schedule")?;

                let schedule = BandwidthSchedule {
                    day: day.to_string(),
                    start_time: start_time.to_string(),
                    end_time: end_time.to_string(),
                    profile_name: profile_name.to_string(),
                };

                let config = client.config();
                let mut config_guard = config
                    .write()
                    .map_err(|e| anyhow::anyhow!("Failed to write config: {}", e))?;

                // Validate profile exists
                if !config_guard.bandwidth_profiles.contains_key(profile_name) {
                    return Err(anyhow::anyhow!("Profile '{}' does not exist", profile_name));
                }

                config_guard.bandwidth_schedules.push(schedule);

                Ok(json!({ "status": "success", "message": "Schedule added" }))
            }
            "remove_schedule" => {
                let index = args
                    .get("index")
                    .and_then(|v| v.as_u64())
                    .context("Missing or invalid 'index'")? as usize;

                let config = client.config();
                let mut config_guard = config
                    .write()
                    .map_err(|e| anyhow::anyhow!("Failed to write config: {}", e))?;

                if index >= config_guard.bandwidth_schedules.len() {
                    return Err(anyhow::anyhow!("Schedule index out of bounds"));
                }

                config_guard.bandwidth_schedules.remove(index);

                Ok(json!({ "status": "success", "message": "Schedule removed" }))
            }
            "set_profile" => {
                let name = args
                    .get("profile_name")
                    .and_then(|v| v.as_str())
                    .context("Missing 'profile_name'")?;

                let profile = {
                    let config = client.config();
                    let config_guard = config
                        .read()
                        .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
                    config_guard
                        .bandwidth_profiles
                        .get(name)
                        .cloned()
                        .context(format!("Profile '{}' not found", name))?
                };

                let options = json!({
                    "max-overall-download-limit": profile.max_download,
                    "max-overall-upload-limit": profile.max_upload,
                });

                client.change_global_option(options).await?;

                Ok(
                    json!({ "status": "success", "message": format!("Profile '{}' activated", name) }),
                )
            }
            _ => Err(anyhow::anyhow!("Action '{}' not implemented yet", action)),
        }
    }
}
