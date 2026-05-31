# language: en
# Maps to:
#   - docs/specs/tasks/task-16.1-parity-suite.md

Feature: parity-suite
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want upstream golden fixtures, gap matrix, parity status reports

  Scenario: SCEN-16.1.1 Parity fixture format stores Python baseline and Rust output
    Given the complete refactor task 16.1
    When TEST-16.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-16.1.2 Gap matrix lists Complete, Partial, and Known Gap per feature
    Given the complete refactor task 16.1
    When TEST-16.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-16.1.3 Parity tests fail on undeclared semantic drift
    Given the complete refactor task 16.1
    When TEST-16.1.3 is executed
    Then the behavior matches the task acceptance criterion
