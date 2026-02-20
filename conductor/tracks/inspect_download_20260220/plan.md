# Implementation Plan - inspect_download_20260220

## Phase 1: Client Inspection Methods

### [~] Task: Expand Client with Inspection Methods
- **Objective**: Ensure the client can retrieve task details.
- [~] Task: Implement `get_files` in `Aria2Client`.
- [x] Task: Implement `get_uris` in `Aria2Client`.

- [ ] Task: Conductor - User Manual Verification 'Phase 1: Client Inspection Methods' (Protocol in workflow.md)

## Phase 2: MCP Tool Implementation

### [ ] Task: Implement inspect_download Tool
- **Objective**: Create the detailed inspection tool.
- [ ] Task: Define tool schema and action logic.
- [ ] Task: Register tool in the MCP server.

### [ ] Task: Add Integration Tests for inspect_download
- **Objective**: Verify inspection depth.
- [ ] Task: Add tests to `tests/docker_integration_test.rs` to verify file and URI listing.

- [ ] Task: Conductor - User Manual Verification 'Phase 2: MCP Tool Implementation' (Protocol in workflow.md)
