# language: en
# Maps to:
#   - docs/specs/tasks/task-11.3-quoted-spans.md

Feature: quoted-spans
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want quoted spans and citation overlap metrics

  Scenario: SCEN-11.3.1 Quoted span extraction preserves byte and char ranges
    Given the complete refactor task 11.3
    When TEST-11.3.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-11.3.2 Overlap scoring handles partial matches
    Given the complete refactor task 11.3
    When TEST-11.3.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-11.3.3 Missing citations produce explicit zero-score reason
    Given the complete refactor task 11.3
    When TEST-11.3.3 is executed
    Then the behavior matches the task acceptance criterion
