use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
    Json,
    Extension,
};
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::aria2::Aria2Client;
use crate::tools::ToolRegistry;

pub async fn run_server(port: u16, registry: Arc<ToolRegistry>, client: Arc<Aria2Client>) -> Result<()> {
    let app = Router::new()
        .route("/tools", get(list_tools))
        .route("/tools/execute", post(execute_tool))
        .layer(Extension(registry))
        .layer(Extension(client));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(&addr).await?;
    
    println!("SSE Server starting on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn list_tools(
    Extension(registry): Extension<Arc<ToolRegistry>>,
) -> Json<serde_json::Value> {
    let tools = registry.list_tools();
    let infos: Vec<_> = tools.iter().map(|t| serde_json::json!({
        "name": t.name(),
        "description": t.description(),
        "inputSchema": t.input_schema(),
    })).collect();
    
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
        match tool.execute(&client, args).await {
            Ok(result) => Json(serde_json::json!({ "content": [{ "type": "text", "text": result.to_string() }] })),
            Err(e) => Json(serde_json::json!({ "isError": true, "content": [{ "type": "text", "text": e.to_string() }] })),
        }
    } else {
        Json(serde_json::json!({ "isError": true, "content": [{ "type": "text", "text": format!("Tool not found: {}", name) }] }))
    }
}
