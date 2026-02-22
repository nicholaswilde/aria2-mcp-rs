use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Aria2Event {
    #[serde(rename = "aria2.onDownloadStart")]
    DownloadStart,
    #[serde(rename = "aria2.onDownloadPause")]
    DownloadPause,
    #[serde(rename = "aria2.onDownloadStop")]
    DownloadStop,
    #[serde(rename = "aria2.onDownloadComplete")]
    DownloadComplete,
    #[serde(rename = "aria2.onDownloadError")]
    DownloadError,
    #[serde(rename = "aria2.onBtDownloadComplete")]
    BtDownloadComplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aria2Notification {
    pub jsonrpc: String,
    pub method: Aria2Event,
    pub params: Vec<Aria2EventParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aria2EventParams {
    pub gid: String,
}

