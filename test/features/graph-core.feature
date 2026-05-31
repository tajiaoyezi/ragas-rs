# language: en
# Maps to:
#   - docs/specs/tasks/task-13.1-graph-core.md

Feature: graph-core
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want knowledge graph node/edge model and graph queries

  Scenario: SCEN-13.1.1 Graph stores nodes, relationships, and typed properties
    Given the complete refactor task 13.1
    When TEST-13.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-13.1.2 Graph queries filter by type and relationship
    Given the complete refactor task 13.1
    When TEST-13.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-13.1.3 Graph serialization roundtrips fixtures
    Given the complete refactor task 13.1
    When TEST-13.1.3 is executed
    Then the behavior matches the task acceptance criterion
