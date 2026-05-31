# Task 6.3 - callbacks-cost-cache

**Status**: Done
**Phase**: 6
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

callbacks、token usage/cost model、cache key/value abstraction

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: src/runtime.rs, src/llm.rs, src/lib.rs.
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
- test/features/callbacks-cost-cache.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Callback hooks receive evaluation lifecycle events
- **AC2**: Token usage aggregates per provider and metric
- **AC3**: Cache key derivation is stable and redacts secrets

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-6.3.1 | TEST-6.3.1 | Done |
| AC2 | SCEN-6.3.2 | TEST-6.3.2 | Done |
| AC3 | SCEN-6.3.3 | TEST-6.3.3 | Done |

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
  - `src/runtime.rs`（新增 CallbackManager/RuntimeEvent、UsageTracker/UsageSummary、CacheKey stable redaction）
  - `src/lib.rs`（导出 callbacks/cost/cache runtime API）
  - `docs/specs/tasks/task-6.3-callbacks-cost-cache.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `0f1a7d7` docs(spec): task-6.3 Ready
  - `fd095e6` docs(spec): task-6.3 进入实施
  - `10610ac` test(runtime): 加 task-6.3 RED 测试
  - `2aadead` feat(runtime): 实现 task-6.3 callbacks cost cache
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 31 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
- **剩余风险 / 未做项**：本 task 提供 runtime 抽象层；实际 provider/evaluate 深度接线、token price table、持久化 cache backend 由 provider、integration、backend phases 继续扩展。
- **下游 task 影响**：Phase 7 provider adapters 可记录 usage 并复用 CacheKey；Phase 14 integrations/CLI 可订阅 RuntimeEvent；Phase 15 benchmarks 可复用 UsageSummary 做成本汇总。
