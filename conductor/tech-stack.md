# Technology Stack - aria2-mcp-rs

## Core Language: Rust
- **Reasoning**: Provides memory safety, high performance, and zero-cost abstractions, making it ideal for a robust and efficient MCP server.
- **Version**: Latest stable.

## Framework: Model Context Protocol (MCP)
- **Reasoning**: The industry-standard protocol for connecting LLMs to external data sources and tools.
- **Library**: `mcp-sdk-rs` (Standard Rust SDK for MCP).

## Web Framework & Transport: Axum
- **Reasoning**: Provides a robust, performant foundation for HTTP/SSE transport of MCP messages.
- **Library**: `axum`.

## Async Runtime: Tokio
- **Reasoning**: The most powerful and widely-adopted asynchronous runtime for Rust, essential for handling high-concurrency MCP requests and I/O tasks.
- **Library**: `tokio`.

## Communication: JSON-RPC over HTTP
- **Reasoning**: `aria2` uses JSON-RPC for its remote interface.
- **Library**: `reqwest` for interacting with the `aria2` RPC endpoint.

## Pattern Matching: Regex
- **Reasoning**: Required for robust filename pattern matching in the automatic file organization tool.
- **Library**: `regex`.

## Package Management: Cargo
- **Reasoning**: The standard build system and package manager for Rust.

## Testing: Built-in Rust Test Framework & Testcontainers
- **Reasoning**: Provides robust unit and integration testing capabilities out of the box. `testcontainers` is used to spin up real `aria2` instances for comprehensive integration testing.
- **Library**: `testcontainers` (for Docker-based integration tests).

## Coverage & Reporting: cargo-llvm-cov & Coveralls.io
- **Reasoning**: Ensures high code quality and robustness by tracking test coverage and providing a public dashboard for visibility.
- **Tools**: `cargo-llvm-cov` (for local coverage reports), `Coveralls.io` (for cloud-based tracking).

## Configuration Management: config & clap
- **Reasoning**: `config` allows for dynamic configuration merging from multiple sources (CLI, environment variables, files). `clap` provides a powerful, type-safe CLI argument parser.
- **Library**: `config`, `clap`.
