# Implementation Plan - Implement Stdio JSON-RPC 2.0 Server

## Phase 1: Foundation & Types
- [ ] Task: Define JSON-RPC 2.0 Request, Response, and Error types to match MCP specification.
    - [ ] Add `JsonRpcRequest`, `JsonRpcResponse`, and `JsonRpcError` structs.
    - [ ] Implement serialization/deserialization with `serde`.
- [ ] Task: Create `McpState` struct to hold the shared server state.
    - [ ] Include flags for `running`, `initialized`, and `lazy_mode`.
    - [ ] Add a notification queue (`VecDeque` or similar).

## Phase 2: Core Stdio Transport & Event Loop
- [ ] Task: Implement the `run_stdio` asynchronous event loop.
    - [ ] Use `tokio::io::stdin` and `tokio::io::stdout`.
    - [ ] Use `tokio::select!` to multiplex between reading stdin and flushing notifications.
- [ ] Task: Implement request routing logic.
    - [ ] Route `initialize` and lifecycle methods.
    - [ ] Delegate `tools/*`, `resources/*`, and `prompts/*` to existing registries.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Core Stdio Transport & Event Loop' (Protocol in workflow.md)

## Phase 3: Notifications & Background Tasks
- [ ] Task: Refactor notification handling to support the new stdio push mechanism.
    - [ ] Implement `flush_notifications_async` to write queued events to `stdout`.
    - [ ] Ensure `aria2` event listeners correctly queue notifications in `McpState`.
- [ ] Task: Integrate the new server into `main.rs` as the default stdio transport.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Notifications & Background Tasks' (Protocol in workflow.md)

## Phase 4: Validation & Cleanup
- [ ] Task: Write unit tests for JSON-RPC parsing and routing.
- [ ] Task: Write integration tests for the stdio transport using `tokio-test` or similar.
- [ ] Task: Final project-wide checks with `/test-fix`.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Validation & Cleanup' (Protocol in workflow.md)
