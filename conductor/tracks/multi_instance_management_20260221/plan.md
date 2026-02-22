# Implementation Plan - Multi-Instance Management for aria2-mcp-rs

This plan outlines the steps to implement multi-instance support in `aria2-mcp-rs`, enabling management of multiple `aria2` servers from a single MCP interface.

## Phase 1: Configuration & Client Management
- [ ] Task: Write failing tests for multi-instance configuration parsing (TOML & Env vars).
- [ ] Task: Update `Config` struct and loading logic to support `instances` list and `ARIA2_INSTANCES__<N>__<FIELD>` environment variables.
- [ ] Task: Write failing tests for multi-client initialization in `McpServer`.
- [ ] Task: Update `McpServer` to initialize and store a list of `Aria2Client` instances.
- [ ] Task: Conductor - User Manual Verification 'Configuration & Client Management' (Protocol in workflow.md)

## Phase 2: Tool Argument & Routing
- [ ] Task: Write failing tests for `instance` argument parsing and routing in `McpHandler`.
- [ ] Task: Update `ToolRegistry` to include optional `instance` (integer) parameter in all tool schemas.
- [ ] Task: Refactor `McpHandler` to route tool calls to the correct `Aria2Client` based on the `instance` argument (defaulting to index 0).
- [ ] Task: Conductor - User Manual Verification 'Tool Argument & Routing' (Protocol in workflow.md)

## Phase 3: Global Commands & Health Monitoring
- [ ] Task: Write failing tests for `manage_all_instances` tool.
- [ ] Task: Implement `manage_all_instances` tool to perform bulk operations across all clients.
- [ ] Task: Write failing tests for multi-instance health reporting in `check_health`.
- [ ] Task: Update `check_health` tool to iterate over all clients and report aggregate/individual status.
- [ ] Task: Conductor - User Manual Verification 'Global Commands & Health Monitoring' (Protocol in workflow.md)

## Phase 4: Final Documentation & Quality Gates
- [ ] Task: Update `config.toml.example` with multi-instance configuration examples.
- [ ] Task: Update `README.md` to document the new `instance` argument and multi-instance features.
- [ ] Task: Run project-wide verification (`/test-fix`) and ensure >80% coverage.
- [ ] Task: Conductor - User Manual Verification 'Final Documentation & Quality Gates' (Protocol in workflow.md)
