# Task 24.2 - disk-cache-persistence-closure

**Status**: In Progress
**Phase**: 24
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The release blocker ledger still treats `backend::disk-cache` as `Partial`. Upstream `src/ragas/cache.py` exposes `DiskCacheBackend` with `get`, `set`, `has_key`, default directory support, and persisted values that survive backend re-instantiation. Upstream `tests/unit/test_cache.py` also verifies cache key separation for different arguments.

## 2. Goal

Close the `backend::disk-cache` release blocker by replacing the in-memory-only compatibility model with a deterministic Rust disk cache that persists values by key and carries fixture-backed parity evidence.

## 3. Scope And Out-of-Scope

**In scope**:
- Persistent local disk cache for byte/JSON-compatible cache values.
- `set`/`get`/`has_key` semantics and sorted key listing.
- Reopening a cache directory preserves previously stored values.
- Backend parity claim for `backend::disk-cache` is `Complete` with fixture metadata and no longer release-blocking.

**Out of scope**:
- Python `diskcache` binary or file-format compatibility.
- Google Drive backend implementation.
- Async cacher decorator parity.

## 4. Actors

- Rust caller using cache-backed evaluation or optimizer flows.
- Release owner inspecting backend release blockers.
- QA engineer checking deterministic cache fixture evidence.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-18.3-backend-registry-diskcache.md
- test/features/disk-cache-persistence-closure.feature
- Upstream baseline files: `src/ragas/cache.py`, `tests/unit/test_cache.py`

### 5.2 Imports

Use `src/backends/`, `src/parity/`, `src/release/`, and deterministic temporary directories under `std::env::temp_dir()`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [x] **AC1**: Disk cache supports `set`, `get`, `has_key`, delete, and sorted key listing for deterministic byte values.
- [x] **AC2**: Reopening the same cache directory preserves values without requiring Python, external services, or process-local memory.
- [x] **AC3**: `backend::disk-cache` is fixture-backed `Complete` and absent from consolidated backend release blockers while `backend::gdrive` remains blocking.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-24.2.1 | TEST-24.2.1 | Spec Ready |
| AC2 | SCEN-24.2.2 | TEST-24.2.2 | Spec Ready |
| AC3 | SCEN-24.2.3 | TEST-24.2.3 | Spec Ready |

## 8. Risks

- Path traversal or unsafe key-to-path mapping could turn a cache API into arbitrary file writes.
- Marking disk-cache complete without fixture metadata would violate the PRD parity rule.
- Closing disk-cache does not address external gdrive backend parity.

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
