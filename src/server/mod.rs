pub mod handler;
pub mod sse;
pub mod stdio;

use anyhow::Result;
use chrono::{Datelike, Local, Timelike};
use std::sync::Arc;
use tokio::time::{self, Duration};

use crate::aria2::Aria2Client;
use crate::config::{Config, TransportType};
use crate::tools::ToolRegistry;

pub struct McpServer {
    config: Config,
    registry: Arc<ToolRegistry>,
    client: Arc<Aria2Client>,
}

impl McpServer {
    pub fn new(config: Config, registry: ToolRegistry, client: Aria2Client) -> Self {
        Self {
            config,
            registry: Arc::new(registry),
            client: Arc::new(client),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let client_clone = Arc::clone(&self.client);
        tokio::spawn(async move {
            if let Err(e) = start_scheduler(client_clone).await {
                log::error!("Scheduler error: {}", e);
            }
        });

        match self.config.transport {
            TransportType::Stdio => {
                stdio::run_server(Arc::clone(&self.registry), Arc::clone(&self.client)).await
            }
            TransportType::Sse => {
                sse::run_server(
                    self.config.port,
                    Arc::clone(&self.registry),
                    Arc::clone(&self.client),
                )
                .await
            }
        }
    }
}

async fn start_scheduler(client: Arc<Aria2Client>) -> Result<()> {
    let mut interval = time::interval(Duration::from_secs(60));
    let mut last_profile: Option<String> = None;

    loop {
        interval.tick().await;

        let now = Local::now();
        let current_day = match now.weekday() {
            chrono::Weekday::Mon => "mon",
            chrono::Weekday::Tue => "tue",
            chrono::Weekday::Wed => "wed",
            chrono::Weekday::Thu => "thu",
            chrono::Weekday::Fri => "fri",
            chrono::Weekday::Sat => "sat",
            chrono::Weekday::Sun => "sun",
        };
        let current_time = format!("{:02}:{:02}", now.hour(), now.minute());

        let (profiles, schedules) = {
            let config = client.config();
            let config_guard = config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            (
                config_guard.bandwidth_profiles.clone(),
                config_guard.bandwidth_schedules.clone(),
            )
        };

        let mut active_profile_name: Option<String> = None;

        for schedule in &schedules {
            if (schedule.day == "daily" || schedule.day == current_day)
                && current_time >= schedule.start_time
                && current_time < schedule.end_time
            {
                active_profile_name = Some(schedule.profile_name.clone());
                break;
            }
        }

        if let Some(profile_name) = active_profile_name {
            if last_profile.as_ref() != Some(&profile_name) {
                if let Some(profile) = profiles.get(&profile_name) {
                    log::info!("Activating bandwidth profile: {}", profile_name);
                    let options = serde_json::json!({
                        "max-overall-download-limit": profile.max_download,
                        "max-overall-upload-limit": profile.max_upload,
                    });
                    if let Err(e) = client.change_global_option(options).await {
                        log::error!("Failed to activate profile '{}': {}", profile_name, e);
                    } else {
                        last_profile = Some(profile_name);
                    }
                }
            }
        } else if last_profile.is_some() {
            // No schedule active, but we had one.
            // For now, we don't reset to "default" because we don't know what it is.
            // Maybe we should just leave it.
            // In a real app, you might want a "Default" profile.
            last_profile = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_server() {
        let config = Config::default();
        let registry = ToolRegistry::new();
        let client = Aria2Client::new(config.clone());
        let _server = McpServer::new(config, registry, client);
    }

    #[tokio::test]
    async fn test_server_run_sse_error() {
        let config = Config {
            transport: TransportType::Sse,
            port: 1, // Likely to fail on most systems
            ..Default::default()
        };
        let registry = ToolRegistry::new();
        let client = Aria2Client::new(config.clone());
        let server = McpServer::new(config, registry, client);
        let result = server.run().await;
        assert!(result.is_err());
    }
}
