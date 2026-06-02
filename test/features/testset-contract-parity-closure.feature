Feature: testset contract parity closure

  Scenario: graph cluster and advanced query contracts close graph blockers
    Given the current upstream testset graph baseline
    When deterministic graph cluster and advanced query descriptors are loaded
    Then graph parity claims are fixture-backed complete claims
    And graph cluster and advanced query APIs return stable ordered results

  Scenario: transform LLM extractor and filter contracts close transform blockers
    Given captured upstream-style extractor output
    When Rust parses extractor output and filters the graph
    Then transform parity claims are fixture-backed complete claims
    And no live LLM call is required in default CI

  Scenario: pre-chunked synthesizer closure removes testset release blockers
    Given pre-chunked text chunks and persona data
    When Rust synthesizes deterministic pre-chunked samples
    Then synthesizer parity claims are fixture-backed complete claims
    And the release ledger has no Testset category
