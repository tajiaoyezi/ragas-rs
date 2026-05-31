# Task 12.2 - agents-tools

**Status**: Done
**Phase**: 12
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

goal accuracy, tool call accuracy, tool call F1, topic adherence

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/metrics/advanced/`.
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
- test/features/agents-tools.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Tool call metrics compare names, args, and order policy
- **AC2**: Agent goal accuracy supports multi-turn traces
- **AC3**: Topic adherence records per-topic evidence

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-12.2.1 | TEST-12.2.1 | Done |
| AC2 | SCEN-12.2.2 | TEST-12.2.2 | Done |
| AC3 | SCEN-12.2.3 | TEST-12.2.3 | Done |

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
  - `src/metrics/advanced/mod.rs`（新增 tool call、agent goal、topic adherence 指标 API 与 TEST-12.2.1~12.2.3）
  - `src/metrics/mod.rs`（导出 task-12.2 advanced metrics API）
  - `src/lib.rs`（导出 task-12.2 public crate API）
  - `src/eval.rs`, `src/llm.rs`, `src/metric.rs`, `src/metrics/base.rs`, `src/metrics/rag/mod.rs`, `src/metrics/registry.rs`, `src/metrics/result.rs`, `src/metrics/traditional/mod.rs`, `src/prompts/mod.rs`, `src/providers.rs`, `src/runtime.rs`, `src/validation.rs`（cargo fmt-only refactor）
- **commit 列表**：
  - `4bf1a7c` docs(spec): task-12.2 Ready
  - `e8e1f1e` docs(spec): task-12.2 进入实施
  - `5a6abfa` test(metrics-advanced): 加 task-12.2 RED 测试
  - `3b6567a` feat(metrics-advanced): 实现 task-12.2 agents tools
  - `a11583c` refactor(style): cargo fmt source tree
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 82 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：Python ragas 的 LLM judge prompt parity 未在本 task 内声称完成；本 task 交付确定性 aggregation contract 和可审计 evidence，后续由 parity suite 补 golden fixtures。
- **下游 task 影响**：task 12.3 可复用 advanced metrics result/evidence 模式；task 16.1 需要为 agent/tool/topic metrics 登记 parity fixtures 或 Known Gap。
