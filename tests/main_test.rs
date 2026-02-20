use aria2_mcp_rs::{Config, Error};

#[test]
fn test_config_default() {
    let config = Config::default();
    assert_eq!(config.rpc_url, "http://localhost:6800/jsonrpc");
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
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io error");
    let err = Error::from(io_err);
    assert!(format!("{}", err).contains("io error"));
}
