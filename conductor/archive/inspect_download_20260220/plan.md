# Implementation Plan - inspect_download_20260220

## Phase 1: Client Inspection Methods

### [~] Task: Expand Client with Inspection Methods
- **Objective**: Ensure the client can retrieve task details.
- [~] Task: Implement `get_files` in `Aria2Client`.
- [x] Task: Implement `get_uris` in `Aria2Client`.

- [x] Task: Conductor - User Manual Verification 'Phase 1: Client Inspection Methods' (Protocol in workflow.md)

## Phase 2: MCP Tool Implementation

### [x] Task: Implement inspect_download Tool
- **Objective**: Create the detailed inspection tool.
- [x] Task: Define tool schema and action logic.
- [x] Task: Register tool in the MCP server.

### [x] Task: Add Integration Tests for inspect_download
- **Objective**: Verify inspection depth.
- [x] Task: Add tests to `tests/docker_integration_test.rs` to verify file and URI listing.

- [x] Task: Conductor - User Manual Verification 'Phase 2: MCP Tool Implementation' (Protocol in workflow.md)
