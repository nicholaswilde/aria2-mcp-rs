# Track: Multi-Instance Management for aria2-mcp-rs

## Overview
This track involves implementing multi-instance management for the `aria2-mcp-rs` server, following the primary-replica architecture pattern seen in other MCP implementations (e.g., AdGuard Home). This will allow a single MCP server to connect to and manage multiple `aria2` instances simultaneously.

## Functional Requirements
- **Configuration**:
    - Support for multiple `aria2` instances defined in `config.toml` as a list of objects (name, URL, RPC secret).
    - Support for environment variables using the `ARIA2_INSTANCES__<N>__<FIELD>` format (e.g., `ARIA2_INSTANCES__0__RPC_URL`).
- **Tool Integration**:
    - Update all existing tools (e.g., `manage_downloads`, `monitor_queue`, `search_downloads`) to accept an optional `instance` argument.
    - The `instance` argument will target a specific `aria2` instance by its index (0, 1, etc.) in the configuration list.
    - If the `instance` argument is omitted, the tool will target the first instance in the list by default.
- **Global Commands**:
    - Add a new tool `manage_all_instances` to perform operations (pause, resume, stop) on all configured instances at once.
- **Instance Health Monitoring**:
    - Update `check_health` to report the status of all configured `aria2` instances.
- **Client Management**:
    - The `McpServer` must maintain separate `Aria2Client` instances for each configured `aria2` server.
- **Registry Update**:
    - The `ToolRegistry` and tool schemas must be updated to include the `instance` parameter.

## Non-Functional Requirements
- **Backward Compatibility**: The server should still work if only a single instance is configured (as it does now).
- **Security**: RPC secrets for all instances must be handled securely and not logged.
- **Robustness**: If an invalid `instance` index is provided, tools should return a clear and user-friendly error message.

## Acceptance Criteria
- [ ] Running with multiple instances in `config.toml` or via environment variables initializes multiple `Aria2Client` instances.
- [ ] Tools can be targeted to specific instances using the `instance` argument.
- [ ] Tools default to the first instance if no `instance` argument is provided.
- [ ] `manage_all_instances` correctly affects all configured instances.
- [ ] `check_health` correctly reports the status of all configured instances.
- [ ] Tool schemas (in MCP) reflect the new optional `instance` parameter.
- [ ] Errors are correctly reported for non-existent instance indices.

## Out of Scope
- Dynamic addition/removal of instances without restarting the server.
- Load balancing across instances.
- Per-instance tool registration.
