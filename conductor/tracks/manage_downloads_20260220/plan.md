# Implementation Plan - manage_downloads_20260220

## Phase 1: Client Enhancements & Foundation

### [x] Task: Complete Aria2Client Management Methods (971f1a0)
- **Objective**: Ensure all necessary management methods are in the RPC client.
- [x] Task: Implement `remove` method in `Aria2Client`. (971f1a0)
- [x] Task: Implement `move_position` method in `Aria2Client`. (971f1a0)
- [x] Task: Implement `force_remove` and `force_pause` methods. (971f1a0)


### [x] Task: Define MCP Tool Schema (ceac512)
- **Objective**: Create the request/response schema for the `manage_downloads` tool.
- [x] Task: Define the tool input arguments structure (action, gid, uris, etc.). (ceac512)


- [ ] Task: Conductor - User Manual Verification 'Phase 1: Client Enhancements & Foundation' (Protocol in workflow.md)

## Phase 2: MCP Tool Implementation

### [ ] Task: Implement manage_downloads Tool
- **Objective**: Integrate the tool logic into the MCP server.
- [ ] Task: Implement the tool handler in `src/mcp.rs`.
- [ ] Task: Register the tool in the server.

### [ ] Task: Add Integration Tests for manage_downloads
- **Objective**: Verify the tool works with a real container.
- [ ] Task: Add integration tests in `tests/docker_integration_test.rs` specifically for the MCP tool interface.

- [ ] Task: Conductor - User Manual Verification 'Phase 2: MCP Tool Implementation' (Protocol in workflow.md)
