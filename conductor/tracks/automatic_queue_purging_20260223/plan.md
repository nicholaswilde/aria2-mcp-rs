# Implementation Plan - Automatic Queue Purging

## Phase 1: Policy Configuration
- [ ] Task: Define a `PurgePolicy` struct and add it to the application configuration.
- [ ] Task: Implement a tool to view and update the purge policy.

## Phase 2: Background Purge Job
- [ ] Task: Implement a background loop in the MCP server to periodically check the download queue status.
- [ ] Task: Logic to identify stopped or errored downloads older than the purge limit.
- [ ] Task: Integration with `aria2.removeDownloadResult` for purging.

## Phase 3: Testing & Verification
- [ ] Task: Add unit tests for purge logic.
- [ ] Task: Add integration tests with mocked stopped downloads.
