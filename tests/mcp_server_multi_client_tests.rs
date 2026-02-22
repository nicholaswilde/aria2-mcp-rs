use aria2_mcp_rs::{Aria2Client, Config, McpServer, ToolRegistry};

#[test]
fn test_mcp_server_multi_client_initialization() {
    let mut config = Config::default();
    config.instances = vec![
        aria2_mcp_rs::config::Aria2Instance {
            name: "instance1".to_string(),
            rpc_url: "http://localhost:6800/jsonrpc".to_string(),
            rpc_secret: None,
        },
        aria2_mcp_rs::config::Aria2Instance {
            name: "instance2".to_string(),
            rpc_url: "http://localhost:6801/jsonrpc".to_string(),
            rpc_secret: Some("secret".to_string()),
        },
    ];

    let registry = ToolRegistry::new(&config);

    // This should change to handle multiple clients
    let clients = config
        .instances
        .iter()
        .map(|instance_config| {
            // We might need a way to create client from instance config
            // Or just let McpServer handle it
            Aria2Client::new_with_instance(config.clone(), instance_config.clone())
        })
        .collect::<Vec<_>>();

    let server = McpServer::new(config, registry, clients);

    assert_eq!(server.clients().len(), 2);
}
