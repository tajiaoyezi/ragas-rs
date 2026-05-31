# Task 12.3 - sql-multimodal-summary

**Status**: Done
**Phase**: 12
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

SQL semantic equivalence, multimodal faithfulness/relevance, summarization

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
- test/features/sql-multimodal-summary.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: SQL semantic equivalence compares normalized SQL or judge output
- **AC2**: Multimodal metrics route through multimodal prompt model
- **AC3**: Summarization score parses coverage and conciseness signals

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-12.3.1 | TEST-12.3.1 | Done |
| AC2 | SCEN-12.3.2 | TEST-12.3.2 | Done |
| AC3 | SCEN-12.3.3 | TEST-12.3.3 | Done |

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
  - `src/metrics/advanced/mod.rs`（新增 SQL semantic equivalence、multimodal metric、summarization signal parser 与 TEST-12.3.1~12.3.3）
  - `src/metrics/mod.rs`（导出 task-12.3 advanced metrics API）
  - `src/lib.rs`（导出 task-12.3 public crate API）
- **commit 列表**：
  - `96c75a3` docs(spec): task-12.3 Ready
  - `dcc9ff3` docs(spec): task-12.3 进入实施
  - `bd0f225` test(metrics-advanced): 加 task-12.3 RED 测试
  - `a1ec15f` feat(metrics-advanced): 实现 task-12.3 sql multimodal summary
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 85 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：SQL 本地规范化是轻量 deterministic fallback，不是完整 SQL AST 等价；未声称 Python ragas prompt/judge parity complete，后续由 parity suite 登记 fixture 或 Known Gap。
- **下游 task 影响**：phase 16.1 需要覆盖 SQL/multimodal/summarization golden fixtures；docs/examples 可展示多模态 prompt scaffold 和 summarization judge JSON 格式。
