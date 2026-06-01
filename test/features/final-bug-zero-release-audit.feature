Feature: final bug zero release audit
  Scenario: final audit requires all evidence
    Given final release evidence
    When the audit is evaluated
    Then build, check, unit, parity, examples, quality, blocker, and bug-ledger evidence are required

  Scenario: unresolved blocker refuses release
    Given unresolved high severity bugs or unwaived blockers
    When final audit is evaluated
    Then release is refused

  Scenario: final wording states evidence scope
    Given a final audit summary
    When it is rendered
    Then it avoids unsupported absolute bug-free claims
