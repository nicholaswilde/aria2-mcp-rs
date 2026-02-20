# Implementation Plan - manage_downloads_20260220

## Phase 1: Client Enhancements & Foundation [checkpoint: ca608f8]

### [x] Task: Complete Aria2Client Management Methods (971f1a0)
- **Objective**: Ensure all necessary management methods are in the RPC client.
- [x] Task: Implement `remove` method in `Aria2Client`. (971f1a0)
- [x] Task: Implement `move_position` method in `Aria2Client`. (971f1a0)
- [x] Task: Implement `force_remove` and `force_pause` methods. (971f1a0)


### [x] Task: Define MCP Tool Schema (ceac512)
- **Objective**: Create the request/response schema for the `manage_downloads` tool.
- [x] Task: Define the tool input arguments structure (action, gid, uris, etc.). (ceac512)


- [x] Task: Conductor - User Manual Verification 'Phase 1: Client Enhancements & Foundation' (ca608f8)

## Phase 2: MCP Tool Implementation [checkpoint: ca608f8]

### [x] Task: Implement manage_downloads Tool (ca608f8)
- **Objective**: Integrate the tool logic into the MCP server.
- [x] Task: Implement the tool handler in `src/tools/manage_downloads.rs`. (ca608f8)
- [x] Task: Register the tool in the server in `src/main.rs`. (ca608f8)

### [x] Task: Add Integration Tests for manage_downloads (42cf3d9)
- **Objective**: Verify the tool works with a real container.
- [x] Task: Add integration tests in `tests/mcp_integration_test.rs` specifically for the MCP tool interface. (42cf3d9)

- [ ] Task: Conductor - User Manual Verification 'Phase 2: MCP Tool Implementation' (Protocol in workflow.md)
