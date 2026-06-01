Feature: quickstart docs parity
  Scenario: quickstart registry maps upstream templates
    Given the quickstart registry
    When quickstarts are listed
    Then each upstream template maps to a Rust example or known gap

  Scenario: docs example metadata is runnable
    Given a docs example descriptor
    When metadata is inspected
    Then command, output type, and feature flags are present

  Scenario: missing docs example blocks release
    Given a missing upstream quickstart
    When parity claims are evaluated
    Then release readiness is blocked
