Feature: cross platform e2e matrix
  Scenario: platform matrix covers supported targets
    Given the platform evidence matrix
    When supported targets are listed
    Then Linux x64, macOS arm64, and Windows x64 are present

  Scenario: e2e workflow matrix covers critical flows
    Given the E2E matrix
    When workflows are listed
    Then evaluate, provider mock, dataset IO, CLI, and docs examples are present

  Scenario: missing platform or e2e evidence blocks release
    Given required platform or E2E evidence is missing
    When release gates are evaluated
    Then release readiness is blocked
