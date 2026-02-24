# Track: Load Custom Configuration Files

## Overview
Add support for specifying a custom configuration file path via the CLI using `--config` or `-c`. This allows users to point the server to a specific configuration file instead of relying on the default search locations.

## Functional Requirements
- **CLI Arguments**: Add `--config` and `-c` arguments to the CLI using `clap`.
- **Exclusive Loading**: If a configuration file is specified via the CLI, the default configuration file search (e.g., looking for `config.toml` or `aria2-mcp` in the current directory) must be bypassed.
- **Supported Formats**: Support `.toml`, `.yaml`, `.yml`, and `.json` extensions.
- **Single File Support**: The CLI should only accept a single configuration file argument.
- **Error Handling**: If the specified file is missing or contains invalid syntax, the server should log a warning and attempt to start with default values (plus any environment variables or other CLI overrides), rather than exiting immediately.

## Non-Functional Requirements
- **Performance**: Configuration loading should remain fast and not introduce noticeable delay on startup.
- **Robustness**: Bypassing defaults must be reliable to prevent unintentional configuration merging.

## Acceptance Criteria
- [ ] Running with `-c custom.toml` loads settings from `custom.toml` and ignores `config.toml` if it exists.
- [ ] Running with `--config settings.json` correctly parses JSON configuration.
- [ ] Providing a non-existent file path logs a warning but allows the server to start (using defaults/env vars).
- [ ] The `--help` output shows the new `--config` and `-c` options.

## Out of Scope
- Support for multiple `-c` flags.
- Support for formats other than TOML, YAML, and JSON.
- Remote configuration loading (e.g., via HTTP).
