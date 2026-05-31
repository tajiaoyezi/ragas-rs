# Task 9.2 - metric-result

**Status**: Done
**Phase**: 9
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

result schema, score normalization, reason/evidence, error taxonomy

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/metrics/result.rs`, `src/metrics/mod.rs`, and public exports in `src/lib.rs`.
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
- test/features/metric-result.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Metric result stores score, value type, reason, evidence, and error
- **AC2**: Score normalization clamps or rejects invalid numeric scores by policy
- **AC3**: Error taxonomy distinguishes provider, parse, validation, and metric failures

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-9.2.1 | TEST-9.2.1 | Done |
| AC2 | SCEN-9.2.2 | TEST-9.2.2 | Done |
| AC3 | SCEN-9.2.3 | TEST-9.2.3 | Done |

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
  - `src/metrics/result.rs`（新增 detailed result schema、score normalization policy、evidence 和 error taxonomy 及 task 9.2 unit tests）
  - `src/metrics/mod.rs`（导出 result schema）
  - `src/lib.rs`（导出 metric result API）
  - `docs/specs/tasks/task-9.2-metric-result.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `dad9755` docs(spec): task-9.2 Ready
  - `e5a82ed` docs(spec): task-9.2 进入实施
  - `737c755` test(metrics): 加 task-9.2 RED 测试
  - `77c1593` feat(metrics): 实现 task-9.2 metric result
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 55 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
  - note: project helper `s2v_verify_full` still cannot parse lowercase generated §9 keys, so adapter commands were executed directly and recorded here.
- **剩余风险 / 未做项**：本 task 建立新 result schema；旧 MVP `MetricResult` 保持兼容，registry/parity labels 留给 task 9.3。
- **下游 task 影响**：task 9.3 can register metrics with detailed result capabilities and use `MetricErrorKind` for registry-level diagnostics; phase 10+ metrics can attach evidence and explicit normalization policy.
