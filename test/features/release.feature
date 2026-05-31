# language: en
# Maps to:
#   - docs/specs/tasks/task-16.3-release.md

Feature: release
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want feature flags, crate metadata, CI gates, release checklist

  Scenario: SCEN-16.3.1 Cargo features match optional capability groups
    Given the complete refactor task 16.3
    When TEST-16.3.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-16.3.2 CI runs build, check, test, and parity gates
    Given the complete refactor task 16.3
    When TEST-16.3.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-16.3.3 Release checklist includes versioning and rollback steps
    Given the complete refactor task 16.3
    When TEST-16.3.3 is executed
    Then the behavior matches the task acceptance criterion
