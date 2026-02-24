# Implementation Plan - Sequential Downloading for Media

## Phase 1: API Integration
- [x] Task: Update `manage_downloads` to support the `sequential` flag for BitTorrent. (3cb47a0)
- [x] Task: Update `manage_torrent` to allow toggling sequential download on active tasks. (3cb47a0)

## Phase 2: User Experience Enhancements
- [x] Task: Document media streaming/previewing capabilities enabled by sequential download. (e8f395f)

## Phase 3: Testing & Verification
- [x] Task: Add unit tests for `bt-sequential` option handling. (c5df9bf)
- [x] Task: Verify the option in an integration test with a real aria2 instance. (3cb47a0)
