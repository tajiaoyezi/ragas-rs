# Task 6.2 - executor

**Status**: Done
**Phase**: 6
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

ordered async executor、partial failure isolation、progress events

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: src/runtime.rs, src/error.rs, src/lib.rs, src/eval.rs.
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
- test/features/executor.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Executor preserves output order for concurrent tasks
- **AC2**: Executor records partial failures without aborting unrelated work
- **AC3**: Progress events are emitted for start, success, and failure

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-6.2.1 | TEST-6.2.1 | Done |
| AC2 | SCEN-6.2.2 | TEST-6.2.2 | Done |
| AC3 | SCEN-6.2.3 | TEST-6.2.3 | Done |

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
  - `src/runtime.rs`（新增 AsyncExecutor、ExecutorReport、ExecutorOutcome、ProgressEvent，并实现并发执行、顺序回填、失败隔离）
  - `src/lib.rs`（导出 executor API）
  - `docs/specs/tasks/task-6.2-executor.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `844271b` docs(spec): task-6.2 Ready
  - `583c309` docs(spec): task-6.2 进入实施
  - `c561db5` test(runtime): 加 task-6.2 RED 测试
  - `043da05` feat(runtime): 实现 task-6.2 async executor
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 28 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
- **剩余风险 / 未做项**：本 task 只实现 executor 核心并发、顺序和 progress event；retry/timeout/cancellation 与 callbacks/cost/cache 的具体集成将在 task 6.3 和后续 provider/runtime tasks 扩展。
- **下游 task 影响**：task 6.3 可在 AsyncExecutor progress events 上接 callbacks/cost/cache；task 7 provider adapters 可由 AsyncExecutor 批量调度；task 14 CLI 可复用 ExecutorReport 做执行摘要。
