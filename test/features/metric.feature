# language: en
# Maps to:
#   - docs/specs/tasks/task-2.1-metric-abstractions.md

Feature: metric
  In order to evaluate RAG outputs with custom and built-in rules
  As a Rust caller
  I want type-safe metric values and an async Metric trait

  Scenario: SCEN-2.1.1 metric values expose typed accessors
    Given numeric, discrete, and ranking metric values
    When a caller reads them through helper methods
    Then TEST-2.1.1 returns the expected typed data

  Scenario: SCEN-2.1.2 metric results preserve success and failure details
    Given a metric success and metric failure
    When a caller inspects MetricResult
    Then TEST-2.1.2 sees value, reason, and error fields preserved

  Scenario: SCEN-2.1.3 custom metric scores asynchronously
    Given a closure-backed metric
    When it scores a SingleTurnSample through the Metric trait
    Then TEST-2.1.3 returns the closure result
