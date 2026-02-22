use aria2_mcp_rs::{Config, TransportType};

#[test]
fn test_config_auth_token_default() {
    let config = Config::default();
    assert_eq!(config.http_auth_token, None);
}

#[test]
fn test_config_auth_token_override() {
    let mut config = Config::default();
    config.http_auth_token = Some("secret-token".to_string());
    assert_eq!(config.http_auth_token, Some("secret-token".to_string()));
}
