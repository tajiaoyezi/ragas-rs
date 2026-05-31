# Task 6.1 - run-config

**Status**: Done
**Phase**: 6
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

timeout/retry/concurrency/cancellation model

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: src/runtime.rs, src/eval.rs, src/error.rs, src/lib.rs.
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
- test/features/run-config.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: RunConfig stores timeout, retry, concurrency, and cancellation settings
- **AC2**: Defaults are conservative and deterministic
- **AC3**: Invalid config returns structured errors

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-6.1.1 | TEST-6.1.1 | Done |
| AC2 | SCEN-6.1.2 | TEST-6.1.2 | Done |
| AC3 | SCEN-6.1.3 | TEST-6.1.3 | Done |

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
  - `src/runtime.rs`（新增 RunConfig、TimeoutConfig、RetryConfig、CancellationConfig、RunConfigBuilder、RunConfigError 与 unit tests）
  - `src/eval.rs`（新增 EvaluationOptions::from_run_config）
  - `src/lib.rs`（导出 runtime API）
  - `docs/specs/tasks/task-6.1-run-config.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `18dc064` docs(spec): task-6.1 Ready
  - `9a2df21` docs(spec): task-6.1 进入实施
  - `545202d` test(runtime): 加 task-6.1 RED 测试
  - `142fd89` feat(runtime): 实现 task-6.1 run config
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 25 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
- **剩余风险 / 未做项**：本 task 只定义配置模型和结构化校验；实际 timeout/retry/cancellation 执行语义将在 task 6.2 executor 和 task 6.3 callbacks-cost-cache 中落地。
- **下游 task 影响**：task 6.2 可直接用 RunConfig 驱动并发、timeout、retry 和 cancellation；task 7 provider adapters 可复用 retry/timeout 默认值；task 15 benchmarks 可使用 seed/concurrency 做可复现实验。
