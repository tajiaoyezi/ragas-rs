Feature: gap resolution and waiver policy
  Scenario: waiver requires audit fields
    Given a waiver record
    When it is validated
    Then scope, rationale, owner, expiry, risk, and rollback impact are required

  Scenario: incomplete or expired waiver does not unblock release
    Given an incomplete or expired waiver
    When release blockers are evaluated
    Then release remains blocked

  Scenario: release summary separates fixed waived and blocking gaps
    Given gap resolution records
    When the release summary is rendered
    Then fixed, waived, and still-blocking gaps are separate
