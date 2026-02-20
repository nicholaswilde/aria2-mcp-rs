# Technology Stack - aria2-mcp-rs

## Core Language: Rust
- **Reasoning**: Provides memory safety, high performance, and zero-cost abstractions, making it ideal for a robust and efficient MCP server.
- **Version**: Latest stable.

## Framework: Model Context Protocol (MCP)
- **Reasoning**: The industry-standard protocol for connecting LLMs to external data sources and tools.
- **Library**: `mcp-sdk-rs` (Standard Rust SDK for MCP).

## Communication: JSON-RPC over HTTP
- **Reasoning**: `aria2` uses JSON-RPC for its remote interface.
- **Library**: `reqwest` or `jsonrpsee` for interacting with the `aria2` RPC endpoint.

## Package Management: Cargo
- **Reasoning**: The standard build system and package manager for Rust.

## Testing: Built-in Rust Test Framework & Testcontainers
- **Reasoning**: Provides robust unit and integration testing capabilities out of the box. `testcontainers` is used to spin up real `aria2` instances for comprehensive integration testing.
- **Library**: `testcontainers` (for Docker-based integration tests).
