# language: en
# Maps to:
#   - docs/specs/tasks/task-8.3-multimodal-prompt.md

Feature: multimodal-prompt
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want image/text prompt scaffold and typed multimodal message model

  Scenario: SCEN-8.3.1 Multimodal message supports text and image parts
    Given the complete refactor task 8.3
    When TEST-8.3.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-8.3.2 Prompt rendering preserves part order
    Given the complete refactor task 8.3
    When TEST-8.3.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-8.3.3 Unsupported media returns structured error
    Given the complete refactor task 8.3
    When TEST-8.3.3 is executed
    Then the behavior matches the task acceptance criterion
