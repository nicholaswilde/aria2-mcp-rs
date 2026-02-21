# Specification - Search & Filter Downloads

## Overview
Implement a `search_downloads` tool that allows users to find specific downloads based on various criteria. This tool will help save tokens by returning only relevant download metadata instead of the entire queue.

## User Stories
- As a user, I want to find a download by its filename or URI so I don't have to scan the whole list.
- As a power user, I want to filter downloads by status (e.g., "active", "waiting") or GID range.

## Functional Requirements
- Support case-insensitive substring matching for filenames and URIs.
- Support filtering by status.
- Allow specifying which keys (fields) to return for each match.
- Return a list of matching downloads with their metadata.

## Technical Requirements
- The tool should utilize `aria2.tellActive`, `aria2.tellWaiting`, and `aria2.tellStopped` to gather data and then filter it in Rust.
- Efficiently handle large queues by applying filters as early as possible.
