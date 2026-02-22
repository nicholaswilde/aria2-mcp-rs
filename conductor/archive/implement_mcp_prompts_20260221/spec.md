# Track: Implement MCP Prompts

## Overview
This track adds support for "Prompts" in the Model Context Protocol (MCP). Prompts are predefined interaction templates that help users and LLMs complete complex tasks efficiently by providing structured guidance and pre-selecting relevant tools or resources.

## Functional Requirements
- **Prompt Registry**: Implement a registry to manage and list available prompts.
- **Prompt Handlers**: Implement handlers for specific prompts:
    - `diagnose-download`: A prompt that guides the user to check download status, logs, and health.
        - **Arguments**: `gid` (optional download GID).
        - **Context**: Loads `check_health` tool schema and `aria2://logs/recent` resource reference.
    - `optimize-schedule`: A prompt to review and adjust bandwidth schedules.
        - **Context**: Loads `schedule_limits` tool and current schedule configuration.
- **McpServer Update**: Update `McpServer` to handle `prompts/list` and `prompts/get` requests.

## Non-Functional Requirements
- **Helpful Descriptions**: Prompts must have clear descriptions and argument definitions.
- **Context Awareness**: Prompts should leverage existing resources and tools effectively.

## Acceptance Criteria
- [ ] `prompts/list` returns the list of available prompts.
- [ ] `prompts/get` with `diagnose-download` returns the prompt structure with correct arguments and context.
- [ ] `prompts/get` with `optimize-schedule` returns the prompt structure.
- [ ] Integration tests verify prompt availability and structure.