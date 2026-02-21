# Specification - Bandwidth Scheduler

## Overview
Implement a `schedule_limits` tool that allows defining bandwidth speed profiles that can be activated on a schedule (e.g., "Day Mode", "Night Mode").

## User Stories
- As a user, I want to limit download speeds during work hours and have them go to full speed at night.
- As a power user, I want a "Night Mode" profile to maximize downloads when I'm sleeping.

## Functional Requirements
- Support defining named speed profiles (e.g., "Work Mode", "Night Mode").
- Allow switching between speed profiles via MCP tool calls.
- Support activating speed profiles on a simple daily or weekly schedule.

## Technical Requirements
- Utilize `aria2.changeGlobalOption` to set `max-overall-download-limit` and `max-overall-upload-limit`.
- Implement a background task to check and activate scheduled profiles.
- Persist profile definitions in the server configuration.
