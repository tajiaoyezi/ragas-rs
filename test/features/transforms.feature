# language: en
# Maps to:
#   - docs/specs/tasks/task-13.2-transforms.md

Feature: transforms
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want splitters, extractors, filters, relationship builders

  Scenario: SCEN-13.2.1 Splitters produce stable chunks with source metadata
    Given the complete refactor task 13.2
    When TEST-13.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-13.2.2 Extractors attach entities/themes/summaries
    Given the complete refactor task 13.2
    When TEST-13.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-13.2.3 Relationship builders create deterministic edges
    Given the complete refactor task 13.2
    When TEST-13.2.3 is executed
    Then the behavior matches the task acceptance criterion
