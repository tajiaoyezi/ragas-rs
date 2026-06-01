Feature: parity fixture policy
  Scenario: parity complete requires fixtures
    Given a feature marked parity complete
    When it has no fixture reference
    Then validation fails

  Scenario: fixture metadata explains comparison mode
    Given a parity fixture
    When the fixture metadata is inspected
    Then upstream path, mode, and tolerance are recorded

  Scenario: gaps block release by default
    Given partial and known gap entries
    When release readiness is evaluated
    Then release readiness is blocked

