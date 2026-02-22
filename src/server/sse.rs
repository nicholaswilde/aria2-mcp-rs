use anyhow::Result;
use axum::{
    extract::{Json, Request},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::aria2::Aria2Client;
use crate::tools::ToolRegistry;

pub async fn run_server(
    http_port: u16,
    http_auth_token: Option<String>,
    registry: Arc<RwLock<ToolRegistry>>,
    clients: Vec<Arc<Aria2Client>>,
) -> Result<()> {
    let mut app = Router::new()
        .route("/tools", get(list_tools))
        .route("/tools/execute", post(execute_tool))
        .layer(Extension(registry))
        .layer(Extension(clients));

    if let Some(token) = http_auth_token {
        app = app.layer(middleware::from_fn(move |req, next| {
            auth_middleware(req, next, token.clone())
        }));
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], http_port));
    let listener = TcpListener::bind(addr).await?;

    println!("SSE Server starting on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn auth_middleware(
    req: Request,
    next: Next,
    required_token: String,
) -> Result<Response, Response> {
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let expected_auth = format!("Bearer {}", required_token);

    if let Some(auth) = auth_header {
        if auth == expected_auth {
            return Ok(next.run(req).await);
        }
    }

    Err((
        axum::http::StatusCode::UNAUTHORIZED,
        "Unauthorized: Invalid or missing Bearer token",
    )
        .into_response())
}

async fn list_tools(
    Extension(registry): Extension<Arc<RwLock<ToolRegistry>>>,
) -> Json<serde_json::Value> {
    let registry = registry.read().await;
    let tools = registry.list_tools();
    let mut infos: Vec<_> = tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name(),
                "description": t.description(),
                "inputSchema": t.schema().unwrap_or(serde_json::json!({})),
            })
        })
        .collect();

    if registry.is_lazy_mode() {
        infos.push(serde_json::json!({
            "name": "manage_tools",
            "description": "Manage available tools (enable/disable) to save tokens.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list", "enable", "disable"],
                        "description": "The action to perform."
                    },
                    "tools": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of tool names to enable or disable."
                    }
                },
                "required": ["action"]
            }
        }));
    }

    Json(serde_json::json!({ "tools": infos }))
}

async fn execute_tool(
    Extension(registry): Extension<Arc<RwLock<ToolRegistry>>>,
    Extension(clients): Extension<Vec<Arc<Aria2Client>>>,
    Json(req): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let name = req["name"].as_str().unwrap_or_default();
    let args = req["arguments"].clone();

    let instance_idx = args.get("instance").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let client = match clients.get(instance_idx) {
        Some(c) => c,
        None => {
            return Json(serde_json::json!({
                "isError": true,
                "content": [{
                    "type": "text",
                    "text": format!("Invalid instance index: {}", instance_idx)
                }]
            }));
        }
    };

    if name == "manage_tools" {
        let registry_guard = registry.read().await;
        if registry_guard.is_lazy_mode() {
            drop(registry_guard);
            // We need a way to call handle_manage_tools here or duplicate the logic.
            // Since sse.rs is simple, let's just implement a basic version or
            // refactor handler.rs to expose the logic.
            // For now, let's just say it's not supported in SSE if we want to be quick,
            // but that's not good.
            // Let's implement it here.
            let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
            match action {
                "list" => {
                    let registry = registry.read().await;
                    let tools = registry.list_available_tools();
                    return Json(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&tools).unwrap_or_default()
                        }]
                    }));
                }
                "enable" => {
                    let tools_to_enable = args.get("tools").and_then(|v| v.as_array());
                    if let Some(tools) = tools_to_enable {
                        let mut registry = registry.write().await;
                        let mut enabled_count = 0;
                        for t in tools {
                            if let Some(name) = t.as_str() {
                                if registry.enable_tool(name) {
                                    enabled_count += 1;
                                }
                            }
                        }
                        return Json(serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Enabled {} tools.", enabled_count)
                            }]
                        }));
                    }
                }
                "disable" => {
                    let tools_to_disable = args.get("tools").and_then(|v| v.as_array());
                    if let Some(tools) = tools_to_disable {
                        let mut registry = registry.write().await;
                        let mut disabled_count = 0;
                        for t in tools {
                            if let Some(name) = t.as_str() {
                                if registry.disable_tool(name) {
                                    disabled_count += 1;
                                }
                            }
                        }
                        return Json(serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Disabled {} tools.", disabled_count)
                            }]
                        }));
                    }
                }
                _ => {}
            }
        }
    }

    let registry = registry.read().await;
    if let Some(tool) = registry.get_tool(name) {
        if !registry.is_tool_enabled(name) {
            return Json(
                serde_json::json!({ "isError": true, "content": [{ "type": "text", "text": format!("Tool not found or not enabled: {}", name) }] }),
            );
        }
        drop(registry);
        match tool.run(client, args).await {
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
        let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
        let result = list_tools(axum::extract::Extension(registry)).await;
        let body = serde_json::to_value(result.0).unwrap();
        let tools = body["tools"].as_array().unwrap();
        assert!(!tools.is_empty());
    }

    #[tokio::test]
    async fn test_execute_tool_not_found() {
        let registry = Arc::new(RwLock::new(ToolRegistry::new(&Config::default())));
        let client = Arc::new(Aria2Client::new(Config::default()));
        let req = serde_json::json!({
            "name": "unknown",
            "arguments": {}
        });
        let result = execute_tool(
            axum::extract::Extension(registry),
            axum::extract::Extension(vec![client]),
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
        let mut registry = ToolRegistry::new(&Config::default());
        registry.register(Arc::new(DummyTool));
        let registry = Arc::new(RwLock::new(registry));
        let client = Arc::new(Aria2Client::new(Config::default()));
        let req = serde_json::json!({
            "name": "dummy",
            "arguments": {}
        });
        let result = execute_tool(
            axum::extract::Extension(registry),
            axum::extract::Extension(vec![client]),
            Json(req),
        )
        .await;
        let body = serde_json::to_value(result.0).unwrap();
        assert_eq!(
            body["content"][0]["text"].as_str().unwrap(),
            "{\"status\":\"ok\"}"
        );
    }
}
