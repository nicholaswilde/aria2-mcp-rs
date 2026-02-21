# Specification - Implement Lazy Mode

## Overview
Implement a token-optimized "Lazy Mode" for `aria2-mcp-rs`, similar to its implementation in `adguardhome-mcp-rs`. This feature allows the server to initially present a minimal toolset, with additional tools being discoverable and enablable on-demand via a `manage_tools` meta-tool.

## User Stories
- As an AI agent user, I want to minimize the tokens used by tool definitions so that I have more room in the context window for my task.
- As a power user, I want the ability to choose which tools are visible to the AI, especially if I only use a subset of functionality.

## Functional Requirements
- Support a `lazy_mode` configuration setting (default: `false`).
- Initially only expose a subset of tools (e.g., `monitor_queue`) and a `manage_tools` meta-tool when `lazy_mode` is enabled.
- The `manage_tools` tool must support actions to `list`, `enable`, and `disable` other tools.
- Changes to enabled tools should take effect immediately in the `tools/list` response.
- Provide a notification (if supported by transport) when the tool list changes.

## Technical Requirements
- Update `Config` in `src/config.rs` to include `lazy_mode`.
- Modify `ToolRegistry` in `src/tools/registry.rs` to maintain a set of `enabled_tools`.
- Update `McpHandler` in `src/server/handler.rs` to:
    - Filter the result of `tools/list` based on the enabled status.
    - Inject the `manage_tools` tool definition when `lazy_mode` is active.
    - Handle `tools/call` for the `manage_tools` meta-tool.
- Implement unit tests for the registry's tool-enabling logic.
- Add integration tests for the `manage_tools` tool and its effect on the tool list.
