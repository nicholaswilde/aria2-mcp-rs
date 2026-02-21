# Product Guide - aria2-mcp-rs

## Initial Concept
A rust implementation of a aria2 MCP server.

## Overview
`aria2-mcp-rs` is a robust and flexible Model Context Protocol (MCP) server for `aria2`, written in Rust. It enables seamless integration of `aria2`'s powerful download capabilities into the MCP ecosystem, allowing LLMs and other agents to manage downloads efficiently.

## Target Users
- **Power Users**: Power users who want to automate their download workflows.

## Core Features
- **Download Control**: Support for adding, pausing, resuming, and removing downloads via MCP. Includes advanced control like force-pause, force-remove, moving download positions, bulk management, and specialized BitTorrent management (trackers, peers, file selection).
- **Status Monitoring**: High-level visibility into the download queue, including active, waiting, and stopped tasks, global statistics, and automated health monitoring for stalled downloads and system resources.
- **Configuration Management**: Tools to manage aria2 settings (global and per-download) on the fly.
- **Download Inspection**: Detailed inspection of specific download tasks, including file lists and URI information.

## User Experience
The project provides a dual interface approach:
- **MCP Server**: A pure MCP server for programmatic access by AI agents, supporting both Stdio and SSE transports.
- **CLI**: A command-line tool for direct interaction alongside the MCP server.

## Success Criteria
- **Robustness & Flexibility**: A robust and highly configurable server for complex use cases.
- **Reliability**: Maintained with high code coverage (>90%) to ensure long-term stability and prevent regressions.
- **Token Efficiency**: Optimized to save on LLM tokens using MCP server best practices (e.g., concise tool outputs, structured data, and context-aware responses).
