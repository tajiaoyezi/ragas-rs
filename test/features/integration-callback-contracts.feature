Feature: integration callback contracts
  Scenario: integration registry lists upstream families
    Given the integration registry
    When integration families are listed
    Then every upstream integration family has a status

  Scenario: callback payloads are normalized and redacted
    Given a callback payload containing secrets
    When the payload is exported
    Then secrets are redacted and lifecycle fields remain

  Scenario: unsupported integration blocks release
    Given an unsupported upstream integration
    When parity claims are evaluated
    Then release readiness is blocked

