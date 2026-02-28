use aria2_mcp_rs::server::mcp::{JsonRpcRequest, JsonRpcResponse, JsonRpcError, RequestId, McpState};
use serde_json::json;

#[test]
fn test_deserialize_request_with_id() {
    let data = json!({
        "jsonrpc": "2.0",
        "method": "test_method",
        "params": {"key": "value"},
        "id": 1
    });
    let req: JsonRpcRequest = serde_json::from_value(data).unwrap();
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "test_method");
    assert_eq!(req.params.unwrap()["key"], "value");
    assert_eq!(req.id, Some(RequestId::Number(1)));
}

#[test]
fn test_deserialize_notification() {
    let data = json!({
        "jsonrpc": "2.0",
        "method": "test_notification",
        "params": ["a", "b"]
    });
    let req: JsonRpcRequest = serde_json::from_value(data).unwrap();
    assert_eq!(req.method, "test_notification");
    assert_eq!(req.id, None);
}

#[test]
fn test_serialize_success_response() {
    let res = JsonRpcResponse::success(RequestId::String("abc".to_string()), json!({"ok": true}));
    let data = serde_json::to_value(res).unwrap();
    assert_eq!(data["jsonrpc"], "2.0");
    assert_eq!(data["id"], "abc");
    assert_eq!(data["result"]["ok"], true);
    assert!(data.get("error").is_none());
}

#[test]
fn test_serialize_error_response() {
    let err = JsonRpcError::new(-32601, "Method not found");
    let res = JsonRpcResponse::error(RequestId::Number(42), err);
    let data = serde_json::to_value(res).unwrap();
    assert_eq!(data["id"], 42);
    assert_eq!(data["error"]["code"], -32601);
    assert_eq!(data["error"]["message"], "Method not found");
    assert!(data.get("result").is_none());
}

#[test]
fn test_mcp_state_notifications() {
    let mut state = McpState::new(false);
    assert!(state.running);
    assert!(!state.initialized);
    assert!(!state.lazy_mode);
    
    state.queue_notification("test_event", json!({"info": "data"}));
    assert_eq!(state.notifications.len(), 1);
    
    let notif = state.pop_notification().unwrap();
    assert_eq!(notif.method, "test_event");
    assert_eq!(notif.params.unwrap()["info"], "data");
    assert!(state.pop_notification().is_none());
}
