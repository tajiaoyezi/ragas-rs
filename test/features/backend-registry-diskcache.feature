Feature: backend registry diskcache
  Scenario: backend registry lists upstream families
    Given the backend registry
    When backends are listed
    Then in-memory, local JSONL, local CSV, disk-cache, and gdrive families appear

  Scenario: disk cache compatibility preserves key value semantics
    Given a disk-cache compatibility model
    When values are stored and read by key
    Then deterministic key value behavior is preserved

  Scenario: unsupported external backend blocks release
    Given an unsupported external backend
    When parity claims are evaluated
    Then release readiness is blocked

