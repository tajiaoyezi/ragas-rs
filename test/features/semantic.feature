# language: en
# Maps to:
#   - docs/specs/tasks/task-11.2-semantic.md

Feature: semantic
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want embedding similarity and thresholded semantic metrics

  Scenario: SCEN-11.2.1 Semantic similarity uses embedding provider with batching
    Given the complete refactor task 11.2
    When TEST-11.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-11.2.2 Threshold policy is configurable
    Given the complete refactor task 11.2
    When TEST-11.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-11.2.3 Scores are stable for zero vectors
    Given the complete refactor task 11.2
    When TEST-11.2.3 is executed
    Then the behavior matches the task acceptance criterion
