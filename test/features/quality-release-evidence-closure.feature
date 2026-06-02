Feature: quality release evidence closure

  Scenario: required quality evidence is complete
    Given the required release quality descriptors
    When release quality evidence records are loaded
    Then every required descriptor has passed evidence
    And optional long-running gates remain outside the release blocker set

  Scenario: release ledger has no remaining blockers
    Given all parity categories are complete
    And required quality evidence is present
    When the consolidated release ledger is summarized
    Then the ledger has zero blockers
    And release readiness is true

  Scenario: final audit stays scoped
    Given complete final audit evidence
    And an empty release blocker ledger
    And no release-blocking bugs
    When the final bug-zero audit is rendered
    Then release readiness is true
    And the statement avoids absolute bug-free claims
