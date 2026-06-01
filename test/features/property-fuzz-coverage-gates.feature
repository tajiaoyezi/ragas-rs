Feature: property fuzz coverage gates
  Scenario: quality gates describe commands
    Given property, fuzz, and coverage gates
    When gates are listed
    Then command, scope, and required mode are present

  Scenario: missing required quality evidence blocks release
    Given required quality evidence is absent
    When release gates are evaluated
    Then release readiness is blocked

  Scenario: optional long running gates remain visible
    Given optional fuzz evidence
    When default CI gates are evaluated
    Then optional gates do not block deterministic CI
