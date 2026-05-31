# Task 11.2 - semantic

**Status**: Done
**Phase**: 11
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

embedding similarity and thresholded semantic metrics

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
- test/features/semantic.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Semantic similarity uses embedding provider with batching
- **AC2**: Threshold policy is configurable
- **AC3**: Scores are stable for zero vectors

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-11.2.1 | TEST-11.2.1 | Done |
| AC2 | SCEN-11.2.2 | TEST-11.2.2 | Done |
| AC3 | SCEN-11.2.3 | TEST-11.2.3 | Done |

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
  - `docs/specs/tasks/task-11.2-semantic.md`（修改）
- **commit 列表**：
  - `8710a41` test(metrics-traditional): 加 task-11.2 RED 测试
  - `17b12ee` feat(metrics-traditional): 实现 task-11.2 semantic metrics
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 73 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：语义相似度使用 embedding provider 和 cosine；具体 embedding 模型 parity 待 task-16.1 fixture 登记。
- **下游 task 影响**：task-11.3 可复用 traditional result/evidence 模型；task-16.1 需要覆盖 zero-vector 和 threshold policy parity/gap。
