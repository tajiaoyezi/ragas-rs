# language: en
# Maps to:
#   - docs/specs/tasks/task-14.1-backends.md

Feature: backends
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want in-memory, JSONL, CSV backend registry

  Scenario: SCEN-14.1.1 Backend trait supports save, load, list, and delete
    Given the complete refactor task 14.1
    When TEST-14.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-14.1.2 In-memory backend is deterministic for tests
    Given the complete refactor task 14.1
    When TEST-14.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-14.1.3 JSONL and CSV local backends preserve dataset schema
    Given the complete refactor task 14.1
    When TEST-14.1.3 is executed
    Then the behavior matches the task acceptance criterion
