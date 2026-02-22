# Track: Add http_port and http_auth_token for HTTP mode

## Overview
This track involves refining the `http_port` parameter and introducing a new `http_auth_token` parameter for the HTTP (SSE) transport mode. This will provide standardized port configuration and basic security for network-accessible MCP server instances.

## Functional Requirements
- **http_port Parameter**:
    - Must be configurable via command-line arguments, environment variables (`ARIA2_MCP_HTTP_PORT`), and the configuration file (`config.toml`).
    - Default value: `3000`.
    - Explicit check for port availability before startup to improve error reporting.
- **http_auth_token Parameter**:
    - Must be configurable via command-line arguments, environment variables (`ARIA2_MCP_HTTP_AUTH_TOKEN`), and the configuration file (`config.toml`).
    - When set, clients MUST provide the token in the `Authorization` header as `Bearer <token>`.
    - The server (Axum) must validate this token for all incoming requests.

## Non-Functional Requirements
- **Security**: The token must be handled securely and not logged.
- **Robustness**: If a token is set and the header is missing/incorrect, the server must return a `401 Unauthorized` error.

## Acceptance Criteria
- [ ] Running with `--transport sse` uses port 3000 by default.
- [ ] Running with `--http-auth-token your-secure-token` enables authentication.
- [ ] Requests to the SSE server without the correct Bearer token are rejected with `401 Unauthorized`.
- [ ] Requests with the correct token are accepted and processed.
- [ ] Starting the server on a busy port yields a descriptive error message instead of a generic stack trace.

## Out of Scope
- OAuth2/OpenID Connect integration.
- Dynamic token rotation.
