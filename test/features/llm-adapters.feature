# language: en
# Maps to:
#   - docs/specs/tasks/task-7.2-llm-adapters.md

Feature: llm-adapters
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want OpenAI-compatible completion polish、Azure/local-compatible config

  Scenario: SCEN-7.2.1 OpenAI-compatible chat client supports base URL, model, and headers
    Given the complete refactor task 7.2
    When TEST-7.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-7.2.2 Azure-compatible config maps deployment name and API version
    Given the complete refactor task 7.2
    When TEST-7.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-7.2.3 HTTP errors are sanitized and preserve status/body summary
    Given the complete refactor task 7.2
    When TEST-7.2.3 is executed
    Then the behavior matches the task acceptance criterion
