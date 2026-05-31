# language: en
# Maps to:
#   - docs/specs/tasks/task-5.1-schema-core.md

Feature: schema-core
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want MultiTurnSample、Message、ToolCall、rubric/reference/metadata schema

  Scenario: SCEN-5.1.1 Message and ToolCall model supports user/assistant/system/tool roles and tool-call IDs
    Given the complete refactor task 5.1
    When TEST-5.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-5.1.2 MultiTurnSample preserves ordered messages, reference, rubrics, and metadata
    Given the complete refactor task 5.1
    When TEST-5.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-5.1.3 Schema types serialize and deserialize without losing optional fields
    Given the complete refactor task 5.1
    When TEST-5.1.3 is executed
    Then the behavior matches the task acceptance criterion
