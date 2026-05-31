# language: en
# Maps to:
#   - docs/specs/tasks/task-15.2-optimizers.md

Feature: optimizers
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want prompt/model optimization abstractions and genetic optimizer scaffold

  Scenario: SCEN-15.2.1 Optimizer trait accepts objective metric and candidate generator
    Given the complete refactor task 15.2
    When TEST-15.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-15.2.2 Genetic optimizer scaffold evolves candidates deterministically with seeded RNG
    Given the complete refactor task 15.2
    When TEST-15.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-15.2.3 Optimizer history is inspectable
    Given the complete refactor task 15.2
    When TEST-15.2.3 is executed
    Then the behavior matches the task acceptance criterion
