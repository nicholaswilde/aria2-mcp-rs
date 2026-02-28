use config::{Config as ConfigLoader, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub rpc_secret: Option<String>,
    pub transport: TransportType,
    #[serde(default = "default_http_host")]
    pub http_host: String,
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
    #[serde(default)]
    pub retry_config: crate::aria2::recovery::RetryConfig,
    #[serde(default, deserialize_with = "deserialize_instances")]
    pub instances: Vec<Aria2Instance>,
    #[serde(default)]
    pub rss_config: RSSConfig,
    #[serde(default)]
    pub purge_config: PurgeConfig,
    #[serde(default)]
    pub organize_rules: Vec<crate::tools::organize_completed::Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RSSConfig {
    #[serde(default)]
    pub feeds: Vec<RSSFeed>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PurgeConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub min_age_secs: u64,
    #[serde(default)]
    pub excluded_gids: HashSet<String>,
}

impl Default for PurgeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: 3600, // 1 hour
            min_age_secs: 86400, // 1 day
            excluded_gids: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RSSFeed {
    pub url: String,
    pub name: String,
    #[serde(default)]
    pub filters: Vec<RSSFilter>,
    #[serde(default)]
    pub download_history: HashSet<String>,
}

impl RSSFeed {
    pub fn has_downloaded(&self, id: &str) -> bool {
        self.download_history.contains(id)
    }

    pub fn mark_downloaded(&mut self, id: String) {
        self.download_history.insert(id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RSSFilter {
    Keyword(String),
    Regex(String),
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

fn default_http_host() -> String {
    "0.0.0.0".to_string()
}

impl Default for Config {
    fn default() -> Self {
        let rpc_url = "http://127.0.0.1:6800/jsonrpc".to_string();
        let rpc_secret = None;
        Self {
            rpc_url: rpc_url.clone(),
            rpc_secret: rpc_secret.clone(),
            transport: TransportType::Stdio,
            http_host: "0.0.0.0".to_string(),
            http_port: 3000,
            http_auth_token: None,
            log_level: "info".to_string(),
            lazy_mode: false,
            no_verify_ssl: true,
            bandwidth_profiles: HashMap::new(),
            bandwidth_schedules: Vec::new(),
            retry_config: crate::aria2::recovery::RetryConfig::default(),
            instances: vec![Aria2Instance {
                name: "default".to_string(),
                rpc_url,
                rpc_secret,
            }],
            rss_config: RSSConfig::default(),
            purge_config: PurgeConfig::default(),
            organize_rules: Vec::new(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let s = ConfigLoader::builder()
            // Start with default values
            .set_default("rpc_url", "http://127.0.0.1:6800/jsonrpc")?
            .set_default("transport", "stdio")?
            .set_default("http_host", "0.0.0.0")?
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
        assert_eq!(config.rpc_url, "http://127.0.0.1:6800/jsonrpc");
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

    #[test]
    fn test_normalize_updates_default_instance() {
        let mut config = Config::default();
        assert_eq!(config.instances[0].rpc_url, "http://127.0.0.1:6800/jsonrpc");

        config.rpc_url = "http://192.168.1.10:6800/jsonrpc".to_string();
        config.normalize();

        // Normalize no longer updates existing instances to avoid overwriting user config
        assert_eq!(config.instances[0].rpc_url, "http://127.0.0.1:6800/jsonrpc");
    }

    #[test]
    fn test_normalize_does_not_overwrite_instance_with_default() {
        // Simulate a config file that has instances but not top-level rpc_url
        // The rpc_url will be the default 127.0.0.1
        let mut config = Config {
            rpc_url: "http://127.0.0.1:6800/jsonrpc".to_string(),
            instances: vec![Aria2Instance {
                name: "default".to_string(),
                rpc_url: "http://my-aria2:6800/jsonrpc".to_string(),
                rpc_secret: None,
            }],
            ..Default::default()
        };

        config.normalize();

        // It SHOULD NOT overwrite "my-aria2" with "127.0.0.1"
        assert_eq!(config.instances[0].rpc_url, "http://my-aria2:6800/jsonrpc");
    }

    #[test]
    fn test_rss_feed_history() {
        let mut feed = RSSFeed {
            url: "http://test".to_string(),
            name: "test".to_string(),
            filters: vec![],
            download_history: HashSet::new(),
        };

        assert!(!feed.has_downloaded("item1"));
        feed.mark_downloaded("item1".to_string());
        assert!(feed.has_downloaded("item1"));
        assert!(!feed.has_downloaded("item2"));
    }

    #[test]
    fn test_purge_config_defaults() {
        let config = Config::default();
        assert!(!config.purge_config.enabled);
        assert_eq!(config.purge_config.interval_secs, 3600);
        assert_eq!(config.purge_config.min_age_secs, 86400);
        assert!(config.purge_config.excluded_gids.is_empty());
    }
}
