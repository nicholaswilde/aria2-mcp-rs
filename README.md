# :file_folder: aria2 MCP Server (Rust) :robot:

[![Coveralls](https://img.shields.io/coveralls/github/nicholaswilde/aria2-mcp-rs/main?style=for-the-badge&logo=coveralls)](https://coveralls.io/github/nicholaswilde/aria2-mcp-rs?branch=main)
[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/aria2-mcp-rs/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/aria2-mcp-rs/actions/workflows/ci.yml)

> [!WARNING]
> This project is currently in active development (v0.1.0) and is **not production-ready**. Features may change, and breaking changes may occur without notice. **Use this MCP server at your own risk.**

This project is a high-performance Model Context Protocol (MCP) server for [aria2](https://aria2.github.io/), built with Rust. It provides a comprehensive interface for LLMs to monitor and manage downloads.

## Core Architectural Features

- **Dual Transport Implementation:** Supports both **Stdio** for local integration (e.g., Claude Desktop) and **HTTP/SSE** for remote, network-accessible clients.
- **Functional Tool Grouping:** Consolidates granular API actions into logical management tools (e.g., `manage_downloads`, `monitor_queue`) to minimize token usage and optimize the AI context window.
- **Multi-Instance Support:** Enables a single MCP server to monitor and manage multiple aria2 instances simultaneously.
- **Automated Schema Validation:** Tools use a robust registry system with automated JSON schema-based input validation.

## Technical Stack & Components

- **Tool Registry:** A modular system for registering and executing tools with strictly typed inputs.
- **Dynamic Configuration:** Uses the `config` crate to merge settings from CLI arguments, environment variables, and configuration files (TOML, YAML, JSON).
- **Async Runtime:** Built on `tokio`, utilizing background tasks for efficient I/O and instance management.
- **API & Serialization:** Leverages `axum` for web transport and `serde`/`serde_json` for MCP message handling.

## :hammer_and_wrench: Build & Development

### Task-Based Workflow
Utilizes `Taskfile.yml` to standardize development tasks:
```bash
# Build in release mode
task build

# Run tests and lints
task check
```

### Quality Assurance
- **Docker Integration Tests:** Validates real-world interaction with aria2 containers.
- **Unit Testing:** Comprehensive test suite for core logic.
- **Release Optimization:** Link Time Optimization (LTO) and symbol stripping for minimal binary size.

## :rocket: Usage

### Command Line Interface

```bash
./target/release/aria2-mcp-rs --rpc-url "http://localhost:6800/jsonrpc" --rpc-secret "your-secret"
```

#### Configuration Options

| Argument | Environment Variable | Description | Default |
| :--- | :--- | :--- | :--- |
| `--rpc-url` | `ARIA2_RPC_URL` | aria2 RPC URL | `http://localhost:6800/jsonrpc` |
| `--rpc-secret` | `ARIA2_RPC_SECRET` | aria2 RPC Secret | - |

## :balance_scale: License

[MIT License](LICENSE)

## :writing_hand: Author

This project was started in 2026 by [Nicholas Wilde][2].

[2]: <https://github.com/nicholaswilde/>
