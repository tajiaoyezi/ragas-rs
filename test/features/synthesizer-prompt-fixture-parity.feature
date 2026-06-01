Feature: synthesizer prompt fixture parity
  Scenario: synthesizer registry lists strategies
    Given the synthesizer registry
    When strategies are listed
    Then single-hop, multi-hop, and pre-chunked strategies are represented

  Scenario: prompt snapshots preserve rendered order
    Given a synthesizer prompt snapshot
    When it is rendered
    Then variables and message order are preserved

  Scenario: unfixture-backed synthesizer blocks release
    Given a synthesizer without fixture evidence
    When parity claims are evaluated
    Then release readiness is blocked
