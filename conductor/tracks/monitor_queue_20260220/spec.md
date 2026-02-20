# Track Specification - monitor_queue_20260220

## Overview
Implement the `monitor_queue` MCP tool to provide high-level visibility into the aria2 download queue. This tool will allow agents to see active, waiting, and stopped downloads at a glance.

## Requirements
- **MCP Tool**: `monitor_queue`
- **Actions**:
  - `active`: List currently downloading tasks.
  - `waiting`: List tasks in the queue waiting to start.
  - `stopped`: List completed or failed tasks.
  - `stats`: Get global statistics (total up/down speed, task counts).
- **Consolidation**: Group multiple aria2 RPC calls into a single high-signal response.

## Success Criteria
- [ ] `monitor_queue` tool is registered in the MCP server.
- [ ] Tool correctly reports task counts and speeds.
- [ ] Integration tests verify accurate queue reporting.
