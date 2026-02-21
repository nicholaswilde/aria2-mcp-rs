# Implementation Plan - Implement Lazy Mode

## Tasks [checkpoint: ]
- [ ] Add `lazy_mode` to `Config` in `src/config.rs`.
- [ ] Implement tool-enabling logic in `ToolRegistry` in `src/tools/registry.rs`.
- [ ] Update `McpHandler` in `src/server/handler.rs` to support filtering and `manage_tools`.
- [ ] Add unit tests for tool-enabling logic.
- [ ] Add integration tests for `manage_tools`.
- [ ] Verify token optimization with a script to check `tools/list` response size.
