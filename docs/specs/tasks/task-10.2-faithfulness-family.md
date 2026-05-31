# Task 10.2 - faithfulness-family

**Status**: Done
**Phase**: 10
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

faithfulness, response groundedness, factual correctness

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: src/metrics/rag/.
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
- test/features/faithfulness-family.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Faithfulness uses prompt/parser contract from phase 8
- **AC2**: Response groundedness records supporting context evidence
- **AC3**: Factual correctness handles TP/FP/FN style output

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-10.2.1 | TEST-10.2.1 | Done |
| AC2 | SCEN-10.2.2 | TEST-10.2.2 | Done |
| AC3 | SCEN-10.2.3 | TEST-10.2.3 | Done |

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
  - `src/metrics/rag/mod.rs`（修改）
  - `src/metrics/mod.rs`（修改）
  - `src/lib.rs`（修改）
  - `docs/specs/tasks/task-10.2-faithfulness-family.md`（修改）
- **commit 列表**：
  - `0eec028` test(metrics-rag): 加 task-10.2 RED 测试
  - `d007fa1` feat(metrics-rag): 实现 task-10.2 faithfulness family
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 64 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：faithfulness prompt/parser 契约已接入 phase 8；groundedness/factual correctness 当前为 deterministic scoring，Python ragas golden parity 待 task-16.1 登记。
- **下游 task 影响**：task-10.3 可复用 `FaithfulnessJudgeContract`、`response_groundedness` 和 `factual_correctness` 的 evidence/result 模型；task-16.1 需要补 parity fixture 或 Known Gap。
