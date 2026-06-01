Feature: panic mutation safety gates
  Scenario: panic safety gates declare scope
    Given panic safety gate descriptors
    When gates are listed
    Then scope, command, and failure class are present

  Scenario: mutation gates declare thresholds
    Given mutation gate descriptors
    When gates are listed
    Then tool, threshold, and required mode are present

  Scenario: missing required safety evidence blocks release
    Given required panic or mutation evidence is missing
    When release gates are evaluated
    Then release readiness is blocked
