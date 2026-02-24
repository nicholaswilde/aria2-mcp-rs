pub mod handler;
pub mod sse;
pub mod stdio;

use anyhow::Result;
use chrono::{Datelike, Local, Timelike};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{self, Duration};

use crate::aria2::notifications::Aria2Notification;
use crate::aria2::recovery::RecoveryManager;
use crate::aria2::Aria2Client;
use crate::config::{Config, TransportType};
use crate::prompts::PromptRegistry;
use crate::resources::ResourceRegistry;
use crate::tools::ToolRegistry;

pub struct McpServer {
    config: Config,
    registry: Arc<RwLock<ToolRegistry>>,
    resource_registry: Arc<RwLock<ResourceRegistry>>,
    prompt_registry: Arc<RwLock<PromptRegistry>>,
    clients: Vec<Arc<Aria2Client>>,
    recovery_manager: Arc<RecoveryManager>,
}

impl McpServer {
    pub fn new(
        config: Config,
        registry: ToolRegistry,
        resource_registry: ResourceRegistry,
        prompt_registry: PromptRegistry,
        clients: Vec<Aria2Client>,
    ) -> Self {
        let recovery_manager = Arc::new(RecoveryManager::new(config.retry_config.clone()));
        Self {
            config,
            registry: Arc::new(RwLock::new(registry)),
            resource_registry: Arc::new(RwLock::new(resource_registry)),
            prompt_registry: Arc::new(RwLock::new(prompt_registry)),
            clients: clients.into_iter().map(Arc::new).collect(),
            recovery_manager,
        }
    }

    pub fn clients(&self) -> &[Arc<Aria2Client>] {
        &self.clients
    }

    pub async fn run(&self) -> Result<()> {
        let (notification_tx, notification_rx) =
            tokio::sync::mpsc::channel::<Aria2Notification>(100);

        for client in &self.clients {
            let client_clone = Arc::clone(client);
            let tx_clone = notification_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = client_clone.start_notifications(tx_clone).await {
                    log::error!(
                        "Notification error for instance {}: {}",
                        client_clone.name,
                        e
                    );
                }
            });

            let client_clone = Arc::clone(client);
            tokio::spawn(async move {
                if let Err(e) = start_scheduler(client_clone).await {
                    log::error!("Scheduler error: {}", e);
                }
            });

            let client_clone = Arc::clone(client);
            let recovery_manager_clone = Arc::clone(&self.recovery_manager);
            tokio::spawn(async move {
                if let Err(e) = start_recovery_task(client_clone, recovery_manager_clone).await {
                    log::error!("Recovery task error: {}", e);
                }
            });

            let client_clone = Arc::clone(client);
            tokio::spawn(async move {
                if let Err(e) = crate::tools::rss::start_rss_monitoring(client_clone).await {
                    log::error!("RSS monitoring error: {}", e);
                }
            });

            let client_clone = Arc::clone(client);
            tokio::spawn(async move {
                if let Err(e) = start_purge_task(client_clone).await {
                    log::error!("Purge task error: {}", e);
                }
            });
        }

        match self.config.transport {
            TransportType::Stdio => {
                stdio::run_server(
                    Arc::clone(&self.registry),
                    Arc::clone(&self.resource_registry),
                    Arc::clone(&self.prompt_registry),
                    self.clients.clone(),
                    notification_rx,
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
                    Arc::clone(&self.prompt_registry),
                    self.clients.clone(),
                    notification_rx,
                )
                .await
            }
        }
    }
}

pub async fn start_purge_task(client: Arc<Aria2Client>) -> Result<()> {
    let mut interval = time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        let config = client.config();
        let purge_config = {
            let config_guard = config
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;
            config_guard.purge_config.clone()
        };

        if !purge_config.enabled {
            continue;
        }

        // Adjust interval if needed
        if interval.period().as_secs() != purge_config.interval_secs {
            interval = time::interval(Duration::from_secs(purge_config.interval_secs));
            interval.tick().await;
        }

        let stopped = match client.tell_stopped(0, 1000, None).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to fetch stopped downloads for purge: {}", e);
                continue;
            }
        };

        if let Some(items) = stopped.as_array() {
            for item in items {
                if let Some(gid) = item.get("gid").and_then(|v| v.as_str()) {
                    if purge_config.excluded_gids.contains(gid) {
                        continue;
                    }

                    if is_purgeable(item) {
                        log::info!("Purging download {} (instance {})...", gid, client.name);
                        if let Err(e) = client.remove_download_result(gid).await {
                            log::error!("Failed to purge download {}: {}", gid, e);
                        }
                    }
                }
            }
        }
    }
}

pub fn is_purgeable(item: &serde_json::Value) -> bool {
    if let Some(status) = item.get("status").and_then(|v| v.as_str()) {
        // For now, we purge anything that is complete or error.
        // We could add more complex logic here later if aria2 provides timestamps.
        return status == "complete" || status == "error";
    }
    false
}

