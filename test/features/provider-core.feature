# language: en
# Maps to:
#   - docs/specs/tasks/task-7.1-provider-core.md

Feature: provider-core
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want provider registry、mock providers、usage accounting

  Scenario: SCEN-7.1.1 Provider registry resolves LLM and embedding providers by name
    Given the complete refactor task 7.1
    When TEST-7.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-7.1.2 Mock providers support deterministic unit tests
    Given the complete refactor task 7.1
    When TEST-7.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-7.1.3 Provider responses carry usage accounting when available
    Given the complete refactor task 7.1
    When TEST-7.1.3 is executed
    Then the behavior matches the task acceptance criterion
