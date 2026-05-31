# language: en
# Maps to:
#   - docs/specs/tasks/task-15.3-benchmarks.md

Feature: benchmarks
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want LLM/embedding benchmark runner and cost summaries

  Scenario: SCEN-15.3.1 Benchmark runner executes providers over fixed prompts
    Given the complete refactor task 15.3
    When TEST-15.3.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-15.3.2 Cost summary aggregates usage and configured rates
    Given the complete refactor task 15.3
    When TEST-15.3.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-15.3.3 Benchmark output is stable JSON
    Given the complete refactor task 15.3
    When TEST-15.3.3 is executed
    Then the behavior matches the task acceptance criterion
