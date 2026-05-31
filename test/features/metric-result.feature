# language: en
# Maps to:
#   - docs/specs/tasks/task-9.2-metric-result.md

Feature: metric-result
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want result schema, score normalization, reason/evidence, error taxonomy

  Scenario: SCEN-9.2.1 Metric result stores score, value type, reason, evidence, and error
    Given the complete refactor task 9.2
    When TEST-9.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-9.2.2 Score normalization clamps or rejects invalid numeric scores by policy
    Given the complete refactor task 9.2
    When TEST-9.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-9.2.3 Error taxonomy distinguishes provider, parse, validation, and metric failures
    Given the complete refactor task 9.2
    When TEST-9.2.3 is executed
    Then the behavior matches the task acceptance criterion
