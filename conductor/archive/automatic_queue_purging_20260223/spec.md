# Track: Automatic Queue Purging

## Overview
Implement an automated mechanism to remove stopped or errored downloads from the aria2 queue after a certain period of time.

## Functional Requirements
- **Configurable Purge Policy**: Support for setting the time-to-purge (e.g., purge after 7 days).
- **Background Purge Task**: A periodic background job to identify and remove eligible downloads.
- **Manual Overrides**: Option for users to keep specific stopped downloads indefinitely.

## Acceptance Criteria
- [ ] Users can set a purge time limit.
- [ ] Stopped/errored downloads are automatically removed after the limit.
- [ ] A tool or option exists to exclude specific downloads from purging.
