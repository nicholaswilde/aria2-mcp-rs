use aria2_mcp_rs::Config;
use std::env;

#[test]
fn test_config_log_level_default() {
    let config = Config::default();
    // This will fail to compile initially because log_level is not in Config
    assert_eq!(config.log_level, "info");
}

#[test]
fn test_config_log_level_env_override() {
    env::set_var("ARIA2_MCP__LOG_LEVEL", "debug");
    let config = Config::load().expect("Failed to load config");
    assert_eq!(config.log_level, "debug");
    env::remove_var("ARIA2_MCP__LOG_LEVEL");
}
