# Track Specification - manage_downloads_20260220

## Overview
Implement the `manage_downloads` MCP tool to provide core lifecycle control for aria2 download tasks. This tool will consolidate methods for adding, pausing, resuming, and removing downloads.

## Requirements
- **MCP Tool**: `manage_downloads`
- **Actions**:
  - `add`: Add one or more URIs.
  - `pause`: Pause an active download.
  - `resume`: Resume a paused download.
  - `remove`: Remove a download (stopped or active).
  - `move`: Change the position of a download in the queue.
- **Validation**: Ensure GIDs and URIs are validated before calling the RPC.
- **Error Handling**: Map aria2 RPC errors to meaningful MCP tool errors.

## Success Criteria
- [ ] `manage_downloads` tool is registered in the MCP server.
- [ ] All actions (add, pause, resume, remove, move) work as expected against a live aria2 instance.
- [ ] Integration tests verify the tool functionality.
