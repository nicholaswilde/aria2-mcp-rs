# :file_folder: aria2 MCP Server (Rust) :robot:

[![Coveralls](https://img.shields.io/coveralls/github/nicholaswilde/aria2-mcp-rs/main?style=for-the-badge&logo=coveralls)](https://coveralls.io/github/nicholaswilde/aria2-mcp-rs?branch=main)
[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/aria2-mcp-rs/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/aria2-mcp-rs/actions/workflows/ci.yml)

> [!WARNING]
> This project is currently in active development (v0.1.24) and is **not production-ready**. Features may change, and breaking changes may occur without notice. **Use this MCP server at your own risk.**

This project is a high-performance Model Context Protocol (MCP) server for [aria2](https://aria2.github.io/), built with Rust. It provides a comprehensive interface for LLMs to monitor and manage downloads through **interactive tools**, **read-only resources**, **contextual prompts**, and **real-time notifications**.

## :zap: Quick Start

1. **Prerequisites**: Ensure you have [aria2](https://aria2.github.io/) installed and running with RPC enabled (`aria2c --enable-rpc`).
2. **Configuration**: Copy the example config and edit it with your aria2 details:
   ```bash
   cp config.toml.example config.toml
   ```
3. **Run**: Use `cargo` to start the server:
   ```bash
   cargo run --release
   ```
   *By default, the server runs in **Stdio** mode, perfect for local integration.*

## Core Architectural Features

- **Dual Transport Implementation:** Supports both **Stdio** for local integration (e.g., Claude Desktop) and **HTTP/SSE** for remote, network-accessible clients.
- **Functional Tool Grouping:** Consolidates granular API actions into logical management tools (e.g., `manage_downloads`, `monitor_queue`) to minimize token usage and optimize the AI context window.
- **Multi-Instance Support:** Enables a single MCP server to monitor and manage multiple aria2 instances simultaneously.
- **Strict Filesystem Sandboxing:** Provides secure access to the download directory with path traversal prevention.
- **Automated Schema Validation:** Tools use a robust registry system with automated JSON schema-based input validation.

## Implemented Tools

The server provides several high-level tools for managing and monitoring aria2:

- **`manage_downloads`**: Add, pause, resume, and remove individual downloads.
- **`manage_all_instances`**: Perform bulk operations (pause, resume, purge) across all configured instances simultaneously.
- **`bulk_manage_downloads`**: Perform actions (pause, resume, remove) on multiple downloads simultaneously.
- **`monitor_queue`**: Get real-time status of active, waiting, and stopped downloads, plus global statistics.
- **`search_downloads`**: Find specific downloads by filename, URI, tracker URL, or status using substring or **regular expression** filters.
- **`check_health`**: Identify stalled downloads and potential queue issues (e.g., low disk space).
- **`manage_torrent`**: Manage BitTorrent-specific settings like fetching peers, selecting files, and adding/updating trackers.
- **`schedule_limits`**: Define bandwidth speed profiles and automatically activate them on a schedule.
- **`organize_completed`**: Automatically move completed downloads to target directories based on rules (extension or pattern).
- **`inspect_download`**: Get detailed technical metadata, file lists, or URIs for a specific download.
- **`list_download_files`**: List files and directories within a specified path relative to the download directory (strictly sandboxed).
- **`configure_aria2`**: Dynamically view and modify global or per-download aria2 settings.
- **`purge_policy`**: View or update the automated queue purging policy.
- **`add_rss_feed`**: Add a new RSS feed to monitor with optional keyword or regex filters.
- **`list_rss_feeds`**: List all currently monitored RSS feeds and their configurations.
- **`manage_tools`**: (Lazy Mode only) Enable or disable individual tools to optimize token usage.

## Implemented Resources

In addition to tools, the server exposes several read-only resources for direct context. All resources return data in a **multi-instance format**, providing status for all configured aria2 instances simultaneously:

- **`aria2://status/global`**: Real-time global statistics (speeds, counts) across all instances.
- **`aria2://downloads/active`**: List of currently active downloads with their GIDs and progress.
- **`aria2://logs/recent`**: The last N lines of the application log (useful for debugging connectivity issues).

## Implemented Prompts

Prompts provide structured interaction templates for common tasks:

- **`diagnose-download`**: Guides you through diagnosing issues with a specific download (accepts optional `gid`) or the entire queue.
- **`optimize-schedule`**: Helps you review and optimize your bandwidth schedules by providing an analysis of current settings.

## Real-Time Notifications

The server supports real-time notifications via aria2's WebSocket stream (currently supported in **Stdio** transport). The server proactively broadcasts events from **all configured instances** to connected MCP clients:

- **`download_start`**: Triggered when a download begins.
- **`download_pause`**: Triggered when a download is paused.
- **`download_stop`**: Triggered when a download is stopped.
- **`download_complete`**: Triggered when a download finished.
- **`download_error`**: Triggered when a download fails.
- **`bt_download_complete`**: Triggered when a BitTorrent download finished.

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
You can define multiple instances in `config.toml`, via environment variables, or directly via command-line arguments.

#### Command Line
Use the repeatable `-i/--instance` flag:
```bash
aria2-mcp-rs --instance name=local,url=http://localhost:6800/jsonrpc --instance name=remote,url=http://192.168.1.10:6800/jsonrpc,secret=token
```

#### Environment Variables
Use the indexed format supported by the configuration loader:
```bash
# Instance 0 (Primary)
export ARIA2_MCP__INSTANCES__0__NAME="primary"
export ARIA2_MCP__INSTANCES__0__RPC_URL="http://127.0.0.1:6800/jsonrpc"

# Instance 1 (Secondary)
export ARIA2_MCP__INSTANCES__1__NAME="remote-box"
export ARIA2_MCP__INSTANCES__1__RPC_URL="http://192.168.1.10:6800/jsonrpc"
export ARIA2_MCP__INSTANCES__1__RPC_SECRET="your-secret"
```

## 💾 Persistent Custom Rules

Changes to custom rules (like bandwidth schedules and file organization rules) made via the MCP tools are automatically persisted to a local state file (`aria2_mcp_state.json`) in the current working directory. 

When the server starts, it automatically loads these persistent rules and merges them with the rules defined in your `config.toml`, ensuring that any rules added by LLMs survive server restarts.

## :timer_clock: Bandwidth Scheduling

The server can automatically adjust aria2 bandwidth limits based on a schedule. You can define profiles and schedules in `config.toml`:

```toml
[bandwidth_profiles.work]
max_download = "500K"
max_upload = "50K"

[bandwidth_profiles.night]
max_download = "0" # Unlimited
max_upload = "0"

[[bandwidth_schedules]]
day = "daily"
start_time = "09:00"
end_time = "17:00"
profile_name = "work"

[[bandwidth_schedules]]
day = "daily"
start_time = "22:00"
end_time = "06:00"
profile_name = "night"
```

## :wastebasket: Automated Queue Purging

The server can automatically remove completed or errored downloads from the aria2 queue after they reach a certain age.

### Configuration
You can enable and configure the purging behavior in `config.toml`:

```toml
[purge_config]
enabled = true
interval_secs = 3600  # Run check every hour
min_age_secs = 86400  # Purge downloads older than 24 hours
excluded_gids = ["gid1", "gid2"] # Optional: GIDs to never purge
```

## :shield: Automated Error Recovery

The server includes built-in resiliency features to handle transient download failures:

- **Smart Retries**: Automatically retries downloads that fail due to transient errors (e.g., network timeouts, connection refused) using exponential backoff.
- **Tracker Injection**: For BitTorrent downloads that are stalled with 0 peers, the server can automatically fetch and inject additional trackers from a public list to help find peers.

You can configure these features in `config.toml`:

```toml
[retry_config]
max_retries = 3
initial_backoff_secs = 5
tracker_injection_enabled = true
tracker_list_url = "https://trackerslist.com/all.txt"
```

## :rss: RSS Feed Monitoring

The server can automatically monitor RSS feeds and add new items to the download queue based on filters. You can manage feeds via MCP tools or define them in `config.toml`:

```toml
[[rss_config.feeds]]
name = "My Linux ISOs"
url = "https://example.com/rss"
filters = ["ubuntu", "debian"] # Matches any keyword (case-insensitive)

[[rss_config.feeds]]
name = "Important Security Updates"
url = "https://example.com/security"
filters = ["regex:^CVE-2026-.*"] # Matches a regular expression
```

## :open_file_folder: Automated File Organization

The server can automatically move completed downloads to target directories based on user-defined rules. Rules can match files by extension or a regular expression pattern.

### Configuration
Define organization rules in `config.toml`:

```toml
[[organize_rules]]
name = "Movies"
extensions = ["mp4", "mkv", "avi"]
targetDir = "/mnt/media/movies"

[[organize_rules]]
name = "Linux ISOs"
pattern = "ubuntu-.*\\.iso"
targetDir = "/mnt/storage/isos"
```

*Note: The `targetDir` will be created automatically if it does not exist.*

## :movie_camera: Sequential Downloading for Media

The server supports sequential piece downloading for BitTorrent tasks. This allows you to start previewing or streaming media files while they are still downloading by ensuring that pieces are downloaded in order.

### Usage
- **New Downloads**: When adding a torrent via `manage_downloads`, set the `sequential` flag to `true`.
- **Existing Downloads**: Use the `manage_torrent` tool with the `toggleSequential` action and set `sequential` to `true` or `false`.

## :package: Installation

### Homebrew (macOS/Linux)

You can install the server using [Homebrew](https://brew.sh/) via the [nicholaswilde/tap](https://github.com/nicholaswilde/homebrew-tap):

```bash
brew install nicholaswilde/tap/aria2-mcp-rs
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

The server can be configured via command-line arguments, environment variables, or a `config.toml` file in the current directory.

### Command Line Interface

```bash
./target/release/aria2-mcp-rs --rpc-url "http://127.0.0.1:6800/jsonrpc" --rpc-secret "your-secret"
```

#### Configuration Options

| Argument | Environment Variable | Description | Default |
| :--- | :--- | :--- | :--- |
| `-u`, `--rpc-url` | `ARIA2_MCP_RPC_URL` | aria2 RPC URL | `http://127.0.0.1:6800/jsonrpc` |
| `-s`, `--rpc-secret` | `ARIA2_MCP_RPC_SECRET` | aria2 RPC Secret | - |
| `-t`, `--transport` | `ARIA2_MCP_TRANSPORT` | MCP Transport (stdio, sse, http) | `stdio` |
| `--http-host` | `ARIA2_MCP_HTTP_HOST` | HTTP Host for SSE | `0.0.0.0` |
| `--http-port` | `ARIA2_MCP_HTTP_PORT` | HTTP Port for SSE | `3000` |
| `--http-auth-token` | `ARIA2_MCP_HTTP_AUTH_TOKEN` | Bearer token for SSE security | (none) |
| `-i`, `--instance` | - | Add multiple instances (format: `name=N,url=U,secret=S`) | - |
| `-L`, `--log-level` | `ARIA2_MCP_LOG_LEVEL` | Application log level | `info` |
| `-l`, `--lazy` | `ARIA2_MCP_LAZY` | Enable Lazy Mode | `false` |
| `--no-verify-ssl` | `ARIA2_MCP_NO_VERIFY_SSL` | Disable SSL verification (default) | `true` |
| `--verify-ssl` | `ARIA2_MCP_VERIFY_SSL` | Enable SSL verification | `false` |

## :balance_scale: License

[MIT License](LICENSE)

## :writing_hand: Author

This project was started in 2026 by [Nicholas Wilde][2].

[2]: <https://github.com/nicholaswilde/>
