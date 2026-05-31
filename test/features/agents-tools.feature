# language: en
# Maps to:
#   - docs/specs/tasks/task-12.2-agents-tools.md

Feature: agents-tools
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want goal accuracy, tool call accuracy, tool call F1, topic adherence

  Scenario: SCEN-12.2.1 Tool call metrics compare names, args, and order policy
    Given the complete refactor task 12.2
    When TEST-12.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-12.2.2 Agent goal accuracy supports multi-turn traces
    Given the complete refactor task 12.2
    When TEST-12.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-12.2.3 Topic adherence records per-topic evidence
    Given the complete refactor task 12.2
    When TEST-12.2.3 is executed
    Then the behavior matches the task acceptance criterion
