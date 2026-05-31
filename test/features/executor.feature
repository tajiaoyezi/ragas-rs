# language: en
# Maps to:
#   - docs/specs/tasks/task-6.2-executor.md

Feature: executor
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want ordered async executor、partial failure isolation、progress events

  Scenario: SCEN-6.2.1 Executor preserves output order for concurrent tasks
    Given the complete refactor task 6.2
    When TEST-6.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-6.2.2 Executor records partial failures without aborting unrelated work
    Given the complete refactor task 6.2
    When TEST-6.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-6.2.3 Progress events are emitted for start, success, and failure
    Given the complete refactor task 6.2
    When TEST-6.2.3 is executed
    Then the behavior matches the task acceptance criterion
