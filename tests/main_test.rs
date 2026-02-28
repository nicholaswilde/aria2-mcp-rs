use aria2_mcp_rs::{Config, Error};

#[test]
fn test_config_default() {
    let config = Config::default();
    assert_eq!(config.rpc_url, "http://127.0.0.1:6800/jsonrpc");
    assert_eq!(config.rpc_secret, None);
}

#[test]
fn test_config_new() {
    let config = Config::new("http://example.com".to_string(), Some("secret".to_string()));
    assert_eq!(config.rpc_url, "http://example.com");
    assert_eq!(config.rpc_secret, Some("secret".to_string()));
}

#[test]
fn test_error_display() {
    let err = Error::Config("test error".to_string());
    assert_eq!(format!("{}", err), "Configuration error: test error");
}

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::other("io error");
    let err = Error::from(io_err);
    assert!(format!("{}", err).contains("io error"));
}

#[tokio::test]
async fn test_main_logic_state_merge() {
    use std::sync::Mutex;
    static ENV_MUTEX: Mutex<()> = Mutex::new(());
    let temp_state_file = "aria2_mcp_state_test_consolidated.json";

    {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Create a mock state file
        let state_json = r#"{
        "rules": {},
        "bandwidth_profiles": {
            "test_profile": { "max_download": "1M", "max_upload": "1M" }
        },
        "bandwidth_schedules": [
            { "day": "daily", "start_time": "00:00", "end_time": "23:59", "profile_name": "test_profile" }
        ],
        "organize_rules": [
            { "name": "test_rule", "pattern": ".*", "targetDir": "/tmp" }
        ]
    }"#;
        std::fs::write(temp_state_file, state_json).unwrap();
    }

    // Use a clean config
    let mut config = Config::default();

    // Logic from main (simplified/copied)
    let state_manager =
        aria2_mcp_rs::state::StateManager::new(std::path::PathBuf::from(temp_state_file));
    let state = state_manager.load().await.expect("Failed to load state");

    {
        let _guard = ENV_MUTEX.lock().unwrap();
        for (k, v) in state.bandwidth_profiles {
            config.bandwidth_profiles.insert(k, v);
        }
        for schedule in state.bandwidth_schedules {
            if !config.bandwidth_schedules.contains(&schedule) {
                config.bandwidth_schedules.push(schedule);
            }
        }
        for rule in state.organize_rules {
            if !config.organize_rules.iter().any(|r| r.name == rule.name) {
                config.organize_rules.push(rule);
            }
        }
    }

    assert!(config.bandwidth_profiles.contains_key("test_profile"));
    assert_eq!(config.bandwidth_schedules.len(), 1);
    assert_eq!(config.organize_rules.len(), 1);
    assert_eq!(config.organize_rules[0].target_dir, "/tmp");

    // Cleanup
    let _ = std::fs::remove_file(temp_state_file);
}
