# language: en
# Maps to:
#   - docs/specs/tasks/task-8.2-output-parser.md

Feature: output-parser
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want JSON/schema parser、repair strategy、malformed output diagnostics

  Scenario: SCEN-8.2.1 Parser extracts typed JSON scores and reasons
    Given the complete refactor task 8.2
    When TEST-8.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-8.2.2 Malformed judge output returns parse diagnostics with raw excerpt
    Given the complete refactor task 8.2
    When TEST-8.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-8.2.3 Repair strategy is explicit and testable
    Given the complete refactor task 8.2
    When TEST-8.2.3 is executed
    Then the behavior matches the task acceptance criterion
