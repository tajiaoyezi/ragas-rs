Feature: bug zero release audit
  Scenario: defects carry release evidence
    Given a bug ledger entry
    When the entry is inspected
    Then severity, status, affected feature, evidence, and regression test reference are present

  Scenario: high severity unresolved bugs block release
    Given an unresolved high severity correctness bug
    When release readiness is evaluated
    Then release readiness is blocked

  Scenario: ready release reports no blocking bugs
    Given all release-blocking bugs are resolved
    When the audit is summarized
    Then the report shows zero unresolved release-blocking bugs

