use anyhow::Result;
use axum::{
    extract::Json,
    routing::{get, post},
    Extension, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::aria2::Aria2Client;
use crate::tools::ToolRegistry;

pub async fn run_server(
    port: u16,
    registry: Arc<ToolRegistry>,
    client: Arc<Aria2Client>,
) -> Result<()> {
    let app = Router::new()
        .route("/tools", get(list_tools))
        .route("/tools/execute", post(execute_tool))
        .layer(Extension(registry))
        .layer(Extension(client));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    println!("SSE Server starting on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn list_tools(Extension(registry): Extension<Arc<ToolRegistry>>) -> Json<serde_json::Value> {
    let tools = registry.list_tools();
    let infos: Vec<_> = tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name(),
                "description": t.description(),
                "inputSchema": t.schema().unwrap_or(serde_json::json!({})),
            })
        })
        .collect();

    Json(serde_json::json!({ "tools": infos }))
}

async fn execute_tool(
    Extension(registry): Extension<Arc<ToolRegistry>>,
    Extension(client): Extension<Arc<Aria2Client>>,
    Json(req): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let name = req["name"].as_str().unwrap_or_default();
    let args = req["arguments"].clone();

    if let Some(tool) = registry.get_tool(name) {
        match tool.run(&client, args).await {
            Ok(result) => Json(
                serde_json::json!({ "content": [{ "type": "text", "text": result.to_string() }] }),
            ),
            Err(e) => Json(
                serde_json::json!({ "isError": true, "content": [{ "type": "text", "text": e.to_string() }] }),
            ),
        }
    } else {
        Json(
            serde_json::json!({ "isError": true, "content": [{ "type": "text", "text": format!("Tool not found: {}", name) }] }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_list_tools() {
        let registry = Arc::new(ToolRegistry::new());
        let result = list_tools(axum::extract::Extension(registry)).await;
        let body = serde_json::to_value(result.0).unwrap();
        let tools = body["tools"].as_array().unwrap();
        assert!(!tools.is_empty());
    }

    #[tokio::test]
    async fn test_execute_tool_not_found() {
        let registry = Arc::new(ToolRegistry::new());
        let client = Arc::new(Aria2Client::new(Config::default()));
        let req = serde_json::json!({
            "name": "unknown",
            "arguments": {}
        });
        let result = execute_tool(
            axum::extract::Extension(registry),
            axum::extract::Extension(client),
            Json(req),
        )
        .await;
        let body = serde_json::to_value(result.0).unwrap();
        assert!(body["isError"].as_bool().unwrap());
        assert_eq!(
            body["content"][0]["text"].as_str().unwrap(),
            "Tool not found: unknown"
        );
    }

    struct DummyTool;
    #[async_trait::async_trait]
    impl crate::tools::registry::McpeTool for DummyTool {
        fn name(&self) -> String {
            "dummy".to_string()
        }
        fn description(&self) -> String {
            "dummy".to_string()
        }
        fn schema(&self) -> Result<serde_json::Value> {
            Ok(serde_json::json!({}))
        }
        async fn run(
            &self,
            _client: &crate::aria2::Aria2Client,
            _args: serde_json::Value,
        ) -> Result<serde_json::Value> {
            Ok(serde_json::json!({"status": "ok"}))
        }
    }

    #[tokio::test]
    async fn test_execute_tool_success() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(DummyTool));
        let registry = Arc::new(registry);
        let client = Arc::new(Aria2Client::new(Config::default()));
        let req = serde_json::json!({
            "name": "dummy",
            "arguments": {}
        });
        let result = execute_tool(
            axum::extract::Extension(registry),
            axum::extract::Extension(client),
            Json(req),
        )
        .await;
        let body = serde_json::to_value(result.0).unwrap();
        assert_eq!(body["content"][0]["text"].as_str().unwrap(), "{\"status\":\"ok\"}");
    }
}
