# Specification - Automatic File Organizer

## Overview
Implement an `organize_completed` tool that moves completed downloads to specific directories based on user-defined rules (e.g., file extension, filename pattern).

## User Stories
- As a user, I want my movies moved to my movie folder automatically when they finish downloading.
- As a user, I want my document downloads sorted into a specific directory.

## Functional Requirements
- Allow users to define simple rules (regex for filename, list of extensions) and target directories.
- Move files belonging to a completed download to the target directory.
- Update aria2 about the new file location (if possible) or just perform the filesystem move.

## Technical Requirements
- Utilize Rust's standard library `std::fs` for moving files.
- Handle potential errors like missing directories or permission issues.
- Support basic rule persistence (e.g., in a local configuration file).
