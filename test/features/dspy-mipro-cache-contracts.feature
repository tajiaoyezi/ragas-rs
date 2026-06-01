Feature: dspy mipro cache contracts
  Scenario: optimizer registry lists families
    Given the optimizer registry
    When optimizer families are listed
    Then genetic, DSPy, and MIPROv2 families have status metadata

  Scenario: DSPy cache contract is explicit
    Given a DSPy cache descriptor
    When cache behavior is inspected
    Then deterministic and unsupported Python-runtime behavior are visible

  Scenario: unsupported DSPy parity blocks release
    Given an unsupported DSPy or MIPROv2 behavior
    When parity claims are evaluated
    Then release readiness is blocked
