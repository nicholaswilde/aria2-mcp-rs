# :file_folder: aria2 MCP Server (Rust) :robot:

[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/aria2-mcp-rs/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/aria2-mcp-rs/actions/workflows/ci.yml)

> [!WARNING]
> This project is currently in active development and is **not production-ready**. Features may change, and breaking changes may occur without notice. **Use this MCP server at your own risk.**

A Rust implementation of an aria2 [MCP (Model Context Protocol) server](https://modelcontextprotocol.io/docs/getting-started/intro). This server connects to an `aria2` instance and exposes tools to monitor and manage downloads via the Model Context Protocol.

## :sparkles: Features

- **Download Control:** Support for adding, pausing, and resuming downloads via MCP.
- **Status Monitoring:** Ability to query current download status, progress, and speeds.
- **Configuration Management:** Tools to manage `aria2` global settings and configuration on the fly.
- **Robust Configuration:** Supports configuration via CLI arguments and environment variables.

## :hammer_and_wrench: Build

To build the project, you need a Rust toolchain installed.

### Local Build

```bash
# Build in release mode
task build
```

The binary will be available at `target/debug/aria2-mcp-rs`.

## :rocket: Usage

### :keyboard: Command Line Interface

The server can be configured via CLI arguments or environment variables.

```bash
./target/debug/aria2-mcp-rs --rpc-url "http://localhost:6800/jsonrpc" --rpc-secret "your-secret"
```

#### Available Arguments

| Argument | Environment Variable | Description | Default |
| :--- | :--- | :--- | :--- |
| `--rpc-url` | `ARIA2_RPC_URL` | aria2 RPC URL | `http://localhost:6800/jsonrpc` |
| `--rpc-secret` | `ARIA2_RPC_SECRET` | aria2 RPC Secret | - |

## :test_tube: Testing

The project uses [go-task](https://taskfile.dev/) for development tasks.

```bash
# Run all checks (format, lint, unit tests)
task check

# Run unit tests only
task test

# Run Docker integration tests (requires Docker)
cargo test --test docker_integration_test
```

## :balance_scale: License

​[MIT License](LICENSE)

## :writing_hand: Author

​This project was started in 2026 by [Nicholas Wilde][2].

[2]: <https://github.com/nicholaswilde/>
