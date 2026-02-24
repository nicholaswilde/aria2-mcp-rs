# Track: Auto-Shutdown/Sleep Integration

## Overview
Implement a tool to trigger a system shutdown, sleep, or hibernate once all active downloads in the aria2 queue are complete.

## Functional Requirements
- **Action Selection**: Support for shutdown, sleep, and hibernate actions.
- **Monitoring Logic**: Periodically check the queue for active downloads.
- **System Command Execution**: Safe execution of platform-appropriate system commands.

## Acceptance Criteria
- [ ] Users can trigger a system action (shutdown, sleep) once downloads are finished.
- [ ] The system correctly identifies when all active downloads are complete.
- [ ] System commands are executed securely and correctly on supported platforms.
