# Implementation Plan - Add http_port and http_auth_token for HTTP mode

This plan outlines the steps to refine the `http_port` configuration and implement Bearer token authentication for the SSE transport mode, following a Test-Driven Development (TDD) approach.

## Phase 1: Bearer Token Authentication [checkpoint: c269dc0]
- [x] Task: Write failing tests for Bearer Token validation in `sse.rs` and `Config` loading. [2f37543]
- [x] Task: Implement `http_auth_token` in `Config` and `Args`, and add Axum middleware for validation. [2f37543]
- [x] Task: Conductor - User Manual Verification 'Bearer Token Authentication' (Protocol in workflow.md)

## Phase 2: Port Availability Check [checkpoint: 3fc3ea7]
- [x] Task: Write failing tests for the port availability check utility. [a94062f]
- [x] Task: Implement the port availability check and integrate it into the server startup logic. [a94062f]
- [x] Task: Conductor - User Manual Verification 'Port Availability Check' (Protocol in workflow.md)

## Phase 3: Documentation & Final Integration
- [ ] Task: Update `config.toml.example` and `README.md` to document the new parameters.
- [ ] Task: Perform final project-wide verification and quality gate checks.
- [ ] Task: Conductor - User Manual Verification 'Documentation & Final Integration' (Protocol in workflow.md)
