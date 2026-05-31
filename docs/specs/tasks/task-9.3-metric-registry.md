# Task 9.3 - metric-registry

**Status**: Done
**Phase**: 9
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

metric collection registry, feature flags, parity status labels

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/metrics/registry.rs`, `src/metrics/mod.rs`, and public exports in `src/lib.rs`.
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
- test/features/metric-registry.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Metric registry resolves built-ins by stable names
- **AC2**: Feature-gated metrics are hidden unless enabled
- **AC3**: Parity status labels are exported for docs and tests

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-9.3.1 | TEST-9.3.1 | Done |
| AC2 | SCEN-9.3.2 | TEST-9.3.2 | Done |
| AC3 | SCEN-9.3.3 | TEST-9.3.3 | Done |

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
  - `src/metrics/registry.rs`（新增 metric registry、feature gating、parity status labels 和 task 9.3 unit tests）
  - `src/metrics/mod.rs`（导出 registry API）
  - `src/lib.rs`（导出 metric registry API）
  - `docs/specs/tasks/task-9.3-metric-registry.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `eb39b7b` docs(spec): task-9.3 Ready
  - `a765fba` docs(spec): task-9.3 进入实施
  - `852d55d` test(metrics): 加 task-9.3 RED 测试
  - `f5cccf3` feat(metrics): 实现 task-9.3 metric registry
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 58 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
  - note: project helper `s2v_verify_full` still cannot parse lowercase generated §9 keys, so adapter commands were executed directly and recorded here.
- **剩余风险 / 未做项**：本 task 只实现 registry metadata/gating，不实例化 all future metric implementations; built-in concrete registration will expand in phase 10+ as metrics are migrated.
- **下游 task 影响**：phase 10/11/12 can register migrated metrics by stable name, optional feature and parity label; phase 16 docs/tests can render parity labels from `ParityStatus`.
