# Task 5.3 - validation

**Status**: Done
**Phase**: 5
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

sample/metric compatibility validator、required column checker

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: src/validation.rs, src/metric.rs, src/eval.rs, src/lib.rs.
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
- test/features/validation.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Validator detects missing fields required by a metric
- **AC2**: Validator reports sample index and field path for invalid records
- **AC3**: Validation can run before evaluate and fail without provider calls

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-5.3.1 | TEST-5.3.1 | Done |
| AC2 | SCEN-5.3.2 | TEST-5.3.2 | Done |
| AC3 | SCEN-5.3.3 | TEST-5.3.3 | Done |

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
  - `src/validation.rs`（新增 metric requirement、ValidationReport、pre-evaluate validation、unit tests）
  - `src/metric.rs`（为 Metric/FnMetric/内置指标暴露 requirements）
  - `src/lib.rs`（导出 validation API）
  - `docs/specs/tasks/task-5.3-validation.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `2633a24` docs(spec): task-5.3 Ready
  - `badcd68` docs(spec): task-5.3 进入实施
  - `334e727` test(validation): 加 task-5.3 RED 测试
  - `21ac782` feat(validation): 实现 task-5.3 预评估校验
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 22 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
- **剩余风险 / 未做项**：当前 validator 覆盖 SingleTurnSample 必填字段和 metric requirement；MultiTurnSample/agent/tool/sql/multimodal 的兼容校验将在 phase 9/12 对应 metric framework 与 advanced metrics 中扩展。
- **下游 task 影响**：phase 6 的 executor/evaluate wrapper 可在 provider 调用前复用 validate_before_evaluate；phase 9 的 metric framework 可扩展 requirements 模型；phase 16 parity suite 可基于 ValidationReport 做失败快照。
