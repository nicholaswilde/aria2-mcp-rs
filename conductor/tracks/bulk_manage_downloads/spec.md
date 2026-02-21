# Specification - Bulk Operations

## Overview
Implement a `bulk_manage_downloads` tool that performs actions (pause, resume, remove) on multiple GIDs provided in a list. This will reduce the number of tool calls needed for managing large queues.

## User Stories
- As a user, I want to pause all my downloads at once so I can free up bandwidth for a video call.
- As a power user, I want to remove multiple completed downloads in one go.

## Functional Requirements
- Support actions like `pause`, `resume`, `remove`, `forcePause`, and `forceRemove`.
- Accept a list of GIDs to apply the action to.
- Return a summary of the results (e.g., "10 downloads paused, 2 failed").

## Technical Requirements
- Utilize existing `aria2` methods (`aria2.pause`, `aria2.unpause`, `aria2.remove`) in parallel or sequentially.
- Correctly handle errors for individual GIDs without aborting the entire operation.
