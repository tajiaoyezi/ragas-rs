Feature: graph persistence query parity
  Scenario: graph fixture roundtrips deterministically
    Given a graph fixture
    When it is saved and loaded
    Then nodes, edges, and typed properties are preserved

  Scenario: graph query contracts are explicit
    Given graph query descriptors
    When filters are inspected
    Then node type, property, and relationship filters are covered

  Scenario: missing graph features block release
    Given an unsupported upstream graph feature
    When parity claims are evaluated
    Then release readiness is blocked
