# Task 4.1 - evaluator-builtins

**Status**: In Progress
**Phase**: 4 - evaluator-builtins
**PRD**: docs/prds/ragas-rs.prd.md

## 1. Background

The PRD requires asynchronous batch `evaluate()` and 2-3 built-in metrics: Faithfulness, ResponseRelevancy, and ContextPrecision.

## 2. Goal

Implement batch evaluator, report aggregation, and the three built-in metrics using provider traits.

## 3. Scope And Out-of-Scope

**In scope**:
- Add `src/eval.rs`.
- Add built-in metrics in `src/metric.rs`.
- Export evaluate and built-in metric types from `src/lib.rs`.
- Isolate per-sample metric errors in the report.
- Support configurable concurrency.

**Out of scope**:
- Exact Python ragas parity.
- Persistent reports or dashboards.
- Benchmark harness for the 5x target.

## 4. Actors

- Rust caller running evaluation over a dataset.
- Metric/provider implementations called by the evaluator.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/specs/tasks/task-1.1-foundation-dataset.md
- docs/specs/tasks/task-2.1-metric-abstractions.md
- docs/specs/tasks/task-3.1-providers.md
- docs/decisions/adr-001-trait-layering.md
- test/features/eval.feature

### 5.2 Imports

Uses dataset, metric, provider traits, tokio semaphore, and futures utilities.

### 5.3 Function Signatures

- `pub async fn evaluate(dataset: &EvaluationDataset, metrics: &[Arc<dyn Metric>], options: EvaluationOptions) -> EvaluationReport`
- `pub struct EvaluationOptions { pub concurrency: usize }`
- `pub struct EvaluationReport { pub results: Vec<SampleEvaluation>, pub metric_names: Vec<String> }`
- `FaithfulnessMetric::new(llm: Arc<dyn LlmProvider>) -> Self`
- `ResponseRelevancyMetric::new(embedding: Arc<dyn EmbeddingProvider>) -> Self`
- `ContextPrecisionMetric::new(embedding: Arc<dyn EmbeddingProvider>) -> Self`

## 6. Acceptance Criteria

- **AC1**: `evaluate()` runs every metric for every sample and keeps metric-level failures as failed `MetricResult` entries.
- **AC2**: `FaithfulnessMetric` parses JSON LLM judgement into a numeric score and reason.
- **AC3**: `ResponseRelevancyMetric` computes cosine similarity between question and response embeddings.
- **AC4**: `ContextPrecisionMetric` computes average precision over retrieved contexts using embedding similarity threshold.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-4.1.1 | TEST-4.1.1 | Not Started |
| AC2 | SCEN-4.1.2 | TEST-4.1.2 | Not Started |
| AC3 | SCEN-4.1.3 | TEST-4.1.3 | Not Started |
| AC4 | SCEN-4.1.4 | TEST-4.1.4 | Not Started |

## 8. Risks

- Built-in metrics are v1.0 heuristic-compatible, not exact Python ragas parity.
- Provider calls can be expensive; concurrency default must be conservative.

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
