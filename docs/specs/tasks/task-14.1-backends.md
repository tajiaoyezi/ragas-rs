# Task 14.1 - backends

**Status**: Done
**Phase**: 14
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

in-memory, JSONL, CSV backend registry

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/backends/`.
- Behavior listed in §6 acceptance criteria.
- Unit tests and, where applicable, parity fixtures for upstream ragas semantics.

**Out of scope**:
- Unrelated phases from the complete refactor matrix.
- Hidden Python runtime dependency or pyo3 bridge.
- Marking parity complete without explicit fixture evidence.

## 4. Actors

- Rust caller using ragas-rs.
- Evaluation framework maintainer tracking Python ragas parity.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-complete-refactor.prd.md
- docs/specs/ragas-complete-refactor-breakdown.md
- test/features/backends.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Backend trait supports save, load, list, and delete
- **AC2**: In-memory backend is deterministic for tests
- **AC3**: JSONL and CSV local backends preserve dataset schema

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-14.1.1 | TEST-14.1.1 | Done |
| AC2 | SCEN-14.1.2 | TEST-14.1.2 | Done |
| AC3 | SCEN-14.1.3 | TEST-14.1.3 | Done |

## 8. Risks

- Upstream Python semantics may not map one-to-one to Rust types.
- Optional external integrations must not leak into the default dependency set.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：2026-05-31
- **改动文件**：
  - `src/backends/mod.rs`（新增 DatasetBackend trait、InMemory/JSONL/CSV backends 与 TEST-14.1.1~14.1.3）
  - `src/lib.rs`（导出 backend public API）
- **commit 列表**：
  - `092b199` docs(spec): task-14.1 Ready
  - `292f3c3` docs(spec): task-14.1 进入实施
  - `3b9ef39` test(backends): 加 task-14.1 RED 测试
  - `4d01e81` feat(backends): 实现 task-14.1 dataset backends
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 97 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：CSV backend 当前只支持 single-turn dataset schema；multi-turn 使用 JSONL backend，后续 CLI/docs 需要显式提示。
- **下游 task 影响**：task 14.3 CLI 可复用 JSONL/CSV backend 读写；task 16.2 docs examples 应说明 backend 格式限制。
