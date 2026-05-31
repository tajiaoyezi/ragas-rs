# Task 1.1 - foundation-dataset

**Status**: Draft
**Phase**: 1 - foundation-dataset
**PRD**: docs/prds/ragas-rs.prd.md

## 1. Background

ragas-rs needs a strongly typed Rust crate foundation before metrics and providers can be built. The PRD requires `EvaluationDataset` and `SingleTurnSample` as core ability 3.

## 2. Goal

Create the crate manifest, public module layout, reusable error type, `SingleTurnSample`, and `EvaluationDataset` with validation.

## 3. Scope And Out-of-Scope

**In scope**:
- Add `Cargo.toml` for a Rust library crate named `ragas`.
- Add `src/lib.rs`, `src/error.rs`, and `src/dataset.rs`.
- Implement `SingleTurnSample` with user input, response, retrieved contexts, optional reference, and metadata.
- Implement `EvaluationDataset` construction, validation, length, empty check, iteration, and indexing helpers.

**Out of scope**:
- Metric trait or built-in metrics.
- Provider HTTP clients.
- Async evaluate orchestration.

## 4. Actors

- Rust caller constructing datasets.
- Metric implementations reading validated samples.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs.prd.md
- docs/decisions/adr-001-trait-layering.md
- test/features/dataset.feature

### 5.2 Imports

Production code exports dataset and error modules from `src/lib.rs`.

### 5.3 Function Signatures

- `SingleTurnSample::new(user_input: impl Into<String>, response: impl Into<String>, retrieved_contexts: Vec<String>) -> Self`
- `SingleTurnSample::with_reference(self, reference: impl Into<String>) -> Self`
- `SingleTurnSample::with_metadata(self, key: impl Into<String>, value: impl Into<String>) -> Self`
- `EvaluationDataset::new(samples: Vec<SingleTurnSample>) -> Result<Self, RagasError>`
- `EvaluationDataset::from_sample(sample: SingleTurnSample) -> Result<Self, RagasError>`
- `EvaluationDataset::len(&self) -> usize`
- `EvaluationDataset::is_empty(&self) -> bool`
- `EvaluationDataset::iter(&self) -> impl Iterator<Item = &SingleTurnSample>`
- `EvaluationDataset::samples(&self) -> &[SingleTurnSample]`

## 6. Acceptance Criteria

- **AC1**: Creating a valid `SingleTurnSample` preserves user input, response, contexts, reference, and metadata.
- **AC2**: `EvaluationDataset::new` accepts non-empty valid samples and exposes len, is_empty, iter, and samples.
- **AC3**: Dataset validation rejects empty datasets and samples with empty user input, response, or retrieved contexts, returning `RagasError::InvalidSample` with the sample index.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-1.1.1 | TEST-1.1.1 | Not Started |
| AC2 | SCEN-1.1.2 | TEST-1.1.2 | Not Started |
| AC3 | SCEN-1.1.3 | TEST-1.1.3 | Not Started |

## 8. Risks

- Rust ownership APIs must remain ergonomic for callers that build datasets from owned strings.
- Validation errors need enough context to debug batch input issues.

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
