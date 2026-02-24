# Implementation Plan: Update Code Coverage to >90%

## Phase 1: Server Logic Coverage
- [x] Task: Test `src/server/stdio.rs`
    - [x] Refactor `run_server` to allow testing or use a mock transport
    - [x] Add tests for stdio server startup and notification loop
- [x] Task: Test `src/server/sse.rs`
    - [x] Add tests for `auth_middleware`
    - [x] Add tests for `run_server` using a mock listener or local port
    - [x] Add tests for lazy mode branches in `list_tools` and `execute_tool`
- [x] Task: Test `src/server/mod.rs`
    - [x] Add tests for server creation and port checking logic

## Phase 2: Tools Coverage
- [x] Task: Test `src/tools/rss.rs`
    - [x] Add unit tests for RSS feed processing logic and filters
- [x] Task: Test `src/tools/schedule_limits.rs`
    - [x] Add tests for profile management and schedule application

## Phase 3: Core & Main Coverage
- [x] Task: Test `src/aria2/mod.rs` and `src/aria2/notifications.rs`
    - [x] Add tests for error paths and notification conversion
- [x] Task: Test `src/main.rs`
    - [x] Add tests for complex argument combinations and error states

## Phase 4: Final Verification
- [x] Task: Run `task coverage` and verify TOTAL > 90%
