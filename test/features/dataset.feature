# language: en
# Maps to:
#   - docs/specs/tasks/task-1.1-foundation-dataset.md

Feature: dataset
  In order to run repeatable RAG evaluations
  As a Rust caller
  I want validated single-turn samples grouped into an EvaluationDataset

  Scenario: SCEN-1.1.1 valid sample fields are preserved
    Given a sample with question, answer, contexts, reference, and metadata
    When the caller constructs a SingleTurnSample
    Then TEST-1.1.1 observes the same values through public fields

  Scenario: SCEN-1.1.2 valid dataset exposes collection helpers
    Given one or more valid samples
    When the caller constructs an EvaluationDataset
    Then TEST-1.1.2 observes len, is_empty, iter, and samples behavior

  Scenario: SCEN-1.1.3 invalid dataset reports sample index
    Given empty datasets or samples missing required fields
    When validation runs
    Then TEST-1.1.3 receives RagasError::InvalidSample with the failing index
