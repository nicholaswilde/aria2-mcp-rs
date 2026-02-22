use aria2_mcp_rs::tools::registry::ToolRegistry;
use aria2_mcp_rs::config::Config;

#[tokio::test]
async fn test_manage_all_instances_tool_exists() {
    let config = Config::default();
    let registry = ToolRegistry::new(&config);
    
    let tool = registry.get_tool("manage_all_instances");
    assert!(tool.is_some(), "manage_all_instances tool should be registered");
}

#[tokio::test]
async fn test_manage_all_instances_schema() {
    let config = Config::default();
    let registry = ToolRegistry::new(&config);
    let tool = registry.get_tool("manage_all_instances").unwrap();
    let schema = tool.schema().unwrap();
    
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["action"].is_object());
    assert!(schema["required"].as_array().unwrap().contains(&serde_json::json!("action")));
}
