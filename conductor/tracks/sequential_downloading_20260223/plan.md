# Implementation Plan - Sequential Downloading for Media

## Phase 1: API Integration
- [ ] Task: Update `manage_downloads` to support the `sequential` flag for BitTorrent.
- [ ] Task: Update `manage_torrent` to allow toggling sequential download on active tasks.

## Phase 2: User Experience Enhancements
- [ ] Task: Document media streaming/previewing capabilities enabled by sequential download.

## Phase 3: Testing & Verification
- [ ] Task: Add unit tests for `bt-sequential` option handling.
- [ ] Task: Verify the option in an integration test with a real aria2 instance.
