# Task 19.1 - metric-catalog-inventory

**Status**: Ready
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
| AC1 | SCEN-19.1.1 | TEST-19.1.1 | Not Started |
| AC2 | SCEN-19.1.2 | TEST-19.1.2 | Not Started |
| AC3 | SCEN-19.1.3 | TEST-19.1.3 | Not Started |

## 8. Risks

- Catalog completeness can be confused with implementation completeness.
- Upstream metric names can drift after the baseline hash.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：<TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
