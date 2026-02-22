# Implementation Plan - Implement Sandboxed Filesystem Tools

## Phase 1: Sandbox Logic & Tool Definition
- [x] Task: Implement `PathSandbox` struct/module to handle safe path resolution. 81cddeb
    - Check canonical paths to prevent symlink bypass.
    - Ensure resolved path starts with the base directory.
- [x] Task: Define schema for `list_download_files` tool. febe196

## Phase 2: Tool Implementation
- [ ] Task: Implement `ListDownloadFilesTool` using `PathSandbox`.
- [ ] Task: Register the tool in `ToolRegistry`.

## Phase 3: Verification & Security Testing
- [ ] Task: Add unit tests for `PathSandbox` covering edge cases (symlinks, `..`, unicode).
- [ ] Task: Add integration tests simulating tool usage.
- [ ] Task: Verify security against common path traversal attacks.