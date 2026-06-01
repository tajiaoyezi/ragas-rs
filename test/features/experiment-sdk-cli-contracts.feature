Feature: experiment sdk cli contracts
  Scenario: workflow registry lists upstream flows
    Given the workflow registry
    When workflows are listed
    Then evaluate, testset, benchmark, experiment, and SDK-facing flows appear

  Scenario: CLI outputs are stable
    Given a CLI workflow contract
    When the command is executed deterministically
    Then output and error schemas are stable

  Scenario: missing CLI or SDK workflow blocks release
    Given an unsupported upstream workflow
    When parity claims are evaluated
    Then release readiness is blocked
