# language: en
# Maps to:
#   - docs/specs/tasks/task-4.1-evaluator-builtins.md

Feature: eval
  In order to run RAG evaluation in Rust services
  As a Rust caller
  I want asynchronous batch evaluation and built-in metrics

  Scenario: SCEN-4.1.1 evaluate isolates metric failures
    Given a dataset and two metrics where one fails
    When evaluate runs
    Then TEST-4.1.1 returns one result per sample per metric and records the failure

  Scenario: SCEN-4.1.2 faithfulness parses LLM judgement
    Given an LLM provider returning JSON score and reason
    When FaithfulnessMetric scores a sample
    Then TEST-4.1.2 returns a numeric score with reason

  Scenario: SCEN-4.1.3 response relevancy uses cosine similarity
    Given embeddings for question and response
    When ResponseRelevancyMetric scores a sample
    Then TEST-4.1.3 returns their cosine similarity

  Scenario: SCEN-4.1.4 context precision computes average precision
    Given embeddings for question and retrieved contexts
    When ContextPrecisionMetric scores a sample
    Then TEST-4.1.4 returns average precision over relevant contexts
