# Implementation Plan - Implement MCP Resources

## Phase 1: Resource Registry & Core Logic
- [ ] Task: Define `Resource` trait or struct in `src/resources/mod.rs`.
- [ ] Task: Implement `ResourceRegistry` to manage resources.
- [ ] Task: Update `McpServer` to initialize and use `ResourceRegistry`.
- [ ] Task: Implement `resources/list` handler in `McpHandler`.
- [ ] Task: Implement `resources/read` handler in `McpHandler`.

## Phase 2: Implement Specific Resources
- [ ] Task: Implement `GlobalStatusResource` (`aria2://status/global`).
- [ ] Task: Implement `ActiveDownloadsResource` (`aria2://downloads/active`).
- [ ] Task: Implement `RecentLogsResource` (`aria2://logs/recent`) with a ring buffer or file reader.

## Phase 3: Integration & Testing
- [ ] Task: Add unit tests for resource registry and handlers.
- [ ] Task: Add integration tests verifying `resources/list` and `resources/read` via MCP protocol.
- [ ] Task: Update documentation to list available resources.