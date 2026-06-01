Feature: metric golden fixture runner
  Scenario: metric golden fixture loads metadata
    Given a metric parity fixture
    When the fixture is parsed
    Then baseline, Rust output, tolerance, and upstream source metadata are available

  Scenario: metric golden comparison detects drift
    Given metric fixture outputs
    When outputs are compared
    Then exact match, tolerated drift, known gap, and undeclared drift are distinct

  Scenario: parity complete requires metric fixtures
    Given a parity complete metric claim
    When fixture metadata is missing
    Then validation fails
