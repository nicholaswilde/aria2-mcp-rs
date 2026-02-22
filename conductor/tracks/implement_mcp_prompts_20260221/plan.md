# Implementation Plan - Implement MCP Prompts

## Phase 1: Prompt Registry & Core Logic
- [x] Task: Define `Prompt` struct and `PromptRegistry` in `src/prompts/mod.rs`. bf74e8f
- [x] Task: Update `McpServer` to initialize and use `PromptRegistry`. 9e8d5ef
- [x] Task: Implement `prompts/list` handler in `McpHandler`. 7e1378a
- [x] Task: Implement `prompts/get` handler in `McpHandler`. fa4b7ec

## Phase 2: Implement Specific Prompts
- [x] Task: Implement `DiagnoseDownloadPrompt` logic. 2ebbbb2
- [x] Task: Implement `OptimizeSchedulePrompt` logic. 81e5ad5

## Phase 3: Integration & Testing
- [x] Task: Add unit tests for prompt registry and handlers. 43be131
- [ ] Task: Add integration tests verifying `prompts/list` and `prompts/get` via MCP protocol.
- [ ] Task: Update documentation to list available prompts.