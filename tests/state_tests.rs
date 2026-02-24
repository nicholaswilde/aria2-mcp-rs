use aria2_mcp_rs::state::{StateData, StateManager};
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
    initial_data
        .rules
        .insert("test_rule".to_string(), "active".to_string());

    // Save state
    manager
        .save(&initial_data)
        .await
        .expect("Failed to save state");

    // Load state into a new manager to verify persistence
    let new_manager = StateManager::new(path);
    let loaded_data: StateData = new_manager.load().await.expect("Failed to load state");

    assert_eq!(
        loaded_data.rules.get("test_rule"),
        Some(&"active".to_string())
    );
}

#[tokio::test]
async fn test_state_manager_load_non_existent_file() {
    let path = PathBuf::from("/non/existent/path/for/state.json");
    let manager = StateManager::new(path);

    let result = manager.load().await;
    let data: StateData = result.expect("Should return default on non-existent file");
    assert!(data.rules.is_empty());
}
