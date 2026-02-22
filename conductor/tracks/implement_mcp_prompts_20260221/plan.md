# Implementation Plan - Implement MCP Prompts

## Phase 1: Prompt Registry & Core Logic
- [ ] Task: Define `Prompt` struct and `PromptRegistry` in `src/prompts/mod.rs`.
- [ ] Task: Update `McpServer` to initialize and use `PromptRegistry`.
- [ ] Task: Implement `prompts/list` handler in `McpHandler`.
- [ ] Task: Implement `prompts/get` handler in `McpHandler`.

## Phase 2: Implement Specific Prompts
- [ ] Task: Implement `DiagnoseDownloadPrompt` logic.
- [ ] Task: Implement `OptimizeSchedulePrompt` logic.

## Phase 3: Integration & Testing
- [ ] Task: Add unit tests for prompt registry and handlers.
- [ ] Task: Add integration tests verifying `prompts/list` and `prompts/get` via MCP protocol.
- [ ] Task: Update documentation to list available prompts.