# language: en
# Maps to:
#   - docs/specs/tasks/task-5.3-validation.md

Feature: validation
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want sample/metric compatibility validator、required column checker

  Scenario: SCEN-5.3.1 Validator detects missing fields required by a metric
    Given the complete refactor task 5.3
    When TEST-5.3.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-5.3.2 Validator reports sample index and field path for invalid records
    Given the complete refactor task 5.3
    When TEST-5.3.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-5.3.3 Validation can run before evaluate and fail without provider calls
    Given the complete refactor task 5.3
    When TEST-5.3.3 is executed
    Then the behavior matches the task acceptance criterion
