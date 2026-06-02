Feature: metric fixture parity closure

  Scenario: metric catalog descriptors are fixture-backed complete
    Given the current upstream metric catalog baseline
    When metric catalog descriptors are loaded
    Then every tracked metric family is complete
    And every descriptor has fixture metadata rooted in src/ragas/metrics

  Scenario: metric golden fixtures parse and compare
    Given every fixture path declared by the metric catalog
    When the Rust fixture runner parses and compares those fixtures
    Then every comparison is exact or within declared tolerance
    And no undeclared drift is accepted

  Scenario: metric blockers are absent from the release ledger
    Given the release blocker ledger after provider, backend, and integration closure
    When the ledger is summarized
    Then the Metric category is absent
    And remaining categories are Testset, Optimizer, and Quality
