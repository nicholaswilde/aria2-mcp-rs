# Track: Sequential Downloading for Media

## Overview
Implement an option for sequential downloading of files in BitTorrent tasks, allowing users to start previewing media files while they are still downloading.

## Functional Requirements
- **BitTorrent Option**: Add support for the `bt-sequential` option in `aria2.addUri` and `aria2.changeOption`.
- **Toggle Mechanism**: A way for users to enable/disable sequential download via MCP for active tasks.

## Acceptance Criteria
- [ ] Users can specify sequential downloading when adding a torrent.
- [ ] Sequential download can be toggled on/off for existing downloads.
- [ ] Integration tests verify the option is correctly sent to aria2.
