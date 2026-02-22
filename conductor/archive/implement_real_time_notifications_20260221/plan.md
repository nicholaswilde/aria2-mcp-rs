# Implementation Plan - Implement Real-Time Notifications

## Phase 1: WebSocket Client Integration
- [x] Task: Add `tokio-tungstenite` or similar crate for WebSocket support. 73b847a
- [x] Task: Update `Aria2Client` to establish and maintain a WebSocket connection. fa90f06
- [x] Task: Implement a background task to read from the WebSocket stream. aac518a

## Phase 2: Event Mapping & Dispatch
- [x] Task: Define internal event structures for aria2 notifications. e86902a
- [x] Task: Implement mapping logic from aria2 events to MCP notification format. 8a5db6c
- [x] Task: Update `McpServer` to broadcast notifications to active transport channels. ba651f2

## Phase 3: Robustness & Testing [checkpoint: 9a219f0]
- [x] Task: Implement exponential backoff for reconnection logic. 109b9a2
- [x] Task: Add integration tests simulating aria2 events and verifying MCP output. 1294c9d
- [x] Task: Verify behavior with multiple aria2 instances. 044db61