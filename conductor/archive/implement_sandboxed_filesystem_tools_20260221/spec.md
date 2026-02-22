# Track: Implement Sandboxed Filesystem Tools

## Overview
This track introduces tools that allow LLMs to interact with the filesystem in a secure, sandboxed manner. The primary goal is to provide a way to inspect the contents of the download directory (e.g., verify download success, check extracted files) without exposing the entire system.

## Functional Requirements
- **`list_download_files` Tool**: Lists files and directories within a specified path relative to the download directory.
    - **Arguments**: `path` (relative), `max_depth` (optional).
    - **Validation**: Strict check to ensure `path` resolves within the configured `dir` of the aria2 instance. Prevent path traversal (`../`).

## Non-Functional Requirements
- **Security**: The most critical requirement. Any attempt to access files outside the sandbox MUST be rejected.
- **Privacy**: File content inspection (reading actual bytes) is out of scope for this initial implementation unless strictly necessary and safe.

## Acceptance Criteria
- [ ] `list_download_files` successfully lists files in valid subdirectories of the download folder.
- [ ] Attempts to access `..` or absolute paths outside the download directory return an error.
- [ ] Attempts to access symlinks pointing outside the sandbox return an error (or resolve safely within).
- [ ] Integration tests verify both success and security denial cases.