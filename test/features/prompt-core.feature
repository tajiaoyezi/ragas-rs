# language: en
# Maps to:
#   - docs/specs/tasks/task-8.1-prompt-core.md

Feature: prompt-core
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want typed prompt template、few-shot examples、language adaptation hooks

  Scenario: SCEN-8.1.1 Prompt template renders typed variables with missing-variable diagnostics
    Given the complete refactor task 8.1
    When TEST-8.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-8.1.2 Few-shot examples can be attached and serialized
    Given the complete refactor task 8.1
    When TEST-8.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-8.1.3 Language adaptation hook can rewrite prompt text deterministically
    Given the complete refactor task 8.1
    When TEST-8.1.3 is executed
    Then the behavior matches the task acceptance criterion
