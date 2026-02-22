# Implementation Plan - Implement MCP Resources

## Phase 1: Resource Registry & Core Logic
- [x] Task: Define `Resource` trait or struct in `src/resources/mod.rs`. bb487c3
- [x] Task: Implement `ResourceRegistry` to manage resources. c466ecd
- [x] Task: Update `McpServer` to initialize and use `ResourceRegistry`. af88fd0
- [x] Task: Implement `resources/list` handler in `McpHandler`. cae3a2e
- [x] Task: Implement `resources/read` handler in `McpHandler`. 8dec05f

## Phase 2: Implement Specific Resources
- [x] Task: Implement `GlobalStatusResource` (`aria2://status/global`). 8dec05f
- [~] Task: Implement `ActiveDownloadsResource` (`aria2://downloads/active`).
- [ ] Task: Implement `RecentLogsResource` (`aria2://logs/recent`) with a ring buffer or file reader.

## Phase 3: Integration & Testing
- [ ] Task: Add unit tests for resource registry and handlers.
- [ ] Task: Add integration tests verifying `resources/list` and `resources/read` via MCP protocol.
- [ ] Task: Update documentation to list available resources.