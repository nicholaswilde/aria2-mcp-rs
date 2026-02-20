use crate::Config;
use anyhow::Result;
use reqwest::Client;

pub struct Aria2Client {
    config: Config,
    client: Client,
}

impl Aria2Client {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn get_version(&self) -> Result<String> {
        // Placeholder for real JSON-RPC call
        Ok("1.36.0".to_string())
    }
}
