# Specification: Update Code Coverage to >90%

## Overview
This track aims to bring the `aria2-mcp-rs` project's code coverage back to above 90%. Recent additions have caused the coverage to drop to ~81%. This track will specifically target the `server/` module and several `tools/` that are currently under-tested.

## Functional Requirements
- **Coverage Expansion:**
    - Increase test coverage for `src/server/stdio.rs` (currently 0%).
    - Increase test coverage for `src/server/sse.rs` (currently 41.30%).
    - Increase test coverage for `src/server/mod.rs` (currently 47.48%).
    - Increase test coverage for `src/tools/rss.rs` (currently 68.03%).
    - Increase test coverage for `src/tools/schedule_limits.rs` (currently 71.43%).
    - Address gaps in `src/main.rs`, `src/aria2/mod.rs`, and `src/aria2/notifications.rs`.
- **Maintainability:**
    - Tests should be reliable and not flaky.
    - Use existing testing infrastructure (mocks, common test helpers).

## Non-Functional Requirements
- **Coverage Target:** >90% total line coverage.

## Acceptance Criteria
- [ ] Total project line coverage is at or above 90% as reported by `task coverage`.
- [ ] `src/server/stdio.rs` coverage is > 80%.
- [ ] `src/server/sse.rs` coverage is > 80%.
- [ ] `src/server/mod.rs` coverage is > 80%.
- [ ] All new tests pass on the host machine.
