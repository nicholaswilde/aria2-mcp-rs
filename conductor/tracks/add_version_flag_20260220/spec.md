# Specification: Add Version Command Line Argument

## Overview
Add a standard command-line argument to display the application version. This enables users to easily identify which version of the `aria2-mcp-rs` server they are running.

## Functional Requirements
- **Version Flag**: The application MUST support both `--version` and `-V` flags.
- **Output Format**: The output MUST be formatted as `aria2-mcp-rs <version>`, where `<version>` is the current version of the application.
- **Version Source**: The version information MUST be retrieved automatically from the `Cargo.toml` file.
- **Exit Behavior**: After displaying the version information, the application MUST exit immediately with a success code (0).

## Technical Requirements
- Use `clap`'s built-in version support (e.g., `#[command(version)]` or similar macro-based configuration).
- Ensure the version is correctly propagated from the crate metadata.

## Acceptance Criteria
- Running `aria2-mcp-rs --version` outputs `aria2-mcp-rs 0.1.0` (or current version) and exits.
- Running `aria2-mcp-rs -V` outputs `aria2-mcp-rs 0.1.0` (or current version) and exits.
- No server is started when the version flag is used.

## Out of Scope
- Customizing the version output beyond the standard format.
- Storing version information in an external file other than `Cargo.toml`.
