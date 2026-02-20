# Implementation Plan: Increase Code Coverage to >90% with Coveralls

## Phase 1: Tooling & Setup
- [ ] Task: Verify `Taskfile.yml` coverage commands
    - [ ] Run `task coverage` and verify summary output
    - [ ] Run `task coverage:report` and verify `lcov.info` and `html/` generation
- [ ] Task: Configure Coveralls Secret
    - [ ] Ensure `COVERALLS_REPO_TOKEN` is present in `.env`
    - [ ] Run `task encrypt` to update `.env.enc`
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Tooling & Setup' (Protocol in workflow.md)

## Phase 2: Server Logic Coverage
- [ ] Task: Expand tests for `src/server/handler.rs`
    - [ ] Write tests for MCP request handling (list_tools, tools/call)
    - [ ] Verify coverage increase
- [ ] Task: Expand tests for `src/server/sse.rs` and `src/server/stdio.rs`
    - [ ] Write integration/unit tests for SSE routes and Stdio transport
    - [ ] Verify coverage increase
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Server Logic Coverage' (Protocol in workflow.md)

## Phase 3: Tools & Registry Coverage
- [ ] Task: Expand tests for `src/tools/registry.rs`
    - [ ] Write tests for tool registration and retrieval
    - [ ] Verify coverage increase
- [ ] Task: Expand tests for `src/tools/manage_downloads.rs` and `src/tools/monitor_queue.rs`
    - [ ] Write unit tests for tool execution logic and argument parsing
    - [ ] Verify coverage increase
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Tools & Registry Coverage' (Protocol in workflow.md)

## Phase 4: Main & Config Coverage
- [ ] Task: Expand tests for `src/main.rs` and `src/config.rs`
    - [ ] Write tests for CLI argument parsing and configuration loading
    - [ ] Verify coverage increase
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Main & Config Coverage' (Protocol in workflow.md)

## Phase 5: Final Verification & Upload
- [ ] Task: Perform final coverage check
    - [ ] Run `task coverage` and confirm TOTAL > 90%
- [ ] Task: Upload to Coveralls
    - [ ] Run `task coverage:upload` and verify successful report submission
- [ ] Task: Conductor - User Manual Verification 'Phase 5: Final Verification & Upload' (Protocol in workflow.md)
