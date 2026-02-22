# Track: Implement Configurable Log Level

## Overview
This track involves making the application's log level configurable across multiple sources (CLI, environment variables, and configuration files). This will allow users to adjust the verbosity of the server logs to suit their needs (e.g., debugging issues or reducing noise in production).

## Functional Requirements
- **Log Level Support**: The application must support the following log levels: `error`, `warn`, `info`, `debug`, `trace`.
- **Configuration Sources**:
    - **CLI**: Add a `--log-level` (or `-l`) argument.
    - **Environment Variable**: Support `ARIA2_MCP_LOG_LEVEL`.
    - **Configuration File**: Add a `log_level` field to `config.toml`.
- **Default Behavior**: If no log level is specified, it must default to `info`.
- **Invalid Input Handling**: If an invalid log level string is provided, the application should log a warning (using the default level) and proceed with the `info` level.

## Non-Functional Requirements
- **Performance**: Log level checks should not introduce significant overhead.
- **Robustness**: The application must not crash due to an invalid log level string.

## Acceptance Criteria
- [ ] Running with `--log-level debug` produces debug logs.
- [ ] Setting `ARIA2_MCP_LOG_LEVEL=trace` produces trace logs.
- [ ] Setting `log_level = "error"` in `config.toml` produces only error logs.
- [ ] Providing an invalid level (e.g., `invalid`) results in a warning and the `info` level being used.
- [ ] All configuration sources are correctly prioritized (CLI > Env > Config File > Default).

## Out of Scope
- Dynamic log level changes without restarting the application.
- Per-module log level configuration.
- Log rotation or persistent log storage.
