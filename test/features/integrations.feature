# language: en
# Maps to:
#   - docs/specs/tasks/task-14.2-integrations.md

Feature: integrations
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want tracing hooks and optional LangSmith/Langfuse/Opik-style adapters

  Scenario: SCEN-14.2.1 Tracing integration receives callback events
    Given the complete refactor task 14.2
    When TEST-14.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-14.2.2 External integrations are feature-gated
    Given the complete refactor task 14.2
    When TEST-14.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-14.2.3 Payload redaction is applied before export
    Given the complete refactor task 14.2
    When TEST-14.2.3 is executed
    Then the behavior matches the task acceptance criterion
