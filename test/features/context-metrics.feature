# language: en
# Maps to:
#   - docs/specs/tasks/task-10.1-context-metrics.md

Feature: context-metrics
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want context precision/recall/entity recall/relevance variants

  Scenario: SCEN-10.1.1 Context precision variants match declared formulas
    Given the complete refactor task 10.1
    When TEST-10.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-10.1.2 Context recall and entity recall operate on references and contexts
    Given the complete refactor task 10.1
    When TEST-10.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-10.1.3 Context relevance returns score with evidence
    Given the complete refactor task 10.1
    When TEST-10.1.3 is executed
    Then the behavior matches the task acceptance criterion
