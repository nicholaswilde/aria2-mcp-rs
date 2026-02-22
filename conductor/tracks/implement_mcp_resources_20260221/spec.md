# Track: Implement MCP Resources

## Overview
This track focuses on implementing the "Resources" capability of the Model Context Protocol (MCP). Resources allow the server to expose read-only data directly to the client (LLM) as context, which is faster and safer than executing tools for simple data retrieval.

## Functional Requirements
- **Resource Registry**: Implement a registry to manage and dispatch resource requests.
- **Resource Handlers**: Implement handlers for specific resources:
    - `aria2://status/global`: Returns global statistics (download speed, upload speed, active/waiting/stopped counts).
    - `aria2://logs/recent`: Returns the last N lines of application logs (if available/configured).
    - `aria2://downloads/active`: Returns a list of currently active downloads with key details (GID, name, progress).
- **McpServer Update**: Update `McpServer` to handle `resources/list` and `resources/read` requests.
- **Client Integration**: Ensure resources fetch data from the `Aria2Client`.

## Non-Functional Requirements
- **Performance**: Resource reads should be fast and non-blocking.
- **Security**: Logs should be sanitized if they contain sensitive information (like RPC secrets).
- **Usability**: Resource URIs should be intuitive and follow a consistent scheme.

## Acceptance Criteria
- [ ] `resources/list` returns the list of available resources.
- [ ] `resources/read` with `aria2://status/global` returns valid JSON stats.
- [ ] `resources/read` with `aria2://downloads/active` returns a valid JSON list.
- [ ] `resources/read` with `aria2://logs/recent` returns text logs.
- [ ] Integration tests verify resource availability and content.