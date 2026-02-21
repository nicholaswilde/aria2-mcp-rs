# Specification: Increase Code Coverage to >90% with Coveralls

## Overview
This track aims to increase the `aria2-mcp-rs` project's code coverage to above 90% and ensure the reporting mechanism to Coveralls.io is fully functional using the project's `Taskfile.yml` commands.

## Functional Requirements
- **Coverage Expansion:**
    - Increase test coverage for `src/main.rs`, `src/server/` (handler, mod, sse, stdio), and `src/tools/` (manage_downloads, monitor_queue, registry).
    - Maintain and improve coverage for `src/aria2/mod.rs` and `src/config.rs`.
- **Tooling Verification:**
    - Ensure `task coverage`, `task coverage:report`, and `task coverage:upload` are correctly configured and functional.
    - Verify that `lcov.info` is generated correctly.
- **Coveralls.io Integration:**
    - Successfully upload coverage reports using `task coverage:upload`.
    - Use `COVERALLS_REPO_TOKEN` managed via SOPS-encrypted `.env.enc`.

## Non-Functional Requirements
- **Coverage Target:** >90% total line coverage.
- **Maintainability:** Coverage tasks should remain independent and easy to run locally.

## Acceptance Criteria
- [ ] Total project line coverage is at or above 90% as reported by `cargo llvm-cov`.
- [ ] Significant coverage gaps in `src/server/` and `src/tools/` are addressed.
- [ ] `task coverage:upload` successfully pushes data to Coveralls.io (verified with a manual check).
- [ ] `.env.enc` contains the necessary `COVERALLS_REPO_TOKEN`.

## Out of Scope
- Integration of coverage reporting into CI/CD (GitHub Actions) for this specific track.
- 100% coverage (90% is the target).
