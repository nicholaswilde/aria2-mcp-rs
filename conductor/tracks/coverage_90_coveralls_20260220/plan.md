# Implementation Plan: Increase Code Coverage to >90% with Coveralls

## Phase 1: Tooling & Setup [checkpoint: 6a89a1a]
- [x] Task: Verify `Taskfile.yml` coverage commands d53fa74
    - [x] Run `task coverage` and verify summary output
    - [x] Run `task coverage:report` and verify `lcov.info` and `html/` generation
- [x] Task: Configure Coveralls Secret d53fa74
    - [x] Ensure `COVERALLS_REPO_TOKEN` is present in `.env`
    - [x] Run `task encrypt` to update `.env.enc`
- [x] Task: Conductor - User Manual Verification 'Phase 1: Tooling & Setup' (Protocol in workflow.md) 6a89a1a

## Phase 2: Server Logic Coverage [checkpoint: ae9b944]
- [x] Task: Expand tests for `src/server/handler.rs` 789d159
    - [x] Write tests for MCP request handling (list_tools, tools/call)
    - [x] Verify coverage increase
- [x] Task: Expand tests for `src/server/sse.rs` and `src/server/stdio.rs` 789d159
    - [x] Write integration/unit tests for SSE routes and Stdio transport
    - [x] Verify coverage increase
- [x] Task: Conductor - User Manual Verification 'Phase 2: Server Logic Coverage' (Protocol in workflow.md) ae9b944

## Phase 3: Tools & Registry Coverage [checkpoint: db766aa]
- [x] Task: Expand tests for `src/tools/registry.rs` 789d159
    - [x] Write tests for tool registration and retrieval
    - [x] Verify coverage increase
- [x] Task: Expand tests for `src/tools/manage_downloads.rs` and `src/tools/monitor_queue.rs` 789d159
    - [x] Write unit tests for tool execution logic and argument parsing
    - [x] Verify coverage increase
- [x] Task: Conductor - User Manual Verification 'Phase 3: Tools & Registry Coverage' (Protocol in workflow.md) db766aa

## Phase 4: Main & Config Coverage [checkpoint: bb1d32d]
- [x] Task: Expand tests for `src/main.rs` and `src/config.rs` 789d159
    - [x] Write tests for CLI argument parsing and configuration loading
    - [x] Verify coverage increase
- [x] Task: Conductor - User Manual Verification 'Phase 4: Main & Config Coverage' (Protocol in workflow.md) bb1d32d

## Phase 5: Final Verification & Upload
- [~] Task: Perform final coverage check
    - [ ] Run `task coverage` and confirm TOTAL > 90%
- [ ] Task: Upload to Coveralls
    - [ ] Run `task coverage:upload` and verify successful report submission
- [ ] Task: Conductor - User Manual Verification 'Phase 5: Final Verification & Upload' (Protocol in workflow.md)
