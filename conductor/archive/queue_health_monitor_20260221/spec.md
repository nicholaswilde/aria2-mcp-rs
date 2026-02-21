# Specification - Queue Health Monitor

## Overview
Implement a `check_health` tool that provides actionable insights into the download queue by identifying stalled downloads, disk space issues, and other potential problems.

## User Stories
- As a user, I want to know if any downloads are stalled so I can try changing trackers or restarting them.
- As a user, I want a warning if I'm about to run out of disk space.

## Functional Requirements
- Identify downloads with 0 peers or stalled progress.
- Report remaining disk space vs. total size of active downloads.
- Suggest removals for completed downloads that are taking up space.
- Provide a summary report with identified issues and recommendations.

## Technical Requirements
- Utilize `aria2.getGlobalStat` and `aria2.tellActive`/`waiting`/`stopped`.
- Implement logic to detect "stalled" downloads (e.g., progress hasn't changed in N minutes).
- Add support for checking disk space on the local machine where the server is running.
