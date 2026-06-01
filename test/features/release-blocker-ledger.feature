Feature: release blocker ledger
  Scenario: blocker ledger aggregates sources
    Given module-level blocker claims
    When the ledger is built
    Then provider, backend, integration, metric, testset, optimizer, docs, and quality blockers are included

  Scenario: blockers have release metadata
    Given a blocker entry
    When it is inspected
    Then category, feature, severity, source, and impact are present

  Scenario: remaining blocker prevents release
    Given a non-waived blocker
    When release readiness is evaluated
    Then release is refused
