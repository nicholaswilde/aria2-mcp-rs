use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub rpc_secret: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:6800/jsonrpc".to_string(),
            rpc_secret: None,
        }
    }
}

impl Config {
    pub fn new(rpc_url: String, rpc_secret: Option<String>) -> Self {
        Self {
            rpc_url,
            rpc_secret,
        }
    }
}
