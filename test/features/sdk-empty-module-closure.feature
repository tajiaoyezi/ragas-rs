Feature: sdk empty module closure
  Scenario: SDK contract records empty upstream module
    Given the current upstream src/ragas/sdk.py baseline
    When the Rust SDK module contract is inspected
    Then it records a zero-byte upstream module and does not block release

  Scenario: SDK workflow claim is fixture backed complete
    Given workflow parity claims
    When workflow::sdk_facing is inspected
    Then it is complete with fixture metadata

  Scenario: synthetic missing workflow still blocks release
    Given a synthetic missing workflow claim
    When workflow release blockers are inspected
    Then the synthetic missing workflow blocks release while workflow::sdk_facing does not
