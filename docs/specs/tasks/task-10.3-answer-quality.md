# Task 10.3 - answer-quality

**Status**: Done
**Phase**: 10
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

answer relevancy/correctness/similarity/noise sensitivity

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
- test/features/answer-quality.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Answer relevancy supports embedding and LLM judge paths
- **AC2**: Answer correctness combines semantic and factual signals
- **AC3**: Noise sensitivity returns interpretable numeric score

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-10.3.1 | TEST-10.3.1 | Done |
| AC2 | SCEN-10.3.2 | TEST-10.3.2 | Done |
| AC3 | SCEN-10.3.3 | TEST-10.3.3 | Done |

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
  - `docs/specs/tasks/task-10.3-answer-quality.md`（修改）
- **commit 列表**：
  - `4c7c726` test(metrics-rag): 加 task-10.3 RED 测试
  - `052e8ef` feat(metrics-rag): 实现 task-10.3 answer quality
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 67 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：answer relevancy/correctness/noise sensitivity 已提供 Rust deterministic/judge-output API；Python ragas golden parity 待 task-16.1 登记。
- **下游 task 影响**：Phase 10 可进入完成 gate；task-16.1 需要登记 answer-quality parity fixture 或 Known Gap。
