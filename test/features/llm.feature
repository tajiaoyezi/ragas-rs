# language: en
# Maps to:
#   - docs/specs/tasks/task-3.1-providers.md

Feature: llm
  In order to use LLM-as-judge and embedding-based metrics
  As a Rust caller
  I want OpenAI-compatible provider traits and DTO helpers

  Scenario: SCEN-3.1.1 chat parser extracts assistant content
    Given an OpenAI-compatible chat completion JSON body
    When parse_chat_response runs
    Then TEST-3.1.1 returns assistant content and usage

  Scenario: SCEN-3.1.2 embedding parser preserves vector order
    Given an OpenAI-compatible embeddings JSON body
    When parse_embedding_response runs
    Then TEST-3.1.2 returns embeddings in response order

  Scenario: SCEN-3.1.3 provider errors sanitize credentials
    Given an OpenAI-compatible client with an API key
    When an HTTP error is converted to RagasError
    Then TEST-3.1.3 does not expose the secret
