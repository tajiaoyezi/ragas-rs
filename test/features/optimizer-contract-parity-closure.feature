Feature: optimizer contract parity closure

  Scenario: optimizer descriptors are fixture-backed complete claims
    Given the current upstream optimizer baseline
    When optimizer family descriptors are loaded
    Then DSPy and MIPROv2 are complete fixture-backed claims
    And Python runtime limitations remain explicit

  Scenario: optimizer planning is deterministic and redacted
    Given DSPy cache payloads and MIPROv2 trial settings
    When Rust plans optimizer contracts
    Then cache keys are stable and secrets are redacted
    And trial schedules are deterministic by seed

  Scenario: optimizer blockers are absent from the release ledger
    Given optimizer fixture-backed complete claims
    When the release ledger is summarized
    Then the Optimizer category is absent
    And remaining categories are Quality only
