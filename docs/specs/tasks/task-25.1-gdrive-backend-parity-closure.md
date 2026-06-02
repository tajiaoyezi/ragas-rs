# Task 25.1 - gdrive-backend-parity-closure

**Status**: Ready
**Phase**: 25
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The consolidated release blocker ledger still contains `backend::gdrive`. Current upstream `src/ragas/backends/gdrive_backend.py` implements a `GDriveBackend` that stores datasets and experiments as Google Sheets under Drive folders. Default Rust CI cannot depend on Google credentials or network access, so parity must be proven through a deterministic transport abstraction that preserves the same observable sheet-oriented behavior.

## 2. Goal

Close the `backend::gdrive` release blocker by implementing a Rust Google Drive / Sheets backend contract with deterministic in-memory transport tests, upstream configuration metadata, and fixture-backed parity evidence.

## 3. Scope And Out-of-Scope

**In scope**:
- `GDriveBackendConfig` metadata for folder id, auth path environment names, token default, OAuth/service-account modes, and Google Drive/Sheets scopes.
- `GoogleDriveDatasetBackend` backed by a transport abstraction that can save, load, list, and delete datasets as Google Sheets-compatible rows.
- Deterministic fake transport for default CI that verifies headers, JSON-encoded nested values, sorted dataset names, and missing dataset errors.
- `backend::gdrive` parity claim marked `Complete` only with fixture metadata.

**Out of scope**:
- Real Google API HTTP client implementation in default CI.
- Shipping Google credentials, OAuth flows, or token persistence.
- Experiment-specific gdrive rows beyond dataset backend parity.

## 4. Actors

- Rust caller selecting a Google Drive dataset backend.
- Release owner inspecting backend blockers.
- QA engineer comparing deterministic transport behavior with upstream `GDriveBackend`.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-18.3-backend-registry-diskcache.md
- docs/specs/tasks/task-24.2-disk-cache-persistence-closure.md
- test/features/gdrive-backend-parity-closure.feature
- Upstream baseline files: `src/ragas/backends/gdrive_backend.py`, `src/ragas/backends/gdrive_backend.md`

### 5.2 Imports

Use `src/backends/`, `src/parity/`, `src/release/`, and `tests/parity/fixtures/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [x] **AC1**: Rust exposes gdrive config metadata matching upstream folder id, credential environment variables, token default, scopes, and auth modes.
- [x] **AC2**: A deterministic Google Sheets transport backend saves, loads, lists, and deletes datasets while preserving sample fields and nested context/reference metadata.
- [x] **AC3**: `backend::gdrive` is fixture-backed `Complete` and absent from backend release blockers; synthetic unsupported external backends still block release.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-25.1.1 | TEST-25.1.1 | Spec Ready |
| AC2 | SCEN-25.1.2 | TEST-25.1.2 | Spec Ready |
| AC3 | SCEN-25.1.3 | TEST-25.1.3 | Spec Ready |

## 8. Risks

- Deterministic transport parity does not prove live Google API credentials, quotas, or OAuth behavior.
- Header/key ordering must stay deterministic or fixture drift will make release evidence brittle.
- This task closes only the backend gdrive blocker; provider, integration, metric, testset, optimizer, and quality blockers remain.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：待实施后回填
- **改动文件**：待实施后回填
- **commit 列表**：待实施后回填
- **§9 Verification 结果**：待实施后回填
- **剩余风险 / 未做项**：待实施后回填
- **下游 task 影响**：待实施后回填
