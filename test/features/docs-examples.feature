# language: en
# Maps to:
#   - docs/specs/tasks/task-16.2-docs-examples.md

Feature: docs-examples
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want Rust examples mapped to upstream howtos/tutorials

  Scenario: SCEN-16.2.1 Each public workflow has a runnable Rust example
    Given the complete refactor task 16.2
    When TEST-16.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-16.2.2 Examples map to upstream docs section names
    Given the complete refactor task 16.2
    When TEST-16.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-16.2.3 Docs state feature flags and known parity gaps
    Given the complete refactor task 16.2
    When TEST-16.2.3 is executed
    Then the behavior matches the task acceptance criterion
