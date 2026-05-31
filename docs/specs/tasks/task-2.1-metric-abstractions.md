# Task 2.1 - metric-abstractions

**Status**: Done
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
| AC1 | SCEN-2.1.1 | TEST-2.1.1 | Done |
| AC2 | SCEN-2.1.2 | TEST-2.1.2 | Done |
| AC3 | SCEN-2.1.3 | TEST-2.1.3 | Done |

## 8. Risks

- Async closure ergonomics can become hard for callers if bounds are too strict.
- Metric result errors must not conflict with task-level evaluation errors.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：2026-05-31
- **改动文件**：
  - `Cargo.toml`（新增 async trait/runtime 依赖）
  - `Cargo.lock`（更新锁文件）
  - `src/lib.rs`（导出 metric API）
  - `src/metric.rs`（新增 MetricValue、MetricResult、Metric、FnMetric 与单元测试）
- **commit 列表**：
  - `ef0d27d` docs(spec): task-2.1 Ready
  - `d1c82a7` docs(spec): task-2.1 进入实施
  - `933e394` test(metric): 加 task-2.1 RED 测试
  - `7d12630` feat(metric): 实现 task-2.1 指标抽象
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 6 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
- **剩余风险 / 未做项**：无
- **下游 task 影响**：task-3.1 可独立定义 provider trait；task-4.1 可基于 `Metric` 和 `MetricResult` 实现内置指标
