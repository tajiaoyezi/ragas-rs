Feature: metric catalog inventory
  Scenario: metric catalog lists upstream families
    Given the metric catalog
    When metric families are listed
    Then every upstream family has a Rust owner or known gap

  Scenario: metric descriptors expose scoring contracts
    Given a metric descriptor
    When the descriptor is inspected
    Then sample kind, provider requirement, output type, and fixture status are present

  Scenario: missing metric parity blocks release
    Given a metric without complete fixture parity
    When parity claims are evaluated
    Then release readiness is blocked
