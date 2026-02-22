use config::{Config as ConfigLoader, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub rpc_secret: Option<String>,
    pub transport: TransportType,
    #[serde(alias = "port")]
    pub http_port: u16,
    pub http_auth_token: Option<String>,
    pub log_level: String,
    pub lazy_mode: bool,
    pub no_verify_ssl: bool,
    #[serde(default)]
    pub bandwidth_profiles: HashMap<String, BandwidthProfile>,
    #[serde(default)]
    pub bandwidth_schedules: Vec<BandwidthSchedule>,
    #[serde(default, deserialize_with = "deserialize_instances")]
    pub instances: Vec<Aria2Instance>,
}

fn deserialize_instances<'de, D>(deserializer: D) -> Result<Vec<Aria2Instance>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Instances {
        Vec(Vec<Aria2Instance>),
        Map(HashMap<String, Aria2Instance>),
    }

    match Instances::deserialize(deserializer)? {
        Instances::Vec(v) => Ok(v),
        Instances::Map(m) => {
            let mut v: Vec<_> = m.into_iter().collect();
            v.sort_by_key(|(k, _)| k.parse::<usize>().unwrap_or(0));
            Ok(v.into_iter().map(|(_, v)| v).collect())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Aria2Instance {
    pub name: String,
    pub rpc_url: String,
    pub rpc_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BandwidthProfile {
    pub max_download: String,
    pub max_upload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BandwidthSchedule {
    pub day: String,        // "daily", "mon", "tue", ...
    pub start_time: String, // HH:MM
    pub end_time: String,   // HH:MM
    pub profile_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
    #[serde(alias = "http")]
    Sse,
}

impl Default for Config {
    fn default() -> Self {
        let rpc_url = "http://localhost:6800/jsonrpc".to_string();
        let rpc_secret = None;
        Self {
            rpc_url: rpc_url.clone(),
            rpc_secret: rpc_secret.clone(),
            transport: TransportType::Stdio,
            http_port: 3000,
            http_auth_token: None,
            log_level: "info".to_string(),
            lazy_mode: false,
            no_verify_ssl: true,
            bandwidth_profiles: HashMap::new(),
            bandwidth_schedules: Vec::new(),
            instances: vec![Aria2Instance {
                name: "default".to_string(),
                rpc_url,
                rpc_secret,
            }],
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let s = ConfigLoader::builder()
            // Start with default values
            .set_default("rpc_url", "http://localhost:6800/jsonrpc")?
            .set_default("transport", "stdio")?
            .set_default("http_port", 3000)?
            .set_default("log_level", "info")?
            .set_default("lazy_mode", false)?
            .set_default("no_verify_ssl", true)?
            // Add configuration from files
            .add_source(File::with_name("config").required(false))
            .add_source(File::with_name("aria2-mcp").required(false))
            // Add configuration from environment variables (with a prefix)
            .add_source(Environment::with_prefix("ARIA2_MCP").separator("__"))
            .build()?;

        let mut config: Self = s.try_deserialize()?;
        config.normalize();
        Ok(config)
    }

    pub fn normalize(&mut self) {
        // If instances is empty, populate from legacy fields
        if self.instances.is_empty() {
            self.instances.push(Aria2Instance {
                name: "default".to_string(),
                rpc_url: self.rpc_url.clone(),
                rpc_secret: self.rpc_secret.clone(),
            });
        }
    }

    pub fn new(rpc_url: String, rpc_secret: Option<String>) -> Self {
        Self {
            rpc_url: rpc_url.clone(),
            rpc_secret: rpc_secret.clone(),
            instances: vec![Aria2Instance {
                name: "default".to_string(),
                rpc_url,
                rpc_secret,
            }],
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
        assert_eq!(config.http_port, 3000);
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
