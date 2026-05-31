# language: en
# Maps to:
#   - docs/specs/tasks/task-13.3-synthesizers.md

Feature: synthesizers
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want persona, single-hop, multi-hop synthesizers

  Scenario: SCEN-13.3.1 Persona generator stores name, role, and goals
    Given the complete refactor task 13.3
    When TEST-13.3.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-13.3.2 Single-hop synthesizer creates samples from one chunk
    Given the complete refactor task 13.3
    When TEST-13.3.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-13.3.3 Multi-hop synthesizer combines related graph nodes
    Given the complete refactor task 13.3
    When TEST-13.3.3 is executed
    Then the behavior matches the task acceptance criterion
