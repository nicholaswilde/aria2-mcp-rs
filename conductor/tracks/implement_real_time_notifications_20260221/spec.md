# Track: Implement Real-Time Notifications

## Overview
This track enhances the `aria2-mcp-rs` server by implementing real-time notifications. It will listen to aria2's WebSocket event stream (e.g., download complete, error, start) and forward these as MCP notifications to the client. This allows for proactive updates instead of polling.

## Functional Requirements
- **WebSocket Client**: Implement a WebSocket client in `Aria2Client` to connect to aria2's notification stream.
- **Event Handling**: Map aria2 events (`onDownloadStart`, `onDownloadPause`, `onDownloadStop`, `onDownloadComplete`, `onDownloadError`) to MCP notifications.
- **Notification Dispatch**: Update `McpServer` to dispatch notifications to connected MCP clients (SSE/Stdio).
- **Configuration**: Ensure WebSocket URL is derived correctly from RPC URL.

## Non-Functional Requirements
- **Reliability**: Auto-reconnect if the WebSocket connection drops.
- **Performance**: Notifications should be lightweight and not block other operations.

## Acceptance Criteria
- [ ] Connects to aria2 WebSocket stream upon startup.
- [ ] Receives and parses aria2 notification messages.
- [ ] Dispatches `notifications/post` (or equivalent MCP method) with event details.
- [ ] Reconnects on connection loss.
- [ ] Integration test verifies notification flow.