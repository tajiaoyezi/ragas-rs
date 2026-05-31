# Task 9.1 - metric-base

**Status**: Done
**Phase**: 9
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

full metric traits: single/multi-turn, LLM/embedding requirements, batch hooks

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/metrics/base.rs`, `src/metrics/mod.rs`, and public exports in `src/lib.rs`.
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
- test/features/metric-base.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Metric traits distinguish single-turn, multi-turn, LLM, and embedding requirements
- **AC2**: Batch scoring hooks default to per-sample behavior
- **AC3**: Metric metadata declares required sample fields

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-9.1.1 | TEST-9.1.1 | Done |
| AC2 | SCEN-9.1.2 | TEST-9.1.2 | Done |
| AC3 | SCEN-9.1.3 | TEST-9.1.3 | Done |

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
  - `src/metrics/base.rs`（新增 `MetricMetadata`、single/multi-turn metric traits、provider requirements、batch hook defaults 和 task 9.1 unit tests）
  - `src/metrics/mod.rs`（新增 metrics module boundary）
  - `src/lib.rs`（导出 metric base API）
  - `docs/specs/tasks/task-9.1-metric-base.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `dc0786c` docs(spec): task-9.1 Ready
  - `2bd5c9d` docs(spec): task-9.1 进入实施
  - `6832822` test(metrics): 加 task-9.1 RED 测试
  - `37e5f99` feat(metrics): 实现 task-9.1 metric base
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 52 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
  - note: project helper `s2v_verify_full` still cannot parse lowercase generated §9 keys, so adapter commands were executed directly and recorded here.
- **剩余风险 / 未做项**：本 task 建立新 `src/metrics` base layer；旧 `src/metric.rs` MVP API 保持兼容，result schema enrichment 和 registry 留给 task 9.2/9.3。
- **下游 task 影响**：task 9.2 can build richer result/error taxonomy on `MetricMetadata`; task 9.3 can register metrics by sample kind, provider requirements, and required fields.
