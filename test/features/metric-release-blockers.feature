Feature: metric release blockers
  Scenario: metric blockers aggregate catalog and fixture failures
    Given metric catalog and fixture claims
    When release blockers are summarized
    Then partial, missing, and drifted metrics are included

  Scenario: unclassified metric blocks release
    Given a metric name absent from the catalog
    When release blockers are evaluated
    Then the metric blocks release

  Scenario: metric blocker summary is auditable
    Given metric blockers
    When the summary is rendered
    Then blocker count and feature names are visible
