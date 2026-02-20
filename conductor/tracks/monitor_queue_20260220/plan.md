# Implementation Plan - monitor_queue_20260220

## Phase 1: Client Monitoring Capabilities

### [ ] Task: Implement tell* Methods in Aria2Client
- **Objective**: Expand the client to support queue listing.
- [ ] Task: Implement `tell_active` in `Aria2Client`.
- [ ] Task: Implement `tell_waiting` in `Aria2Client`.
- [ ] Task: Implement `tell_stopped` in `Aria2Client`.
- [ ] Task: Implement `get_global_stat` in `Aria2Client`.

- [ ] Task: Conductor - User Manual Verification 'Phase 1: Client Monitoring Capabilities' (Protocol in workflow.md)

## Phase 2: MCP Tool Implementation

### [ ] Task: Implement monitor_queue Tool
- **Objective**: Create the consolidated monitoring tool.
- [ ] Task: Define tool schema and action logic.
- [ ] Task: Register tool in the MCP server.

### [ ] Task: Add Integration Tests for monitor_queue
- **Objective**: Verify reporting accuracy.
- [ ] Task: Add tests to `tests/docker_integration_test.rs` to verify queue lists and global stats.

- [ ] Task: Conductor - User Manual Verification 'Phase 2: MCP Tool Implementation' (Protocol in workflow.md)
