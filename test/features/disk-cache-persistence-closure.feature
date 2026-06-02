Feature: disk cache persistence closure
  Scenario: disk cache exposes upstream key value semantics
    Given a Rust disk cache directory
    When values are set, read, checked, listed, and deleted
    Then deterministic key value behavior matches upstream DiskCacheBackend expectations

  Scenario: disk cache persists across instances
    Given a value stored in a disk cache directory
    When a new cache instance opens the same directory
    Then the value is still available without process local memory

  Scenario: disk cache no longer blocks release
    Given backend parity claims
    When backend release blockers are inspected
    Then backend::disk-cache is complete with fixture metadata and backend::gdrive remains blocking
