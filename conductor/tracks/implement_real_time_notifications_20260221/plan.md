# Implementation Plan - Implement Real-Time Notifications

## Phase 1: WebSocket Client Integration
- [ ] Task: Add `tokio-tungstenite` or similar crate for WebSocket support.
- [ ] Task: Update `Aria2Client` to establish and maintain a WebSocket connection.
- [ ] Task: Implement a background task to read from the WebSocket stream.

## Phase 2: Event Mapping & Dispatch
- [ ] Task: Define internal event structures for aria2 notifications.
- [ ] Task: Implement mapping logic from aria2 events to MCP notification format.
- [ ] Task: Update `McpServer` to broadcast notifications to active transport channels.

## Phase 3: Robustness & Testing
- [ ] Task: Implement exponential backoff for reconnection logic.
- [ ] Task: Add integration tests simulating aria2 events and verifying MCP output.
- [ ] Task: Verify behavior with multiple aria2 instances.