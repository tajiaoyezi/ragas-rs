# language: en
# Maps to:
#   - docs/specs/tasks/task-9.1-metric-base.md

Feature: metric-base
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want full metric traits: single/multi-turn, LLM/embedding requirements, batch hooks

  Scenario: SCEN-9.1.1 Metric traits distinguish single-turn, multi-turn, LLM, and embedding requirements
    Given the complete refactor task 9.1
    When TEST-9.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-9.1.2 Batch scoring hooks default to per-sample behavior
    Given the complete refactor task 9.1
    When TEST-9.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-9.1.3 Metric metadata declares required sample fields
    Given the complete refactor task 9.1
    When TEST-9.1.3 is executed
    Then the behavior matches the task acceptance criterion
