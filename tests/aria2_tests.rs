use aria2_mcp_rs::aria2::Aria2Client;
use aria2_mcp_rs::Config;

#[tokio::test]
async fn test_aria2_client_new() {
    let config = Config::default();
    let client = Aria2Client::new(config);
    let version = client.get_version().await.unwrap();
    assert_eq!(version, "1.36.0");
}
