use aria2_mcp_rs::Config;
use std::env;

#[test]
fn test_config_env_vars_comprehensive() {
    // Set environment variables for Config loader (config-rs)
    // Note: config-rs with separator "__" expects ARIA2_MCP__FIELD
    env::set_var("ARIA2_MCP__RPC_URL", "http://env-test:6800/jsonrpc");
    env::set_var("ARIA2_MCP__RPC_SECRET", "env-secret");
    env::set_var("ARIA2_MCP__TRANSPORT", "sse");
    env::set_var("ARIA2_MCP__HTTP_HOST", "1.2.3.4");
    env::set_var("ARIA2_MCP__HTTP_PORT", "9999");
    env::set_var("ARIA2_MCP__LOG_LEVEL", "debug");
    env::set_var("ARIA2_MCP__LAZY_MODE", "true");
    env::set_var("ARIA2_MCP__NO_VERIFY_SSL", "false");

    let config = Config::load().expect("Failed to load config from env vars");

    assert_eq!(config.rpc_url, "http://env-test:6800/jsonrpc");
    assert_eq!(config.rpc_secret, Some("env-secret".to_string()));
    assert_eq!(config.transport, aria2_mcp_rs::TransportType::Sse);
    assert_eq!(config.http_host, "1.2.3.4");
    assert_eq!(config.http_port, 9999);
    assert_eq!(config.log_level, "debug");
    assert!(config.lazy_mode);
    assert!(!config.no_verify_ssl);

    // Cleanup
    env::remove_var("ARIA2_MCP__RPC_URL");
    env::remove_var("ARIA2_MCP__RPC_SECRET");
    env::remove_var("ARIA2_MCP__TRANSPORT");
    env::remove_var("ARIA2_MCP__HTTP_HOST");
    env::remove_var("ARIA2_MCP__HTTP_PORT");
    env::remove_var("ARIA2_MCP__LOG_LEVEL");
    env::remove_var("ARIA2_MCP__LAZY_MODE");
    env::remove_var("ARIA2_MCP__NO_VERIFY_SSL");
}
