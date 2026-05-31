# language: en
# Maps to:
#   - docs/specs/tasks/task-9.3-metric-registry.md

Feature: metric-registry
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want metric collection registry, feature flags, parity status labels

  Scenario: SCEN-9.3.1 Metric registry resolves built-ins by stable names
    Given the complete refactor task 9.3
    When TEST-9.3.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-9.3.2 Feature-gated metrics are hidden unless enabled
    Given the complete refactor task 9.3
    When TEST-9.3.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-9.3.3 Parity status labels are exported for docs and tests
    Given the complete refactor task 9.3
    When TEST-9.3.3 is executed
    Then the behavior matches the task acceptance criterion
