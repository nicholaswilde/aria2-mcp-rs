use aria2_mcp_rs::Config;

#[test]
fn test_config_auth_token_default() {
    let config = Config::default();
    assert_eq!(config.http_auth_token, None);
}

#[test]
fn test_config_auth_token_override() {
    let config = Config {
        http_auth_token: Some("secret-token".to_string()),
        ..Config::default()
    };
    assert_eq!(config.http_auth_token, Some("secret-token".to_string()));
}
