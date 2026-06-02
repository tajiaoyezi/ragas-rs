Feature: experiment quickstart closure
  Scenario: experiment quickstart maps to a runnable example
    Given the quickstart docs parity registry
    When the Run experiments quickstart is inspected
    Then it maps to examples/experiment.rs and appears in runnable docs example metadata

  Scenario: experiment quickstart is deterministic
    Given the experiment quickstart example
    When it is built in default example verification
    Then it uses deterministic experiment summary and comparison APIs without live providers

  Scenario: experiment quickstart no longer blocks release
    Given the consolidated release blocker ledger
    When docs parity blockers are inspected
    Then docs::quickstart::experiments is absent from docs release blockers
