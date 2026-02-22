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

impl Aria2Notification {
    pub fn to_mcp_notification(&self) -> serde_json::Value {
        let event_name = match self.method {
            Aria2Event::DownloadStart => "download_start",
            Aria2Event::DownloadPause => "download_pause",
            Aria2Event::DownloadStop => "download_stop",
            Aria2Event::DownloadComplete => "download_complete",
            Aria2Event::DownloadError => "download_error",
            Aria2Event::BtDownloadComplete => "bt_download_complete",
        };

        let gid = self.params.get(0).map(|p| p.gid.as_str()).unwrap_or("");

        serde_json::json!({
            "method": "notifications/aria2/event",
            "params": {
                "event": event_name,
                "gid": gid
            }
        })
    }
}

