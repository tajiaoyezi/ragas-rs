# language: en
# Maps to:
#   - docs/specs/tasks/task-12.3-sql-multimodal-summary.md

Feature: sql-multimodal-summary
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want SQL semantic equivalence, multimodal faithfulness/relevance, summarization

  Scenario: SCEN-12.3.1 SQL semantic equivalence compares normalized SQL or judge output
    Given the complete refactor task 12.3
    When TEST-12.3.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-12.3.2 Multimodal metrics route through multimodal prompt model
    Given the complete refactor task 12.3
    When TEST-12.3.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-12.3.3 Summarization score parses coverage and conciseness signals
    Given the complete refactor task 12.3
    When TEST-12.3.3 is executed
    Then the behavior matches the task acceptance criterion
