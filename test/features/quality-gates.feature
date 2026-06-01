Feature: quality gates
  Scenario: required gate kinds are explicit
    Given the release gate policy
    When the required gates are listed
    Then build, typecheck, unit, integration, parity, examples, coverage, fuzz, and bug-ledger gates are present

  Scenario: evidence states are distinguishable
    Given a release gate report
    When evidence is summarized
    Then passed, failed, skipped-with-justification, and missing states are distinct

  Scenario: missing required evidence blocks release
    Given a required gate with missing evidence
    When release readiness is evaluated
    Then release readiness is blocked

