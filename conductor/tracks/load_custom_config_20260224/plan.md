# Implementation Plan - Load Custom Configuration Files

## Phase 1: CLI Argument Implementation
- [x] Task: Update `Args` struct in `src/main.rs` to include `--config` and `-c`. 98c5704
- [ ] Task: Implement TDD for CLI argument parsing.
    - [ ] Write failing tests in `tests/main_test.rs` for `--config` and `-c` flags.
    - [ ] Implement logic in `Args` to handle the new arguments.
    - [ ] Verify tests pass.
- [ ] Task: Conductor - User Manual Verification 'CLI Argument Implementation' (Protocol in workflow.md)

## Phase 2: Exclusive Configuration Loading
- [ ] Task: Refactor `Config::load` in `src/config.rs` to accept an optional custom path.
- [ ] Task: Implement TDD for exclusive configuration loading.
    - [ ] Write failing tests in `tests/aria2_config_tests.rs` to verify that specifying a file bypasses defaults.
    - [ ] Update `Config::load` implementation to use the provided path exclusively if present.
    - [ ] Verify tests pass.
- [ ] Task: Conductor - User Manual Verification 'Exclusive Configuration Loading' (Protocol in workflow.md)

## Phase 3: Error Handling and Validation
- [ ] Task: Implement warning logs for missing or invalid custom configuration files.
- [ ] Task: Implement TDD for error handling.
    - [ ] Write failing tests to verify that the server starts with defaults when a custom file is missing/invalid.
    - [ ] Update `Config::load` to handle I/O and parsing errors gracefully with warnings.
    - [ ] Verify tests pass.
- [ ] Task: Conductor - User Manual Verification 'Error Handling and Validation' (Protocol in workflow.md)

## Phase 4: Final Verification and Documentation
- [ ] Task: Run project-wide checks using `/test-fix`.
- [ ] Task: Verify code coverage meets >80% for new code.
- [ ] Task: Document the new `--config` flag in `README.md`.
- [ ] Task: Conductor - User Manual Verification 'Final Verification and Documentation' (Protocol in workflow.md)
