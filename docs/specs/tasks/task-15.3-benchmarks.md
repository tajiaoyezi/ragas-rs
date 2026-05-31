# Task 15.3 - benchmarks

**Status**: Done
**Phase**: 15
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

LLM/embedding benchmark runner and cost summaries

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/benchmarks/`.
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
- test/features/benchmarks.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Benchmark runner executes providers over fixed prompts
- **AC2**: Cost summary aggregates usage and configured rates
- **AC3**: Benchmark output is stable JSON

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-15.3.1 | TEST-15.3.1 | Done |
| AC2 | SCEN-15.3.2 | TEST-15.3.2 | Done |
| AC3 | SCEN-15.3.3 | TEST-15.3.3 | Done |

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
  - `src/benchmarks/mod.rs`（新增 provider benchmark runner、cost summary、stable JSON DTO 与 TEST-15.3.1~15.3.3）
  - `src/lib.rs`（导出 benchmarks public API）
- **commit 列表**：
  - `a4d9e98` docs(spec): task-15.3 Ready
  - `6298c78` docs(spec): task-15.3 进入实施
  - `a191554` test(benchmarks): 加 task-15.3 RED 测试
  - `a44469a` feat(benchmarks): 实现 task-15.3 provider runner
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 112 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：benchmark runner 当前使用 provider 返回的 TokenUsage 聚合成本，不主动计时或采集 p50/p95 latency；真实性能基准可在 release/CI 扩展。
- **下游 task 影响**：phase 16 parity/release 可复用 BenchmarkReport stable JSON 作为性能与成本输出格式。
