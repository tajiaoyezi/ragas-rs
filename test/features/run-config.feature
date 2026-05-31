# language: en
# Maps to:
#   - docs/specs/tasks/task-6.1-run-config.md

Feature: run-config
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want timeout/retry/concurrency/cancellation model

  Scenario: SCEN-6.1.1 RunConfig stores timeout, retry, concurrency, and cancellation settings
    Given the complete refactor task 6.1
    When TEST-6.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-6.1.2 Defaults are conservative and deterministic
    Given the complete refactor task 6.1
    When TEST-6.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-6.1.3 Invalid config returns structured errors
    Given the complete refactor task 6.1
    When TEST-6.1.3 is executed
    Then the behavior matches the task acceptance criterion
