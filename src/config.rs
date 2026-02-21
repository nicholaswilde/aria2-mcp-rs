use config::{Config as ConfigLoader, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub rpc_secret: Option<String>,
    pub transport: TransportType,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
    Sse,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:6800/jsonrpc".to_string(),
            rpc_secret: None,
            transport: TransportType::Stdio,
            port: 3000,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let s = ConfigLoader::builder()
            // Start with default values
            .set_default("rpc_url", "http://localhost:6800/jsonrpc")?
            .set_default("transport", "stdio")?
            .set_default("port", 3000)?
            // Add configuration from files
            .add_source(File::with_name("aria2-mcp").required(false))
            // Add configuration from environment variables (with a prefix)
            .add_source(Environment::with_prefix("ARIA2_MCP").separator("__"))
            .build()?;

        s.try_deserialize()
    }

    pub fn new(rpc_url: String, rpc_secret: Option<String>) -> Self {
        Self {
            rpc_url,
            rpc_secret,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.rpc_url, "http://localhost:6800/jsonrpc");
        assert_eq!(config.transport, TransportType::Stdio);
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_new() {
        let config = Config::new("http://test".to_string(), Some("secret".to_string()));
        assert_eq!(config.rpc_url, "http://test");
        assert_eq!(config.rpc_secret, Some("secret".to_string()));
    }

    #[test]
    fn test_load_config() {
        let config = Config::load();
        assert!(config.is_ok());
    }
}
