# Implementation Plan - docker_integration_tests_20260220

## Phase 1: Infrastructure & Test Environment Setup

### [ ] Task: Configure Integration Test Dependencies
- **Objective**: Add necessary crates for containerized testing.
- [ ] Task: Add `testcontainers` and `testcontainers-modules` (if applicable) to `Cargo.toml` dev-dependencies.
- [ ] Task: Update `Cargo.toml` and verify compilation.

### [ ] Task: Implement Aria2 Container Wrapper
- **Objective**: Create a reusable abstraction for the `aria2` Docker container.
- [ ] Task: Create `tests/common/mod.rs` (if not exists) and define an `Aria2Container` struct.
- [ ] Task: Implement `testcontainers::Image` for `Aria2Container` using `aria2/aria2`.
- [ ] Task: Write a basic test to verify the container starts and is reachable.

- [ ] Task: Conductor - User Manual Verification 'Phase 1: Infrastructure & Test Environment Setup' (Protocol in workflow.md)

## Phase 2: Core Functional Integration Tests

### [ ] Task: Implement Basic Download Integration Test
- **Objective**: Verify the full path from MCP tool to `aria2` task creation.
- [ ] Task: Write a failing integration test in `tests/docker_integration_test.rs` for adding a download.
- [ ] Task: Implement/Refine the `Aria2Client` or MCP tool to pass the test.
- [ ] Task: Verify the test passes and the download starts in the container.

### [ ] Task: Implement Status Reporting Integration Test
- **Objective**: Ensure the server correctly retrieves and formats task status.
- [ ] Task: Write a failing integration test for querying download progress and speed.
- [ ] Task: Implement/Refine the status retrieval logic to pass the test.
- [ ] Task: Verify the test passes with live data from the container.

- [ ] Task: Conductor - User Manual Verification 'Phase 2: Core Functional Integration Tests' (Protocol in workflow.md)

## Phase 3: Advanced Control & Configuration Tests

### [ ] Task: Implement Pause/Resume Integration Test
- **Objective**: Verify state management of downloads via the RPC bridge.
- [ ] Task: Write failing integration tests for pausing and resuming active downloads.
- [ ] Task: Implement/Refine the pause/resume logic in the client/server to pass the tests.
- [ ] Task: Verify the tests pass against the live `aria2` instance.

### [ ] Task: Implement Configuration Update Integration Test
- **Objective**: Verify dynamic adjustment of `aria2` settings.
- [ ] Task: Write a failing integration test for updating a global configuration option.
- [ ] Task: Implement/Refine the configuration update logic to pass the test.
- [ ] Task: Verify the setting change is reflected in the containerized instance.

- [ ] Task: Conductor - User Manual Verification 'Phase 3: Advanced Control & Configuration Tests' (Protocol in workflow.md)
