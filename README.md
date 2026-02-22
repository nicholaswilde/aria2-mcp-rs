# :file_folder: aria2 MCP Server (Rust) :robot:

[![Coveralls](https://img.shields.io/coveralls/github/nicholaswilde/aria2-mcp-rs/main?style=for-the-badge&logo=coveralls)](https://coveralls.io/github/nicholaswilde/aria2-mcp-rs?branch=main)
[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/aria2-mcp-rs/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/aria2-mcp-rs/actions/workflows/ci.yml)

> [!WARNING]
> This project is currently in active development (v0.1.5) and is **not production-ready**. Features may change, and breaking changes may occur without notice. **Use this MCP server at your own risk.**

This project is a high-performance Model Context Protocol (MCP) server for [aria2](https://aria2.github.io/), built with Rust. It provides a comprehensive interface for LLMs to monitor and manage downloads.

## Core Architectural Features

- **Dual Transport Implementation:** Supports both **Stdio** for local integration (e.g., Claude Desktop) and **HTTP/SSE** for remote, network-accessible clients.
- **Functional Tool Grouping:** Consolidates granular API actions into logical management tools (e.g., `manage_downloads`, `monitor_queue`) to minimize token usage and optimize the AI context window.
- **Multi-Instance Support:** Enables a single MCP server to monitor and manage multiple aria2 instances simultaneously.
- **Automated Schema Validation:** Tools use a robust registry system with automated JSON schema-based input validation.

## Implemented Tools

The server provides several high-level tools for managing and monitoring aria2:

- **`manage_downloads`**: Add, pause, resume, and remove individual downloads.
- **`manage_all_instances`**: Perform bulk operations (pause, resume, purge) across all configured instances simultaneously.
- **`bulk_manage_downloads`**: Perform actions (pause, resume, remove) on multiple downloads simultaneously.
- **`monitor_queue`**: Get real-time status of active, waiting, and stopped downloads, plus global statistics.
- **`search_downloads`**: Find specific downloads by filename, URI, or status (optimized for token efficiency).
- **`check_health`**: Identify stalled downloads and potential queue issues (e.g., low disk space).
- **`manage_torrent`**: Manage BitTorrent-specific settings like fetching peers, selecting files, and adding/updating trackers.
- **`schedule_limits`**: Define bandwidth speed profiles and automatically activate them on a schedule.
- **`organize_completed`**: Automatically move completed downloads to target directories based on rules (extension or pattern).
- **`inspect_download`**: Get detailed technical metadata and file lists for a specific download.
- **`configure_aria2`**: Dynamically view and modify global or per-download aria2 settings.
- **`manage_tools`**: (Lazy Mode only) Enable or disable individual tools to optimize token usage.

## Technical Stack & Components

- **Tool Registry:** A modular system for registering and executing tools with strictly typed inputs.
- **Dynamic Configuration:** Uses the `config` crate to merge settings from CLI arguments, environment variables, and configuration files (TOML, YAML, JSON).
- **Async Runtime:** Built on `tokio`, utilizing background tasks for efficient I/O and instance management.
- **API & Serialization:** Leverages `axum` for web transport and `serde`/`serde_json` for MCP message handling.

## Multi-Instance Management

The server can manage multiple aria2 instances simultaneously.

### Targeting Instances
All tools accept an optional `instance` argument (integer) to target a specific instance by its index in the configuration (defaulting to `0`).

### Configuration
You can define multiple instances in `config.toml` or via environment variables using the `ARIA2_MCP__INSTANCES__<N>__<FIELD>` format:

```bash
# Instance 0 (Primary)
export ARIA2_MCP__INSTANCES__0__NAME="primary"
export ARIA2_MCP__INSTANCES__0__RPC_URL="http://localhost:6800/jsonrpc"

# Instance 1 (Secondary)
export ARIA2_MCP__INSTANCES__1__NAME="remote-box"
export ARIA2_MCP__INSTANCES__1__RPC_URL="http://192.168.1.10:6800/jsonrpc"
export ARIA2_MCP__INSTANCES__1__RPC_SECRET="your-secret"
```

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
| `--rpc-url` | `ARIA2_MCP_RPC_URL` | aria2 RPC URL | `http://localhost:6800/jsonrpc` |
| `--rpc-secret` | `ARIA2_MCP_RPC_SECRET` | aria2 RPC Secret | - |
| `--transport` | `ARIA2_MCP_TRANSPORT` | MCP Transport (stdio, sse, http) | `stdio` |
| `--http-port` | `ARIA2_MCP_HTTP_PORT` | HTTP Port for SSE | `3000` |
| `--http-auth-token` | `ARIA2_MCP_HTTP_AUTH_TOKEN` | Bearer token for SSE security | (none) |
| `--log-level` | `ARIA2_MCP_LOG_LEVEL` | Application log level | `info` |
| `--lazy` | `ARIA2_MCP_LAZY` | Enable Lazy Mode | `false` |
| `--no-verify-ssl` | `ARIA2_MCP_NO_VERIFY_SSL` | Disable SSL verification | `true` |

## :balance_scale: License

[MIT License](LICENSE)

## :writing_hand: Author

This project was started in 2026 by [Nicholas Wilde][2].

[2]: <https://github.com/nicholaswilde/>
