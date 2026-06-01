# Task 18.3 - backend-registry-diskcache

**Status**: Done
**Phase**: 18
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Upstream has in-memory, local CSV/JSONL, registry, gdrive, and disk cache related backend behavior. Current Rust backends cover deterministic in-memory and string-backed local formats only.

## 2. Goal

Implement backend registry and disk-cache compatibility metadata that clearly distinguishes implemented deterministic backends from unsupported live/external backends.

## 3. Scope And Out-of-Scope

**In scope**:
- Backend registry descriptors.
- Disk-cache compatibility model for key/value persistence semantics.
- Release-blocking parity claim generation for unsupported external backends such as gdrive.

**Out of scope**:
- Real Google Drive API calls in default CI.
- Python diskcache binary compatibility.

## 4. Actors

- Rust caller selecting dataset/cache backend.
- Release owner tracking backend parity.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/ragas-latest-gap-analysis.md
- test/features/backend-registry-diskcache.feature

### 5.2 Imports

Use `src/backends/`, `src/runtime.rs`, and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Backend registry lists in-memory, local JSONL, local CSV, disk-cache, and gdrive families with implementation status.
- **AC2**: Disk-cache compatibility model preserves deterministic key/value semantics without Python diskcache dependency.
- **AC3**: External/unimplemented backend families block release unless explicitly implemented with fixtures.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-18.3.1 | TEST-18.3.1 | Done |
| AC2 | SCEN-18.3.2 | TEST-18.3.2 | Done |
| AC3 | SCEN-18.3.3 | TEST-18.3.3 | Done |

## 8. Risks

- Registry metadata can become stale unless tied to release blockers.
- Disk cache semantics can be overclaimed without persistence fixtures.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：2026-06-01
- **改动文件**：src/backends/mod.rs; src/lib.rs
- **commit 列表**：
  - 122df09 docs(spec): task-18.3 进入实施
  - a93153c test(backends): 加 task-18.3 RED 测试
  - dc92f27 feat(backends): 实现 task-18.3 backend registry contracts
- **RED 结果**：`cargo test test_18_3` failed as expected with 3 failing 18.3 tests because the backend registry, disk-cache key/value behavior, and gdrive blocker were missing.
- **§9 Verification 结果**：
  - install: `cargo build` passed
  - typecheck: `cargo check` passed
  - unit-test: `cargo test` passed, 142 passed / 0 failed
  - build: `cargo build` passed
- **剩余风险 / 未做项**：Disk-cache is a deterministic Rust compatibility model, not Python diskcache binary compatibility; gdrive remains unsupported and release-blocking until implemented with fixtures or explicitly excluded.
- **下游 task 影响**：Release readiness can consume `backend_parity_claims()`; task 18.4 can proceed to integration/callback contracts without changing backend storage traits.
