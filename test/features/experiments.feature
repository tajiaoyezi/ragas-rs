# language: en
# Maps to:
#   - docs/specs/tasks/task-15.1-experiments.md

Feature: experiments
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want experiment record model, compare runs, report summaries

  Scenario: SCEN-15.1.1 Experiment records inputs, metrics, provider config, and outputs
    Given the complete refactor task 15.1
    When TEST-15.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-15.1.2 Compare runs computes metric deltas
    Given the complete refactor task 15.1
    When TEST-15.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-15.1.3 Report summary serializes to JSON
    Given the complete refactor task 15.1
    When TEST-15.1.3 is executed
    Then the behavior matches the task acceptance criterion