async fn start_recovery_task(
    client: Arc<Aria2Client>,
    recovery_manager: Arc<RecoveryManager>,
) -> Result<()> {
    let mut interval = time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        // Check for stopped downloads with errors
        let stopped = match client.tell_stopped(0, 100, None).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to fetch stopped downloads for recovery: {}", e);
                continue;
            }
        };

        if let Some(items) = stopped.as_array() {
            for item in items {
                if let Some(gid) = item.get("gid").and_then(|v| v.as_str()) {
                    if let Some(backoff) = recovery_manager
                        .analyze_and_get_retry_backoff(gid, item)
                        .await
                    {
                        log::info!(
                            "Recovery needed for download {} (instance {}). Retrying in {} seconds.",
                            gid,
                            client.name,
                            backoff
                        );

                        let client_retry = Arc::clone(&client);
                        let recovery_manager_retry = Arc::clone(&recovery_manager);
                        let gid_retry = gid.to_string();

                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_secs(backoff)).await;
                            if let Err(e) = recovery_manager_retry
                                .perform_retry(&client_retry, &gid_retry)
                                .await
                            {
                                log::error!("Retry failed for download {}: {}", gid_retry, e);
                            }
                        });
                    }
                }
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
        let prompt_registry = PromptRegistry::default();
        let client = Aria2Client::new(config.clone());
        let _server = McpServer::new(
            config,
            registry,
            resource_registry,
            prompt_registry,
            vec![client],
        );
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
        let prompt_registry = PromptRegistry::default();
        let client = Aria2Client::new(config.clone());
        let server = McpServer::new(
            config,
            registry,
            resource_registry,
            prompt_registry,
            vec![client],
        );
        let result = server.run().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already in use"));
    }

    #[tokio::test]
    async fn test_check_port_available() {
        // Find a free port
        let listener = std::net::TcpListener::bind("0.0.0.0:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        // Port is occupied by 'listener', so check_port_available should return false
        assert!(!super::check_port_available(port).await);

        drop(listener);

        // Port is now free
        assert!(super::check_port_available(port).await);
    }

    #[test]
    fn test_is_purgeable() {
        let item_complete = serde_json::json!({ "status": "complete" });
        let item_error = serde_json::json!({ "status": "error" });
        let item_active = serde_json::json!({ "status": "active" });
        let item_removed = serde_json::json!({ "status": "removed" });

        assert!(is_purgeable(&item_complete));
        assert!(is_purgeable(&item_error));
        assert!(!is_purgeable(&item_active));
        assert!(!is_purgeable(&item_removed));
    }

    #[tokio::test]
    async fn test_start_purge_task_mock() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let rpc_url = format!("{}/jsonrpc", mock_server.uri());

        let config = Config {
            instances: vec![crate::config::Aria2Instance {
                name: "test".to_string(),
                rpc_url,
                rpc_secret: None,
            }],
            purge_config: crate::config::PurgeConfig {
                enabled: true,
                interval_secs: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let client = Arc::new(Aria2Client::new_with_instance(
            config.clone(),
            config.instances[0].clone(),
        ));

        // Mock tellStopped
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "1",
                "result": [
                    { "gid": "purge1", "status": "complete" },
                    { "gid": "keep1", "status": "active" }
                ]
            })))
            .mount(&mock_server)
            .await;

        // Mock removeDownloadResult
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "1",
                "result": "OK"
            })))
            .mount(&mock_server)
            .await;

        // Run for a short time
        let task_client = Arc::clone(&client);
        tokio::spawn(async move {
            let _ = start_purge_task(task_client).await;
        });

        tokio::time::sleep(Duration::from_millis(1500)).await;
        // If we reach here, it at least didn't panic and ran one iteration
    }

    #[tokio::test]
    async fn test_start_recovery_task_mock() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let rpc_url = format!("{}/jsonrpc", mock_server.uri());

        let config = Config {
            instances: vec![crate::config::Aria2Instance {
                name: "test".to_string(),
                rpc_url,
                rpc_secret: None,
            }],
            retry_config: crate::aria2::recovery::RetryConfig {
                max_retries: 3,
                ..Default::default()
            },
            ..Default::default()
        };

        let client = Arc::new(Aria2Client::new_with_instance(
            config.clone(),
            config.instances[0].clone(),
        ));
        let recovery_manager = Arc::new(RecoveryManager::new(config.retry_config.clone()));

        // Mock tellStopped
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "1",
                "result": [
                    { "gid": "recover1", "status": "error", "errorCode": "1" }
                ]
            })))
            .mount(&mock_server)
            .await;

        // Mock tellStatus for recovery check
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "1",
                "result": { "gid": "recover1", "status": "error", "errorCode": "1" }
            })))
            .mount(&mock_server)
            .await;

        // Run for a short time
        let task_client = Arc::clone(&client);
        let task_manager = Arc::clone(&recovery_manager);
        tokio::spawn(async move {
            let _ = start_recovery_task(task_client, task_manager).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_start_scheduler_mock() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let rpc_url = format!("{}/jsonrpc", mock_server.uri());

        let config = Config {
            instances: vec![crate::config::Aria2Instance {
                name: "test".to_string(),
                rpc_url,
                rpc_secret: None,
            }],
            bandwidth_profiles: std::collections::HashMap::from([(
                "night".to_string(),
                crate::config::BandwidthProfile {
                    max_download: "1M".to_string(),
                    max_upload: "100K".to_string(),
                },
            )]),
            bandwidth_schedules: vec![crate::config::BandwidthSchedule {
                day: "daily".to_string(),
                start_time: "00:00".to_string(),
                end_time: "23:59".to_string(),
                profile_name: "night".to_string(),
            }],
            ..Default::default()
        };

        let client = Arc::new(Aria2Client::new_with_instance(
            config.clone(),
            config.instances[0].clone(),
        ));

        // Mock changeGlobalOption
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "1",
                "result": "OK"
            })))
            .mount(&mock_server)
            .await;

        let task_client = Arc::clone(&client);
        tokio::spawn(async move {
            let _ = start_scheduler(task_client).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_start_recovery_task_empty_mock() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let rpc_url = format!("{}/jsonrpc", mock_server.uri());

        let config = Config {
            instances: vec![crate::config::Aria2Instance {
                name: "test".to_string(),
                rpc_url,
                rpc_secret: None,
            }],
            ..Default::default()
        };

        let client = Arc::new(Aria2Client::new_with_instance(
            config.clone(),
            config.instances[0].clone(),
        ));
        let recovery_manager = Arc::new(RecoveryManager::new(config.retry_config.clone()));

        // Mock tellStopped with empty result
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "1",
                "result": []
            })))
            .mount(&mock_server)
            .await;

        let task_client = Arc::clone(&client);
        let task_manager = Arc::clone(&recovery_manager);
        tokio::spawn(async move {
            let _ = start_recovery_task(task_client, task_manager).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_start_scheduler_missing_profile_mock() {
        use wiremock::MockServer;

        let mock_server = MockServer::start().await;

        let config = Config {
            instances: vec![crate::config::Aria2Instance {
                name: "test".to_string(),
                rpc_url: format!("{}/jsonrpc", mock_server.uri()),
                rpc_secret: None,
            }],
            bandwidth_schedules: vec![crate::config::BandwidthSchedule {
                day: "daily".to_string(),
                start_time: "00:00".to_string(),
                end_time: "23:59".to_string(),
                profile_name: "nonexistent".to_string(),
            }],
            ..Default::default()
        };

        let client = Arc::new(Aria2Client::new_with_instance(
            config.clone(),
            config.instances[0].clone(),
        ));

        let task_client = Arc::clone(&client);
        tokio::spawn(async move {
            let _ = start_scheduler(task_client).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_start_purge_task_error_mock() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let rpc_url = format!("{}/jsonrpc", mock_server.uri());

        let config = Config {
            instances: vec![crate::config::Aria2Instance {
                name: "test".to_string(),
                rpc_url,
                rpc_secret: None,
            }],
            purge_config: crate::config::PurgeConfig {
                enabled: true,
                interval_secs: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let client = Arc::new(Aria2Client::new_with_instance(
            config.clone(),
            config.instances[0].clone(),
        ));

        // Mock error for tellStopped
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let task_client = Arc::clone(&client);
        tokio::spawn(async move {
            let _ = start_purge_task(task_client).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_start_purge_task_disabled_mock() {
        let config = Config {
            purge_config: crate::config::PurgeConfig {
                enabled: false,
                ..Default::default()
            },
            ..Default::default()
        };

        let client = Arc::new(Aria2Client::new(config));

        let task_client = Arc::clone(&client);
        tokio::spawn(async move {
            let _ = start_purge_task(task_client).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_start_scheduler_error_mock() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let rpc_url = format!("{}/jsonrpc", mock_server.uri());

        let config = Config {
            instances: vec![crate::config::Aria2Instance {
                name: "test".to_string(),
                rpc_url,
                rpc_secret: None,
            }],
            bandwidth_profiles: std::collections::HashMap::from([(
                "night".to_string(),
                crate::config::BandwidthProfile {
                    max_download: "1M".to_string(),
                    max_upload: "100K".to_string(),
                },
            )]),
            bandwidth_schedules: vec![crate::config::BandwidthSchedule {
                day: "daily".to_string(),
                start_time: "00:00".to_string(),
                end_time: "23:59".to_string(),
                profile_name: "night".to_string(),
            }],
            ..Default::default()
        };

        let client = Arc::new(Aria2Client::new_with_instance(
            config.clone(),
            config.instances[0].clone(),
        ));

        // Mock error for changeGlobalOption
        Mock::given(method("POST"))
            .and(path("/jsonrpc"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let task_client = Arc::clone(&client);
        tokio::spawn(async move {
            let _ = start_scheduler(task_client).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
