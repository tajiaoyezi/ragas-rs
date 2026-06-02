# Task 28.1 - metric-fixture-parity-closure

**Status**: Ready
**Phase**: 28
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The current release ledger is green for provider, backend, integration, docs, workflow, and SDK categories, but still reports 25 metric release blockers. Existing Rust metric algorithms cover the families, while Phase 19 intentionally kept most catalog entries Partial or KnownGap until golden fixture evidence was attached.

## 2. Goal

Close the metric release-blocker category with fixture-backed complete parity claims for every tracked metric family, using deterministic Rust fixture validation and upstream-current module metadata.

## 3. Scope And Out-of-Scope

**In scope**:
- Fixture-backed metadata for every tracked metric catalog descriptor.
- Catalog updates so metric parity claims return no blockers only when descriptors are `Complete` and fixture-backed.
- RED/GREEN tests that fail before fixture-backed completion and prove release ledger metric blockers drop to zero.
- JSON parity fixtures under `tests/parity/fixtures/metric_*.json`.

**Out of scope**:
- Running live upstream Python metric execution during default CI.
- Live LLM, embedding, multimodal, or database service calls.
- Waiving metric blockers without evidence.

## 4. Actors

- Maintainer validating upstream metric parity.
- Release owner checking that metric blockers cannot be hidden without fixture evidence.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-19.1-metric-catalog-inventory.md
- docs/specs/tasks/task-19.2-metric-golden-fixture-runner.md
- docs/specs/tasks/task-19.3-metric-release-blockers.md
- src/metrics/registry.rs
- src/parity/mod.rs
- test/features/metric-fixture-parity-closure.feature

### 5.2 Imports

Use `src/metrics/`, `src/parity/`, `src/release/`, and `tests/parity/fixtures/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Metric catalog descriptors for all tracked families are `Complete`, fixture-backed, and include non-empty fixture metadata with upstream module paths rooted in `src/ragas/metrics/`.
- **AC2**: Metric golden fixtures for all tracked families parse, validate, and compare without undeclared drift.
- **AC3**: Release blocker ledger contains no `Metric` category entries while preserving remaining Testset, Optimizer, and Quality blockers.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-28.1.1 | TEST-28.1.1 | Spec Ready |
| AC2 | SCEN-28.1.2 | TEST-28.1.2 | Spec Ready |
| AC3 | SCEN-28.1.3 | TEST-28.1.3 | Spec Ready |

## 8. Risks

- Deterministic fixtures can prove contract stability but not live provider quality for LLM-judged metrics.
- Fixture JSON values can overfit if tests only count files; tests must parse and compare every registered fixture.
- Upstream metric additions after baseline `298b68274234c060deacab3cf5fb52aa3a20e885` remain future work and must re-enter the ledger.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build
- metrics-test
- parity-test
- examples-build

## 10. Completion Notes

- **完成日期**：待实施后回填
- **改动文件**：待实施后回填
- **commit 列表**：待实施后回填
- **RED 结果**：待实施后回填
- **§9 Verification 结果**：待实施后回填
- **剩余风险 / 未做项**：待实施后回填
- **下游 task 影响**：待实施后回填
