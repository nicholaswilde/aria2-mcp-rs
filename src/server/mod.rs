pub mod handler;
pub mod sse;
pub mod stdio;

use anyhow::Result;
use chrono::{Datelike, Local, Timelike};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{self, Duration};

use crate::aria2::Aria2Client;
use crate::config::{Config, TransportType};
use crate::resources::ResourceRegistry;
use crate::tools::ToolRegistry;

pub struct McpServer {
    config: Config,
    registry: Arc<RwLock<ToolRegistry>>,
    resource_registry: Arc<RwLock<ResourceRegistry>>,
    clients: Vec<Arc<Aria2Client>>,
}

impl McpServer {
    pub fn new(
        config: Config,
        registry: ToolRegistry,
        resource_registry: ResourceRegistry,
        clients: Vec<Aria2Client>,
    ) -> Self {
        Self {
            config,
            registry: Arc::new(RwLock::new(registry)),
            resource_registry: Arc::new(RwLock::new(resource_registry)),
            clients: clients.into_iter().map(Arc::new).collect(),
        }
    }

    pub fn clients(&self) -> &[Arc<Aria2Client>] {
        &self.clients
    }

    pub async fn run(&self) -> Result<()> {
        for client in &self.clients {
            let client_clone = Arc::clone(client);
            tokio::spawn(async move {
                if let Err(e) = start_scheduler(client_clone).await {
                    log::error!("Scheduler error: {}", e);
                }
            });
        }

        match self.config.transport {
            TransportType::Stdio => {
                stdio::run_server(
                    Arc::clone(&self.registry),
                    Arc::clone(&self.resource_registry),
                    self.clients.clone(),
                )
                .await
            }
            TransportType::Sse => {
                if !check_port_available(self.config.http_port).await {
                    return Err(anyhow::anyhow!(
                        "HTTP port {} is already in use",
                        self.config.http_port
                    ));
                }
                sse::run_server(
                    self.config.http_port,
                    self.config.http_auth_token.clone(),
                    Arc::clone(&self.registry),
                    Arc::clone(&self.resource_registry),
                    self.clients.clone(),
                )
                .await
            }
        }
    }
}

async fn check_port_available(port: u16) -> bool {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tokio::net::TcpListener::bind(addr).await.is_ok()
}

fn get_active_profile(
    current_day: &str,
    current_time: &str,
    schedules: &[crate::config::BandwidthSchedule],
) -> Option<String> {
    for schedule in schedules {
        if schedule.day == "daily" || schedule.day == current_day {
            let start = schedule.start_time.as_str();
            let end = schedule.end_time.as_str();

            if start <= end {
                // Normal range
                if current_time >= start && current_time < end {
                    return Some(schedule.profile_name.clone());
                }
            } else {
                // Wraps around midnight
                if current_time >= start || current_time < end {
                    return Some(schedule.profile_name.clone());
                }
            }
        }
    }
    None
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

        let active_profile_name = get_active_profile(current_day, &current_time, &schedules);

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
            last_profile = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BandwidthSchedule;

    #[test]
    fn test_get_active_profile() {
        let schedules = vec![
            BandwidthSchedule {
                day: "mon".to_string(),
                start_time: "09:00".to_string(),
                end_time: "17:00".to_string(),
                profile_name: "work".to_string(),
            },
            BandwidthSchedule {
                day: "daily".to_string(),
                start_time: "22:00".to_string(),
                end_time: "06:00".to_string(), // Note: this simple logic won't handle midnight wrap correctly if not split, but let's test it as implemented
                profile_name: "night".to_string(),
            },
        ];

        // Monday 10:00 -> work
        assert_eq!(
            get_active_profile("mon", "10:00", &schedules),
            Some("work".to_string())
        );

        // Tuesday 10:00 -> None
        assert_eq!(get_active_profile("tue", "10:00", &schedules), None);

        // Any day 23:00 -> night
        assert_eq!(
            get_active_profile("wed", "23:00", &schedules),
            Some("night".to_string())
        );

        // Any day 07:00 -> None
        assert_eq!(get_active_profile("thu", "07:00", &schedules), None);
    }

    #[test]
    fn test_new_server() {
        let config = Config::default();
        let registry = ToolRegistry::new(&config);
        let resource_registry = ResourceRegistry::default();
        let client = Aria2Client::new(config.clone());
        let _server = McpServer::new(config, registry, resource_registry, vec![client]);
    }

    #[tokio::test]
    async fn test_server_run_sse_error() {
        // Find a port and keep it occupied
        let listener = std::net::TcpListener::bind("0.0.0.0:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let config = Config {
            transport: TransportType::Sse,
            http_port: port,
            ..Default::default()
        };
        let registry = ToolRegistry::new(&config);
        let resource_registry = ResourceRegistry::default();
        let client = Aria2Client::new(config.clone());
        let server = McpServer::new(config, registry, resource_registry, vec![client]);
        let result = server.run().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already in use"));
    }

    #[tokio::test]
    async fn test_check_port_available() {
        // Find a free port
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        // Port is occupied by 'listener', so check_port_available should return false
        assert!(!super::check_port_available(port).await);

        drop(listener);

        // Port is now free
        assert!(super::check_port_available(port).await);
    }
}
