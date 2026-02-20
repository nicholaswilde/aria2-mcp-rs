# Track Specification - inspect_download_20260220

## Overview
Implement the `inspect_download` MCP tool to provide detailed inspection of specific download tasks. This is crucial for multi-file downloads (torrents) and detailed troubleshooting.

## Requirements
- **MCP Tool**: `inspect_download`
- **Actions**:
  - `status`: Get detailed state, progress, and error info for a GID.
  - `files`: List all files in a task (useful for torrents).
  - `uris`: List URIs associated with a task.
- **Context**: Provide enough detail for an agent to decide which files to prioritize or why a download failed.

## Success Criteria
- [ ] `inspect_download` tool is registered in the MCP server.
- [ ] Tool provides detailed file and URI information.
- [ ] Integration tests verify detailed task inspection.
