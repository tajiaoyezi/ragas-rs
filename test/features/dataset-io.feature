# language: en
# Maps to:
#   - docs/specs/tasks/task-5.2-dataset-io.md

Feature: dataset-io
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want JSONL/CSV serde roundtrip、dataset builders、validation diagnostics

  Scenario: SCEN-5.2.1 Dataset can load and save JSONL for single-turn and multi-turn samples
    Given the complete refactor task 5.2
    When TEST-5.2.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-5.2.2 CSV import maps required columns into SingleTurnSample with clear errors
    Given the complete refactor task 5.2
    When TEST-5.2.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-5.2.3 Dataset builders preserve sample order and metadata
    Given the complete refactor task 5.2
    When TEST-5.2.3 is executed
    Then the behavior matches the task acceptance criterion
