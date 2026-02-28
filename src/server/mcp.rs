use crate::aria2::notifications::Aria2Notification;
use crate::aria2::Aria2Client;
use crate::prompts::PromptRegistry;
use crate::resources::ResourceRegistry;
use crate::tools::ToolRegistry;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<RequestId>,
}

impl JsonRpcResponse {
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: Some(id),
        }
    }

    pub fn error(id: RequestId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id: Some(id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn new(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: None,
        }
    }
}

pub struct McpState {
    pub running: bool,
    pub initialized: bool,
    pub lazy_mode: bool,
    pub notifications: VecDeque<JsonRpcRequest>,
}

impl McpState {
    pub fn new(lazy_mode: bool) -> Self {
        Self {
            running: true,
            initialized: false,
            lazy_mode,
            notifications: VecDeque::new(),
        }
    }

    pub fn queue_notification(&mut self, method: &str, params: Value) {
        self.notifications.push_back(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(params),
            id: None,
        });
    }

    pub fn pop_notification(&mut self) -> Option<JsonRpcRequest> {
        self.notifications.pop_front()
    }
}

pub async fn run_stdio(
    state: Arc<RwLock<McpState>>,
    registry: Arc<RwLock<ToolRegistry>>,
    resource_registry: Arc<RwLock<ResourceRegistry>>,
    prompt_registry: Arc<RwLock<PromptRegistry>>,
    clients: Vec<Arc<Aria2Client>>,
    mut notification_rx: Receiver<Aria2Notification>,
) -> Result<()> {
    let mut reader = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();

    loop {
        {
            let state_guard = state.read().await;
            if !state_guard.running {
                break;
            }
        }

        tokio::select! {
            line_res = reader.next_line() => {
                match line_res {
                    Ok(Some(line)) => {
                        let req: JsonRpcRequest = match serde_json::from_str(&line) {
                            Ok(r) => r,
                            Err(e) => {
                                let err_res = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: None,
                                    error: Some(JsonRpcError::new(-32700, &format!("Parse error: {}", e))),
                                    id: None,
                                };
                                let res_str = serde_json::to_string(&err_res)?;
                                stdout.write_all(res_str.as_bytes()).await?;
                                stdout.write_all(b"\n").await?;
                                stdout.flush().await?;
                                continue;
                            }
                        };

                        let res = handle_request(
                            req.clone(),
                            Arc::clone(&state),
                            Arc::clone(&registry),
                            Arc::clone(&resource_registry),
                            Arc::clone(&prompt_registry),
                            &clients
                        ).await?;

                        if let Some(resp) = res {
                            let res_str = serde_json::to_string(&resp)?;
                            stdout.write_all(res_str.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                    }
                    Ok(None) => break, // EOF
                    Err(e) => return Err(anyhow::anyhow!("Stdin error: {}", e)),
                }
            }
            Some(notif) = notification_rx.recv() => {
                let mut state_guard = state.write().await;
                state_guard.queue_notification("notifications/message", json!({
                    "level": "info",
                    "data": format!("Aria2 Event: {:?}", notif)
                }));
            }
            _ = sleep(Duration::from_millis(100)) => {
                flush_notifications_async(Arc::clone(&state), &mut stdout).await?;
            }
        }
    }

    Ok(())
}

pub async fn flush_notifications_async<W: AsyncWriteExt + Unpin>(
    state: Arc<RwLock<McpState>>,
    writer: &mut W,
) -> Result<()> {
    let mut state_guard = state.write().await;
    while let Some(notif) = state_guard.pop_notification() {
        let notif_str = serde_json::to_string(&notif)?;
        writer.write_all(notif_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }
    writer.flush().await?;
    Ok(())
}

pub async fn handle_request(
    req: JsonRpcRequest,
    state: Arc<RwLock<McpState>>,
    registry: Arc<RwLock<ToolRegistry>>,
    resource_registry: Arc<RwLock<ResourceRegistry>>,
    prompt_registry: Arc<RwLock<PromptRegistry>>,
    clients: &[Arc<Aria2Client>],
) -> Result<Option<JsonRpcResponse>> {
    let id = match req.id {
        Some(id) => id,
        None => return Ok(None), // Notification
    };

    let result = match req.method.as_str() {
        "initialize" => {
            let mut state_guard = state.write().await;
            state_guard.initialized = true;
            Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "aria2-mcp-rs",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }))
        }
        "tools/list" => {
            let reg = registry.read().await;
            let tools = reg.list_tools();
            let mut tool_info = Vec::new();
            for tool in tools {
                tool_info.push(json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "inputSchema": tool.schema().unwrap_or(json!({"type": "object"}))
                }));
            }
            Ok(json!({ "tools": tool_info }))
        }
        "tools/call" => {
            let params = req.params.unwrap_or(json!({}));
            let name = params["name"].as_str().unwrap_or_default();
            let args = params["arguments"].clone();
            let reg = registry.read().await;
            if let Some(tool) = reg.get_tool(name) {
                match tool.run_multi(clients, args).await {
                    Ok(res) => Ok(json!(res)),
                    Err(e) => Err(JsonRpcError::new(-32603, &format!("Tool error: {}", e))),
                }
            } else {
                Err(JsonRpcError::new(-32601, "Tool not found"))
            }
        }
        "resources/list" => {
            let reg = resource_registry.read().await;
            let resources = reg.list_resources();
            Ok(json!({ "resources": resources }))
        }
        "resources/read" => {
             let params = req.params.unwrap_or(json!({}));
             let uri = params["uri"].as_str().unwrap_or_default();
             let reg = resource_registry.read().await;
             if let Some(resource) = reg.get_resource(uri) {
                 // Check if ResourceRegistry has read_resource or we call it on the resource itself.
                 // ResourceRegistry has get_resource which returns McpResource.
                 // Need to find which client to use for McpResource::read.
                 // Use first client for now as default.
                 if let Some(client) = clients.first() {
                    match resource.read(client).await {
                        Ok(res) => Ok(json!(res)),
                        Err(e) => Err(JsonRpcError::new(-32603, &format!("Resource error: {}", e))),
                    }
                 } else {
                    Err(JsonRpcError::new(-32603, "No clients available for resource read"))
                 }
             } else {
                Err(JsonRpcError::new(-32601, "Resource not found"))
             }
        }
        "prompts/list" => {
            let reg = prompt_registry.read().await;
            let prompts = reg.list_prompts();
            Ok(json!({ "prompts": prompts }))
        }
        "prompts/get" => {
            let params = req.params.unwrap_or(json!({}));
            let name = params["name"].as_str().unwrap_or_default();
            let args = params["arguments"].clone();
            let reg = prompt_registry.read().await;
            if let Some(prompt) = reg.get_prompt(name) {
                match prompt.get_messages(args) {
                    Ok(res) => Ok(json!({ "messages": res })),
                    Err(e) => Err(JsonRpcError::new(-32603, &format!("Prompt error: {}", e))),
                }
            } else {
                Err(JsonRpcError::new(-32601, "Prompt not found"))
            }
        }
        _ => Err(JsonRpcError::new(-32601, "Method not found")),
    };

    match result {
        Ok(res) => Ok(Some(JsonRpcResponse::success(id, res))),
        Err(e) => Ok(Some(JsonRpcResponse::error(id, e))),
    }
}
