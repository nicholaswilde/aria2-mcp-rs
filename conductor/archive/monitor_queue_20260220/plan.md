# Implementation Plan - monitor_queue_20260220

## Phase 1: Client Monitoring Capabilities [checkpoint: f75f3bb]

### [x] Task: Implement tell* Methods in Aria2Client (45fea20)
- **Objective**: Expand the client to support queue listing.
- [x] Task: Implement `tell_active` in `Aria2Client`. (45fea20)
- [x] Task: Implement `tell_waiting` in `Aria2Client`. (45fea20)
- [x] Task: Implement `tell_stopped` in `Aria2Client`. (45fea20)
- [x] Task: Implement `get_global_stat` in `Aria2Client`. (45fea20)

- [x] Task: Conductor - User Manual Verification 'Phase 1: Client Monitoring Capabilities' (f75f3bb)

## Phase 2: MCP Tool Implementation [checkpoint: a3c5d86]

### [x] Task: Implement monitor_queue Tool (b83fc21)
- **Objective**: Create the consolidated monitoring tool.
- [x] Task: Define tool schema and action logic. (b83fc21)
- [x] Task: Register tool in the MCP server. (b83fc21)

### [x] Task: Add Integration Tests for monitor_queue (b83fc21)
- **Objective**: Verify reporting accuracy.
- [x] Task: Add tests to `tests/monitor_queue_tool_tests.rs` to verify queue lists and global stats. (b83fc21)

- [x] Task: Conductor - User Manual Verification 'Phase 2: MCP Tool Implementation' (a3c5d86)
