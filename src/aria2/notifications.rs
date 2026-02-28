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
    #[must_use]
    pub fn to_mcp_notification(&self) -> serde_json::Value {
        let event_name = match self.method {
            Aria2Event::DownloadStart => "download_start",
            Aria2Event::DownloadPause => "download_pause",
            Aria2Event::DownloadStop => "download_stop",
            Aria2Event::DownloadComplete => "download_complete",
            Aria2Event::DownloadError => "download_error",
            Aria2Event::BtDownloadComplete => "bt_download_complete",
        };

        let gid = self.params.first().map_or("", |p| p.gid.as_str());

        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/aria2/event",
            "params": {
                "event": event_name,
                "gid": gid
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_mcp_notification_variants() {
        let events = vec![
            (Aria2Event::DownloadStart, "download_start"),
            (Aria2Event::DownloadPause, "download_pause"),
            (Aria2Event::DownloadStop, "download_stop"),
            (Aria2Event::DownloadComplete, "download_complete"),
            (Aria2Event::DownloadError, "download_error"),
            (Aria2Event::BtDownloadComplete, "bt_download_complete"),
        ];

        for (event, expected_name) in events {
            let notification = Aria2Notification {
                jsonrpc: "2.0".to_string(),
                method: event,
                params: vec![Aria2EventParams {
                    gid: "123".to_string(),
                }],
            };
            let mcp = notification.to_mcp_notification();
            assert_eq!(mcp["params"]["event"], expected_name);
            assert_eq!(mcp["params"]["gid"], "123");
        }
    }

    #[test]
    fn test_to_mcp_notification_no_params() {
        let notification = Aria2Notification {
            jsonrpc: "2.0".to_string(),
            method: Aria2Event::DownloadStart,
            params: vec![],
        };
        let mcp = notification.to_mcp_notification();
        assert_eq!(mcp["params"]["gid"], "");
    }
}
