# Task 2.1 - metric-abstractions

**Status**: In Progress
**Phase**: 2 - metric-abstractions
**PRD**: docs/prds/ragas-rs.prd.md

## 1. Background

The PRD requires Metric abstraction for Discrete, Numeric, Ranking, and custom metrics. Downstream evaluate and built-ins depend on this stable contract.

## 2. Goal

Define `MetricValue`, `MetricResult`, async `Metric` trait, and a `FnMetric` helper for custom metrics.

## 3. Scope And Out-of-Scope

**In scope**:
- Add `src/metric.rs`.
- Export metric types from `src/lib.rs`.
- Implement numeric/discrete/ranking value helpers.
- Implement async `Metric` trait and closure-backed custom metric helper.

**Out of scope**:
- Provider integration.
- Built-in Faithfulness, ResponseRelevancy, ContextPrecision logic.
- Batch evaluation orchestration.

## 4. Actors

- Library consumers implementing custom metrics.
- Built-in metrics implemented in later tasks.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/specs/tasks/task-1.1-foundation-dataset.md
- docs/decisions/adr-001-trait-layering.md
- test/features/metric.feature

### 5.2 Imports

Uses `SingleTurnSample` and `RagasError` from phase 1.

### 5.3 Function Signatures

- `pub enum MetricValue { Discrete(String), Numeric(f64), Ranking(Vec<RankingItem>) }`
- `pub struct RankingItem { pub item: String, pub score: f64 }`
- `pub struct MetricResult { pub metric_name: String, pub value: Option<MetricValue>, pub reason: Option<String>, pub error: Option<String> }`
- `#[async_trait] pub trait Metric: Send + Sync { fn name(&self) -> &str; async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError>; }`
- `FnMetric::new(name: impl Into<String>, scorer: F) -> Self`

## 6. Acceptance Criteria

- **AC1**: `MetricValue` exposes constructors/accessors for numeric, discrete, and ranking values.
- **AC2**: `MetricResult::success` and `MetricResult::failure` preserve metric name, value, reason, and error.
- **AC3**: A closure-backed custom metric can asynchronously score a `SingleTurnSample` through the `Metric` trait.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-2.1.1 | TEST-2.1.1 | Not Started |
| AC2 | SCEN-2.1.2 | TEST-2.1.2 | Not Started |
| AC3 | SCEN-2.1.3 | TEST-2.1.3 | Not Started |

## 8. Risks

- Async closure ergonomics can become hard for callers if bounds are too strict.
- Metric result errors must not conflict with task-level evaluation errors.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：待实施
- **改动文件**：待实施
- **commit 列表**：待实施
- **§9 Verification 结果**：待实施
- **剩余风险 / 未做项**：待实施
- **下游 task 影响**：待实施
