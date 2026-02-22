use aria2_mcp_rs::config::Config;
use std::env;

#[test]
fn test_multi_instance_config_env_vars() {
    env::set_var("ARIA2_MCP__INSTANCES__0__NAME", "instance1");
    env::set_var("ARIA2_MCP__INSTANCES__0__RPC_URL", "http://localhost:6800/jsonrpc");
    env::set_var("ARIA2_MCP__INSTANCES__0__RPC_SECRET", "secret1");
    
    env::set_var("ARIA2_MCP__INSTANCES__1__NAME", "instance2");
    env::set_var("ARIA2_MCP__INSTANCES__1__RPC_URL", "http://localhost:6801/jsonrpc");
    
    let config = Config::load().expect("Failed to load config");
    
    assert_eq!(config.instances.len(), 2);
    assert_eq!(config.instances[0].name, "instance1");
    assert_eq!(config.instances[0].rpc_url, "http://localhost:6800/jsonrpc");
    assert_eq!(config.instances[0].rpc_secret, Some("secret1".to_string()));
    
    assert_eq!(config.instances[1].name, "instance2");
    assert_eq!(config.instances[1].rpc_url, "http://localhost:6801/jsonrpc");
    assert_eq!(config.instances[1].rpc_secret, None);
    
    // Cleanup
    env::remove_var("ARIA2_MCP__INSTANCES__0__NAME");
    env::remove_var("ARIA2_MCP__INSTANCES__0__RPC_URL");
    env::remove_var("ARIA2_MCP__INSTANCES__0__RPC_SECRET");
    env::remove_var("ARIA2_MCP__INSTANCES__1__NAME");
    env::remove_var("ARIA2_MCP__INSTANCES__1__RPC_URL");
}

#[test]
fn test_multi_instance_config_toml() {
    let toml_content = r#"
        rpc_url = "http://localhost:6800/jsonrpc"
        transport = "stdio"
        http_port = 3000
        log_level = "info"
        lazy_mode = false
        no_verify_ssl = true

        [[instances]]
        name = "primary"
        rpc_url = "http://localhost:6800/jsonrpc"
        rpc_secret = "secret1"

        [[instances]]
        name = "secondary"
        rpc_url = "http://localhost:6801/jsonrpc"
    "#;
    
    let mut config: Config = toml::from_str(toml_content).expect("Failed to parse TOML");
    config.normalize();
    
    assert_eq!(config.instances.len(), 2);
    assert_eq!(config.instances[0].name, "primary");
    assert_eq!(config.instances[1].name, "secondary");
}

#[test]
fn test_config_backward_compatibility() {
    let toml_content = r#"
        rpc_url = "http://legacy:6800/jsonrpc"
        rpc_secret = "legacy_secret"
        transport = "stdio"
        http_port = 3000
        log_level = "info"
        lazy_mode = false
        no_verify_ssl = true
    "#;
    
    let mut config: Config = toml::from_str(toml_content).expect("Failed to parse TOML");
    config.normalize();
    
    // Even if instances is missing, it should have one entry from legacy fields
    assert_eq!(config.instances.len(), 1);
    assert_eq!(config.instances[0].name, "default");
    assert_eq!(config.instances[0].rpc_url, "http://legacy:6800/jsonrpc");
    assert_eq!(config.instances[0].rpc_secret, Some("legacy_secret".to_string()));
}
