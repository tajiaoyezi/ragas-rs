Feature: provider adapter contracts
  Scenario: provider descriptors classify upstream families
    Given the provider registry
    When provider families are listed
    Then each upstream family has a deterministic or live mode

  Scenario: structured LLM supports system prompt metadata
    Given a structured LLM provider descriptor
    When system prompt support is inspected
    Then Instructor and LiteLLM structured families are represented

  Scenario: unsupported provider blocks release
    Given an unsupported live provider family
    When parity claims are evaluated
    Then release readiness is blocked

