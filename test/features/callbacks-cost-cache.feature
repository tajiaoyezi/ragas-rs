# language: en
# Maps to:
#   - docs/specs/tasks/task-6.3-callbacks-cost-cache.md

Feature: callbacks-cost-cache
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want callbacks、token usage/cost model、cache key/value abstraction

  Scenario: SCEN-6.3.1 Callback hooks receive evaluation lifecycle events
    Given the complete refactor task 6.3
    When TEST-6.3.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-6.3.2 Token usage aggregates per provider and metric
    Given the complete refactor task 6.3
    When TEST-6.3.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-6.3.3 Cache key derivation is stable and redacts secrets
    Given the complete refactor task 6.3
    When TEST-6.3.3 is executed
    Then the behavior matches the task acceptance criterion
