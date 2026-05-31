# Task 11.3 - quoted-spans

**Status**: Done
**Phase**: 11
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

quoted spans and citation overlap metrics

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: src/metrics/traditional/.
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
- test/features/quoted-spans.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Quoted span extraction preserves byte and char ranges
- **AC2**: Overlap scoring handles partial matches
- **AC3**: Missing citations produce explicit zero-score reason

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-11.3.1 | TEST-11.3.1 | Done |
| AC2 | SCEN-11.3.2 | TEST-11.3.2 | Done |
| AC3 | SCEN-11.3.3 | TEST-11.3.3 | Done |

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
  - `src/metrics/traditional/mod.rs`（修改）
  - `src/metrics/mod.rs`（修改）
  - `src/lib.rs`（修改）
  - `docs/specs/tasks/task-11.3-quoted-spans.md`（修改）
- **commit 列表**：
  - `8794a8e` test(metrics-traditional): 加 task-11.3 RED 测试
  - `d95e703` feat(metrics-traditional): 实现 task-11.3 quoted spans
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 76 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：quoted span/citation overlap 为 deterministic span policy；Python ragas parity fixture 待 task-16.1 登记。
- **下游 task 影响**：Phase 11 可进入完成 gate；task-16.1 需要登记 quoted span/citation overlap fixture 或 Known Gap。
