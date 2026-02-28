# Implementation Plan - Implement Stdio JSON-RPC 2.0 Server

## Phase 1: Foundation & Types
- [x] Task: Define JSON-RPC 2.0 Request, Response, and Error types to match MCP specification.
    - [x] Add `JsonRpcRequest`, `JsonRpcResponse`, and `JsonRpcError` structs.
    - [x] Implement serialization/deserialization with `serde`.
- [x] Task: Create `McpState` struct to hold the shared server state.
    - [x] Include flags for `running`, `initialized`, and `lazy_mode`.
    - [x] Add a notification queue (`VecDeque` or similar).

## Phase 2: Core Stdio Transport & Event Loop
- [x] Task: Implement the `run_stdio` asynchronous event loop.
    - [x] Use `tokio::io::stdin` and `tokio::io::stdout`.
    - [x] Use `tokio::select!` to multiplex between reading stdin and flushing notifications.
- [x] Task: Implement request routing logic.
    - [x] Route `initialize` and lifecycle methods.
    - [x] Delegate `tools/*`, `resources/*`, and `prompts/*` to existing registries.
- [x] Task: Conductor - User Manual Verification 'Phase 2: Core Stdio Transport & Event Loop' (Protocol in workflow.md) (checkpoint: 38c30ec)

## Phase 3: Notifications & Background Tasks
- [x] Task: Refactor notification handling to support the new stdio push mechanism.
    - [x] Implement `flush_notifications_async` to write queued events to `stdout`.
    - [x] Ensure `aria2` event listeners correctly queue notifications in `McpState`.
- [x] Task: Integrate the new server into `main.rs` as the default stdio transport.
- [x] Task: Conductor - User Manual Verification 'Phase 3: Notifications & Background Tasks' (Protocol in workflow.md) (checkpoint: 1fe22a3)

## Phase 4: Validation & Cleanup
- [x] Task: Write unit tests for JSON-RPC parsing and routing. (checkpoint: 1fe22a3)
- [x] Task: Write integration tests for the stdio transport using `tokio-test` or similar. (checkpoint: b1c90c5)
- [x] Task: Final project-wide checks with `/test-fix`. (checkpoint: b1c90c5)
- [x] Task: Conductor - User Manual Verification 'Phase 4: Validation & Cleanup' (Protocol in workflow.md) (checkpoint: b1c90c5)
