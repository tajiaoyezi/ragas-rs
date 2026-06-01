# Task 19.1 - metric-catalog-inventory

**Status**: Done
**Phase**: 19
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Upstream ragas exposes collection metrics and legacy metrics across RAG, traditional NLP, rubrics, agents/tools, SQL, multimodal, and summarization. Current Rust code has many metric functions, but not a complete upstream owner inventory with release-blocking status.

## 2. Goal

Implement a machine-readable metric catalog that maps upstream metric families to Rust owners, sample kind, provider requirement, output value type, fixture status, and parity status.

## 3. Scope And Out-of-Scope

**In scope**:
- Metric catalog descriptors for every upstream metric family tracked by the gap analysis.
- Ownership mapping to existing Rust metric functions or explicit KnownGap entries.
- Release-blocking parity claims for unowned or unfixture-backed metrics.

**Out of scope**:
- Implementing each missing metric algorithm in this task.
- Live LLM judge calls in default CI.

## 4. Actors

- Maintainer tracking upstream metric parity.
- Release owner checking full metric catalog readiness.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/ragas-latest-gap-analysis.md
- test/features/metric-catalog-inventory.feature

### 5.2 Imports

Use `src/metrics/`, `src/metric.rs`, and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Metric catalog descriptors list upstream metric families and stable Rust owner names.
- **AC2**: Each descriptor records sample kind, provider requirement, output type, and fixture coverage status.
- **AC3**: Metrics without complete fixture-backed parity create release-blocking claims.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-19.1.1 | TEST-19.1.1 | Done |
| AC2 | SCEN-19.1.2 | TEST-19.1.2 | Done |
| AC3 | SCEN-19.1.3 | TEST-19.1.3 | Done |

## 8. Risks

- Catalog completeness can be confused with implementation completeness.
- Upstream metric names can drift after the baseline hash.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：2026-06-01
- **改动文件**：src/metrics/registry.rs; src/metrics/mod.rs; src/lib.rs
- **commit 列表**：
  - d556364 docs(spec): task-19.1 进入实施
  - ff4fa92 test(metrics): 加 task-19.1 RED 测试
  - fa20a9e feat(metrics): 实现 task-19.1 metric catalog inventory
- **RED 结果**：`cargo test test_19_1` failed as expected with 3 failing 19.1 tests because metric catalog descriptors and release-blocking claims were empty.
- **§9 Verification 结果**：
  - install: `cargo build` passed
  - typecheck: `cargo check` passed
  - unit-test: `cargo test` passed, 148 passed / 0 failed
  - build: `cargo build` passed
- **剩余风险 / 未做项**：Only `context_precision` is fixture-backed complete in this catalog; all other metric families remain Partial or KnownGap release blockers until task 19.2/19.3 add and aggregate fixture evidence.
- **下游 task 影响**：task 19.2 can consume `metric_catalog()` and `metric_catalog_parity_claims()` to enforce fixture-backed parity before any metric is release-ready.
