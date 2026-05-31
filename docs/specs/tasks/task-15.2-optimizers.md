# Task 15.2 - optimizers

**Status**: Done
**Phase**: 15
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

prompt/model optimization abstractions and genetic optimizer scaffold

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/optimizers/`.
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
- test/features/optimizers.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Optimizer trait accepts objective metric and candidate generator
- **AC2**: Genetic optimizer scaffold evolves candidates deterministically with seeded RNG
- **AC3**: Optimizer history is inspectable

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-15.2.1 | TEST-15.2.1 | Done |
| AC2 | SCEN-15.2.2 | TEST-15.2.2 | Done |
| AC3 | SCEN-15.2.3 | TEST-15.2.3 | Done |

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
  - `src/optimizers/mod.rs`（新增 optimizer traits、seeded genetic scaffold、history DTO 与 TEST-15.2.1~15.2.3）
  - `src/lib.rs`（导出 optimizers public API）
- **commit 列表**：
  - `c1195cd` docs(spec): task-15.2 Ready
  - `9b4852c` docs(spec): task-15.2 进入实施
  - `553f065` test(optimizers): 加 task-15.2 RED 测试
  - `ef19207` feat(optimizers): 实现 task-15.2 genetic scaffold
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 109 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：遗传优化器当前是 deterministic scaffold，不内置真实 LLM 调参策略或外部追踪；复杂 mutation/selection 策略需在后续优化 task 中扩展。
- **下游 task 影响**：task 15.3 benchmark 可用 OptimizationResult/history 度量优化运行成本；phase 16 docs 需记录 seeded deterministic 行为。
