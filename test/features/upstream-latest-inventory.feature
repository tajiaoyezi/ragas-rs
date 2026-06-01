Feature: upstream latest inventory
  Scenario: latest upstream baseline is recorded
    Given the upstream main and latest release hashes
    When the parity inventory is loaded
    Then both hashes are visible to release checks

  Scenario: upstream source categories are classified
    Given the upstream source category list
    When the inventory summary is computed
    Then every category has a parity status

  Scenario: incomplete category blocks release
    Given an inventory containing incomplete categories
    When release readiness is evaluated
    Then release readiness is blocked

