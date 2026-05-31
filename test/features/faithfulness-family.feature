# language: en
# Maps to:
#   - docs/specs/tasks/task-10.2-faithfulness-family.md

Feature: faithfulness-family
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want faithfulness, response groundedness, factual correctness

  Scenario: SCEN-10.2.1 Faithfulness uses prompt/parser contract from phase 8
    Given the complete refactor task 10.2
    When TEST-10.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-10.2.2 Response groundedness records supporting context evidence
    Given the complete refactor task 10.2
    When TEST-10.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-10.2.3 Factual correctness handles TP/FP/FN style output
    Given the complete refactor task 10.2
    When TEST-10.2.3 is executed
    Then the behavior matches the task acceptance criterion
