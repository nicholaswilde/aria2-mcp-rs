# Specification - Implement Stdio JSON-RPC 2.0 Server

## Overview
Implement a native Model Context Protocol (MCP) server for `aria2-mcp-rs` that communicates using JSON-RPC 2.0 over standard input/output (stdio). This implementation will replace or augment the existing `mcp-sdk-rs` based server with a custom, high-performance asynchronous event loop, following the pattern established in `qbittorrent-mcp-rs`.

## Functional Requirements
- **Custom Stdio Transport**: Implement a persistent asynchronous loop using `tokio` to read from `stdin` and write to `stdout`.
- **JSON-RPC 2.0 Compliance**:
    - Support `jsonrpc`, `method`, `params`, and `id` fields.
    - Handle both requests (with `id`) and notifications (without `id`).
    - Respond to `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, and `prompts/get`.
- **State Management**: Use a thread-safe state container (`Arc<RwLock<...>>`) to track server status and notification queues.
- **Asynchronous Notifications**:
    - Implement a background task to monitor `aria2` events (e.g., download start, completion, error).
    - Queue events as MCP notifications.
    - Periodically flush notifications to `stdout` in the main loop using `tokio::select!`.
- **Standardized Error Handling**: Return JSON-RPC error objects with standard codes (-32700 to -32000) and descriptive messages.

## Non-Functional Requirements
- **Performance**: Ensure the event loop is non-blocking and highly responsive.
- **Reliability**: Target >90% code coverage for the new server and routing logic.
- **Consistency**: Align implementation patterns (routing, notifications, error handling) with `qbittorrent-mcp-rs`.

## Acceptance Criteria
- [ ] Server successfully completes the MCP handshake (`initialize`).
- [ ] All existing MCP tools, resources, and prompts are accessible via stdio.
- [ ] Real-time notifications for `aria2` events are correctly pushed to the client.
- [ ] Invalid JSON or non-compliant JSON-RPC requests return appropriate error objects.
- [ ] Server gracefully handles shutdown requests.

## Out of Scope
- Implementation of new `aria2` tools or resources (only the transport/routing layer is in scope).
- Changes to the HTTP/SSE transport (unless necessary for shared logic).
