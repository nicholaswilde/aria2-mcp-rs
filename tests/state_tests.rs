use aria2_mcp_rs::config::{BandwidthProfile, BandwidthSchedule};
use aria2_mcp_rs::state::{StateData, StateManager};
use aria2_mcp_rs::tools::organize_completed::Rule;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_state_manager_save_and_load() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    // Create new state manager
    let manager = StateManager::new(path.clone());

    // Initialize data
    let mut initial_data = StateData::default();
    initial_data.bandwidth_profiles.insert(
        "test_profile".to_string(),
        BandwidthProfile {
            max_download: "1M".to_string(),
            max_upload: "500K".to_string(),
        },
    );
    initial_data.bandwidth_schedules.push(BandwidthSchedule {
        day: "daily".to_string(),
        start_time: "08:00".to_string(),
        end_time: "18:00".to_string(),
        profile_name: "test_profile".to_string(),
    });
    initial_data.organize_rules.push(Rule {
        name: "Test Rule".to_string(),
        pattern: None,
        extensions: Some(vec!["txt".to_string()]),
        target_dir: "/tmp/test".to_string(),
    });

    // Save state
    manager
        .save(&initial_data)
        .await
        .expect("Failed to save state");

    // Load state into a new manager to verify persistence
    let new_manager = StateManager::new(path);
    let loaded_data: StateData = new_manager.load().await.expect("Failed to load state");

    assert_eq!(
        loaded_data
            .bandwidth_profiles
            .get("test_profile")
            .unwrap()
            .max_download,
        "1M"
    );
    assert_eq!(
        loaded_data.bandwidth_schedules[0].profile_name,
        "test_profile"
    );
    assert_eq!(loaded_data.organize_rules[0].name, "Test Rule");
}

#[tokio::test]
async fn test_state_manager_load_non_existent_file() {
    let path = PathBuf::from("/non/existent/path/for/state.json");
    let manager = StateManager::new(path);

    let result = manager.load().await;
    let data: StateData = result.expect("Should return default on non-existent file");
    assert!(data.bandwidth_profiles.is_empty());
    assert!(data.bandwidth_schedules.is_empty());
    assert!(data.organize_rules.is_empty());
}

#[tokio::test]
async fn test_state_manager_empty_file() {
    use std::fs;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let path = dir.path().join("empty.json");
    fs::write(&path, "   ").unwrap();

    let manager = StateManager::new(path);
    let data = manager.load().await.expect("Should handle empty file");
    assert!(data.bandwidth_profiles.is_empty());
}

#[tokio::test]
async fn test_state_manager_invalid_json() {
    use std::fs;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let path = dir.path().join("invalid.json");
    fs::write(&path, "{ invalid: json }").unwrap();

    let manager = StateManager::new(path);
    let result = manager.load().await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to parse state file"));
}

#[tokio::test]
async fn test_state_manager_create_dir_failure() {
    // On Linux, /root/test/state.json should fail if not root
    let path = std::path::PathBuf::from("/root/aria2-mcp/state.json");
    let manager = StateManager::new(path);
    let data = StateData::default();

    let result = manager.save(&data).await;
    if let Err(e) = result {
        assert!(
            e.to_string()
                .contains("Failed to create parent directories")
                || e.to_string().contains("Permission denied")
        );
    }
}

#[tokio::test]
async fn test_state_manager_write_failure() {
    use std::fs;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let path = dir.path().join("readonly");
    fs::create_dir(&path).unwrap(); // It's a directory, so writing to it as a file should fail

    let manager = StateManager::new(path);
    let data = StateData::default();

    let result = manager.save(&data).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to write state file"));
}

#[tokio::test]
async fn test_state_manager_read_failure() {
    use std::fs;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let path = dir.path().join("dir");
    fs::create_dir(&path).unwrap(); // It's a directory, so reading it as a file should fail

    let manager = StateManager::new(path);
    let result = manager.load().await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to read state file"));
}
