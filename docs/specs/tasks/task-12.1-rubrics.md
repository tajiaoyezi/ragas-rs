# Task 12.1 - rubrics

**Status**: Done
**Phase**: 12
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

aspect critic, simple criteria, domain/instance rubrics

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: src/metrics/advanced/.
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
- test/features/rubrics.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Rubric metrics accept typed criteria
- **AC2**: Aspect critic returns binary or graded result according to config
- **AC3**: Domain and instance rubrics serialize for audit

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-12.1.1 | TEST-12.1.1 | Done |
| AC2 | SCEN-12.1.2 | TEST-12.1.2 | Done |
| AC3 | SCEN-12.1.3 | TEST-12.1.3 | Done |

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
  - `src/metrics/advanced/mod.rs`（新增）
  - `src/metrics/mod.rs`（修改）
  - `src/lib.rs`（修改）
  - `docs/specs/tasks/task-12.1-rubrics.md`（修改）
- **commit 列表**：
  - `8bfa9aa` test(metrics-advanced): 加 task-12.1 RED 测试
  - `0c5c45e` feat(metrics-advanced): 实现 task-12.1 rubrics
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 79 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：aspect critic/rubric 数据模型已落地；Python ragas prompt parity 和 golden fixtures 待 task-16.1 登记。
- **下游 task 影响**：task-12.2、task-12.3 可复用 `src/metrics/advanced/` 和 audit serialization 模式。
