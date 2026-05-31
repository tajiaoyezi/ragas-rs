# language: en
# Maps to:
#   - docs/specs/tasks/task-14.3-cli.md

Feature: cli
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want ragas evaluate, ragas testset, ragas benchmark

  Scenario: SCEN-14.3.1 CLI evaluate reads dataset and writes report
    Given the complete refactor task 14.3
    When TEST-14.3.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-14.3.2 CLI testset invokes synthesizer flow
    Given the complete refactor task 14.3
    When TEST-14.3.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-14.3.3 CLI benchmark prints machine-readable summary
    Given the complete refactor task 14.3
    When TEST-14.3.3 is executed
    Then the behavior matches the task acceptance criterion
