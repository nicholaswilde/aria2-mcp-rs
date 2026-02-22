mod common;

use anyhow::Result;
use aria2_mcp_rs::resources::ResourceRegistry;
use aria2_mcp_rs::server::handler::McpHandler;
use aria2_mcp_rs::tools::registry::ToolRegistry;
use aria2_mcp_rs::Config;
use mcp_sdk_rs::server::ServerHandler;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_lazy_mode_initial_tools() -> Result<()> {
    let config = Config {
        lazy_mode: true,
        ..Default::default()
    };

    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let client = Arc::new(aria2_mcp_rs::aria2::Aria2Client::new(config.clone()));
    let handler = McpHandler::new(registry, resource_registry, vec![client]);

    let result = handler.handle_method("tools/list", None).await?;
    let tools = result["tools"].as_array().expect("Should have tools array");

    // Should have monitor_queue and manage_tools
    assert_eq!(tools.len(), 2);
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"monitor_queue"));
    assert!(names.contains(&"manage_tools"));
    assert!(!names.contains(&"manage_downloads"));

    Ok(())
}

#[tokio::test]
async fn test_lazy_mode_manage_tools_list() -> Result<()> {
    let config = Config {
        lazy_mode: true,
        ..Default::default()
    };

    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let client = Arc::new(aria2_mcp_rs::aria2::Aria2Client::new(config.clone()));
    let handler = McpHandler::new(registry, resource_registry, vec![client]);

    // Call a tool
    let params = serde_json::json!({
        "name": "manage_tools",
        "arguments": {
            "action": "list"
        }
    });

    let result = handler.handle_method("tools/call", Some(params)).await?;
    let content = result["content"][0]["text"]
        .as_str()
        .expect("Should have text content");
    let available: serde_json::Value = serde_json::from_str(content)?;

    assert!(available.is_array());
    assert_eq!(available.as_array().unwrap().len(), 11);

    Ok(())
}

#[tokio::test]
async fn test_lazy_mode_enable_tool() -> Result<()> {
    let config = Config {
        lazy_mode: true,
        ..Default::default()
    };

    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let client = Arc::new(aria2_mcp_rs::aria2::Aria2Client::new(config.clone()));
    let handler = McpHandler::new(registry, resource_registry, vec![client]);

    // Enable manage_downloads
    let params = serde_json::json!({
        "name": "manage_tools",
        "arguments": {
            "action": "enable",
            "tools": ["manage_downloads"]
        }
    });

    let result = handler.handle_method("tools/call", Some(params)).await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert_eq!(text, "Enabled 1 tools.");

    // Verify it's now in the list
    let result = handler.handle_method("tools/list", None).await?;
    let tools = result["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"manage_downloads"));
    assert_eq!(tools.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_lazy_mode_disable_tool() -> Result<()> {
    let config = Config {
        lazy_mode: true,
        ..Default::default()
    };

    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let client = Arc::new(aria2_mcp_rs::aria2::Aria2Client::new(config.clone()));
    let handler = McpHandler::new(registry.clone(), resource_registry, vec![client]);

    // Enable then disable
    let registry_clone = Arc::clone(&registry);
    {
        let mut reg = registry_clone.write().await;
        reg.enable_tool("manage_downloads");
    }

    let params = serde_json::json!({
        "name": "manage_tools",
        "arguments": {
            "action": "disable",
            "tools": ["manage_downloads"]
        }
    });

    handler.handle_method("tools/call", Some(params)).await?;

    // Verify it's gone
    let result = handler.handle_method("tools/list", None).await?;
    let tools = result["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(!names.contains(&"manage_downloads"));

    Ok(())
}

#[tokio::test]
async fn test_non_lazy_mode_no_manage_tools() -> Result<()> {
    let config = Config::default(); // lazy_mode is false by default

    let registry = Arc::new(RwLock::new(ToolRegistry::new(&config)));
    let resource_registry = Arc::new(RwLock::new(ResourceRegistry::default()));
    let client = Arc::new(aria2_mcp_rs::aria2::Aria2Client::new(config.clone()));
    let handler = McpHandler::new(registry, resource_registry, vec![client]);

    let result = handler.handle_method("tools/list", None).await?;
    let tools = result["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

    assert!(!names.contains(&"manage_tools"));
    assert_eq!(tools.len(), 11);

    Ok(())
}

#[tokio::test]
async fn test_lazy_mode_token_savings() -> Result<()> {
    // Normal Mode
    let config_normal = Config::default();
    let registry_normal = Arc::new(RwLock::new(ToolRegistry::new(&config_normal)));
    let resource_registry_normal = Arc::new(RwLock::new(ResourceRegistry::default()));
    let client_normal = Arc::new(aria2_mcp_rs::aria2::Aria2Client::new(config_normal.clone()));
    let handler_normal = McpHandler::new(
        registry_normal,
        resource_registry_normal,
        vec![client_normal],
    );

    let result_normal = handler_normal.handle_method("tools/list", None).await?;
    let size_normal = serde_json::to_string(&result_normal)?.len();

    // Lazy Mode
    let config_lazy = Config {
        lazy_mode: true,
        ..Default::default()
    };
    let registry_lazy = Arc::new(RwLock::new(ToolRegistry::new(&config_lazy)));
    let resource_registry_lazy = Arc::new(RwLock::new(ResourceRegistry::default()));
    let client_lazy = Arc::new(aria2_mcp_rs::aria2::Aria2Client::new(config_lazy.clone()));
    let handler_lazy = McpHandler::new(registry_lazy, resource_registry_lazy, vec![client_lazy]);

    let result_lazy = handler_lazy.handle_method("tools/list", None).await?;
    let size_lazy = serde_json::to_string(&result_lazy)?.len();

    println!("Normal Mode Tools List Size: {} bytes", size_normal);
    println!("Lazy Mode Tools List Size:   {} bytes", size_lazy);

    // Expect significant savings (more than 50%)
    assert!(size_lazy < size_normal);
    let savings_pct = (size_normal - size_lazy) as f64 / size_normal as f64;
    assert!(
        savings_pct > 0.5,
        "Savings should be more than 50%, got {:.2}%",
        savings_pct * 100.0
    );

    Ok(())
}
