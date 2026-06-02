Feature: gdrive backend parity closure
  Scenario: GDrive config records upstream authentication contract
    Given the current upstream GDriveBackend baseline
    When the Rust gdrive config metadata is inspected
    Then folder id, credential environment variables, token default, scopes, and auth modes match the upstream contract

  Scenario: deterministic Google Sheets transport roundtrips datasets
    Given a Rust Google Drive backend with fake Sheets transport
    When a dataset is saved, listed, loaded, and deleted
    Then row headers and sample fields roundtrip without a live Google service

  Scenario: gdrive no longer blocks backend release
    Given backend parity claims
    When backend release blockers are inspected
    Then backend::gdrive is complete with fixture metadata while a synthetic unsupported external backend still blocks release
