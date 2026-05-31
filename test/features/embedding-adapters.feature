# language: en
# Maps to:
#   - docs/specs/tasks/task-7.3-embedding-adapters.md

Feature: embedding-adapters
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want OpenAI-compatible embeddings、batching、normalization

  Scenario: SCEN-7.3.1 Embedding provider batches inputs without reordering outputs
    Given the complete refactor task 7.3
    When TEST-7.3.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-7.3.2 Optional vector normalization is deterministic
    Given the complete refactor task 7.3
    When TEST-7.3.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-7.3.3 Embedding errors include request batch position
    Given the complete refactor task 7.3
    When TEST-7.3.3 is executed
    Then the behavior matches the task acceptance criterion
