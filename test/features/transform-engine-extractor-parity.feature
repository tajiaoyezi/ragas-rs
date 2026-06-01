Feature: transform engine extractor parity
  Scenario: transform registry lists stages
    Given the transform registry
    When stages are listed
    Then splitters, extractors, and relationship builders have mode metadata

  Scenario: extractor outputs normalize into graph properties
    Given extractor outputs
    When they are attached to graph nodes
    Then entities, themes, summaries, and relationships are stable

  Scenario: unsupported transform stage blocks release
    Given an unsupported upstream transform stage
    When parity claims are evaluated
    Then release readiness is blocked
