# Implementation Plan - Automatic Queue Purging

## Phase 1: Policy Configuration
- [x] Task: Define a `PurgePolicy` struct and add it to the application configuration. (31e8a98)
- [x] Task: Implement a tool to view and update the purge policy. (878269a)

## Phase 2: Background Purge Job
- [x] Task: Implement a background loop in the MCP server to periodically check the download queue status. (73ca092)
- [x] Task: Logic to identify stopped or errored downloads older than the purge limit. (73ca092)
- [x] Task: Integration with `aria2.removeDownloadResult` for purging. (73ca092)

## Phase 3: Testing & Verification
- [x] Task: Add unit tests for purge logic. (73ca092)
- [x] Task: Add integration tests with mocked stopped downloads. (07ab628)
