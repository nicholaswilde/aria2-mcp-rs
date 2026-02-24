# Track: Implement Smart Error Recovery

## Overview
This track aims to make the `aria2-mcp-rs` server more resilient by implementing automated error recovery strategies. Instead of just reporting errors, the server will attempt to fix common issues automatically, such as retrying transient network failures or finding alternative trackers for dead torrents.

## Functional Requirements
- **Error Analysis**: Implement logic to analyze aria2 error codes (e.g., timeout vs. file not found vs. permission denied).
- **Auto-Retry**: For transient errors, implement an automatic retry mechanism with exponential backoff.
- **Tracker Injection (Optional/Advanced)**: If a torrent is stalled with 0 peers, attempt to fetch additional trackers from a known list and inject them.
- **Configurable**: Users should be able to enable/disable these features.

## Non-Functional Requirements
- **Transparency**: Log all recovery attempts so the user knows what happened.
- **Performance**: Recovery logic should run in the background and not block main operations.

## Acceptance Criteria
- [ ] Correctly identifies retryable errors.
- [ ] Retries a failed download with configured backoff.
- [ ] (Optional) Injects trackers for stalled torrents.
- [ ] Updates download status or logs to reflect recovery attempts.
- [ ] Integration tests verify retry behavior.