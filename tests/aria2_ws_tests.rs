use aria2_mcp_rs::aria2::Aria2Client;
use aria2_mcp_rs::config::Config;

#[tokio::test]
async fn test_aria2_client_ws_connection() {
    // This test will attempt to call a non-existent method or use functionality not yet implemented
    let config = Config {
        rpc_url: "http://127.0.0.1:6800/jsonrpc".to_string(),
        ..Default::default()
    };
    let client = Aria2Client::new(config);

    assert_eq!(client.ws_url().unwrap(), "ws://127.0.0.1:6800/jsonrpc");
}

#[tokio::test]
async fn test_aria2_client_ws_url_https() {
    let config = Config {
        rpc_url: "https://127.0.0.1:6800/jsonrpc".to_string(),
        ..Default::default()
    };
    let client = Aria2Client::new(config);

    assert_eq!(client.ws_url().unwrap(), "wss://127.0.0.1:6800/jsonrpc");
}

#[tokio::test]
async fn test_aria2_client_connect_notifications() {
    let server = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap();

    let config = Config {
        rpc_url: format!("http://{}", addr),
        ..Default::default()
    };
    let client = Aria2Client::new(config);

    // Start a simple task to accept the connection
    tokio::spawn(async move {
        let (stream, _) = server.accept().await.unwrap();
        let _ = tokio_tungstenite::accept_async(stream).await;
    });

    let result = client.connect_notifications().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_aria2_client_receive_notification() {
    use futures_util::SinkExt;
    use tokio_tungstenite::tungstenite::protocol::Message;

    let server = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap();

    let config = Config {
        rpc_url: format!("http://{}", addr),
        ..Default::default()
    };
    let client = Aria2Client::new(config);

    tokio::spawn(async move {
        let (stream, _) = server.accept().await.unwrap();
        let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();

        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "aria2.onDownloadComplete",
            "params": [{"gid": "123"}]
        });

        ws.send(Message::Text(notification.to_string().into()))
            .await
            .unwrap();
    });

    let (tx, mut rx) =
        tokio::sync::mpsc::channel::<aria2_mcp_rs::aria2::notifications::Aria2Notification>(10);
    client.start_notifications(tx).await.unwrap();

    let received = rx.recv().await.unwrap();
    assert_eq!(
        received.method,
        aria2_mcp_rs::aria2::notifications::Aria2Event::DownloadComplete
    );
}

#[test]
fn test_parse_aria2_notification() {
    use aria2_mcp_rs::aria2::notifications::{Aria2Event, Aria2Notification};

    let json = r#"{
        "jsonrpc": "2.0",
        "method": "aria2.onDownloadComplete",
        "params": [{"gid": "123"}]
    }"#;

    let notification: Aria2Notification = serde_json::from_str(json).unwrap();
    assert_eq!(notification.method, Aria2Event::DownloadComplete);
    assert_eq!(notification.params[0].gid, "123");
}

#[test]
fn test_to_mcp_notification() {
    use aria2_mcp_rs::aria2::notifications::{Aria2Event, Aria2EventParams, Aria2Notification};

    let notification = Aria2Notification {
        jsonrpc: "2.0".to_string(),
        method: Aria2Event::DownloadComplete,
        params: vec![Aria2EventParams {
            gid: "123".to_string(),
        }],
    };

    let mcp = notification.to_mcp_notification();
    assert_eq!(mcp["method"], "notifications/aria2/event");
    assert_eq!(mcp["params"]["event"], "download_complete");
    assert_eq!(mcp["params"]["gid"], "123");
}
