# language: en
# Maps to:
#   - docs/specs/tasks/task-12.1-rubrics.md

Feature: rubrics
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want aspect critic, simple criteria, domain/instance rubrics

  Scenario: SCEN-12.1.1 Rubric metrics accept typed criteria
    Given the complete refactor task 12.1
    When TEST-12.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-12.1.2 Aspect critic returns binary or graded result according to config
    Given the complete refactor task 12.1
    When TEST-12.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-12.1.3 Domain and instance rubrics serialize for audit
    Given the complete refactor task 12.1
    When TEST-12.1.3 is executed
    Then the behavior matches the task acceptance criterion
