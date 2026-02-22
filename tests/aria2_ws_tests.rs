use aria2_mcp_rs::aria2::Aria2Client;
use aria2_mcp_rs::config::Config;

#[tokio::test]
async fn test_aria2_client_ws_connection() {
    // This test will attempt to call a non-existent method or use functionality not yet implemented
    let config = Config {
        rpc_url: "http://localhost:6800/jsonrpc".to_string(),
        ..Default::default()
    };
    let client = Aria2Client::new(config);
    
    assert_eq!(client.ws_url().unwrap(), "ws://localhost:6800/jsonrpc");
}

#[tokio::test]
async fn test_aria2_client_ws_url_https() {
    let config = Config {
        rpc_url: "https://localhost:6800/jsonrpc".to_string(),
        ..Default::default()
    };
    let client = Aria2Client::new(config);
    
    assert_eq!(client.ws_url().unwrap(), "wss://localhost:6800/jsonrpc");
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
