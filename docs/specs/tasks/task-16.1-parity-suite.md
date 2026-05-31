# Task 16.1 - parity-suite

**Status**: Done
**Phase**: 16
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

upstream golden fixtures, gap matrix, parity status reports

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/parity/` and `tests/parity/`.
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
- test/features/parity-suite.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Parity fixture format stores Python baseline and Rust output
- **AC2**: Gap matrix lists Complete, Partial, and Known Gap per feature
- **AC3**: Parity tests fail on undeclared semantic drift

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-16.1.1 | TEST-16.1.1 | Done |
| AC2 | SCEN-16.1.2 | TEST-16.1.2 | Done |
| AC3 | SCEN-16.1.3 | TEST-16.1.3 | Done |

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
  - `src/parity/mod.rs`（新增 parity fixture、gap matrix、semantic drift checker 与 TEST-16.1.1~16.1.3）
  - `src/lib.rs`（导出 parity public API）
  - `tests/parity/fixtures/context_precision.json`（tracked golden fixture）
- **commit 列表**：
  - `cff13da` docs(spec): task-16.1 Ready
  - `50d9aa9` docs(spec): task-16.1 进入实施
  - `37041a3` test(parity): 加 task-16.1 RED 测试
  - `beebba7` feat(parity): 实现 task-16.1 parity checks
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 115 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：当前 drift checker 对 `score` 字段做 tolerance 比较，并对其他 JSON 做整体比较；复杂 metric-specific diff 需要后续 fixture policy 扩展。
- **下游 task 影响**：task 16.2 docs 可引用 parity fixture 格式；task 16.3 release 可把 gap matrix 与 undeclared drift policy 写入发布检查。
