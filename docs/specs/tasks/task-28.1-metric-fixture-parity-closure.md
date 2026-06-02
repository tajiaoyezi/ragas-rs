# Task 28.1 - metric-fixture-parity-closure

**Status**: Done
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
| AC1 | SCEN-28.1.1 | TEST-28.1.1 | Done |
| AC2 | SCEN-28.1.2 | TEST-28.1.2 | Done |
| AC3 | SCEN-28.1.3 | TEST-28.1.3 | Done |

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

- **完成日期**：2026-06-02
- **改动文件**：`src/metrics/registry.rs`; `src/metrics/mod.rs`; `src/lib.rs`; `src/release/mod.rs`; `tests/parity/fixtures/metric_*.json`; `docs/specs/tasks/task-28.1-metric-fixture-parity-closure.md`
- **commit 列表**：
  - `cbaf77a docs(spec): add task-28.1 metric fixture parity closure`
  - `80909f1 docs(spec): task-28.1 进入实施`
  - `c9f3080 test(metrics): 加 task-28.1 RED 测试`
  - `4f141c6 feat(metrics): 实现 task-28.1 metric fixture parity closure`
- **RED 结果**：`cargo test test_28_1` failed as expected with 3 tests discovered, 0 passed, 3 failed. The failures proved fixture metadata was empty, metric fixture validation covered 0 of 26 descriptors, and the release ledger still contained Metric blockers.
- **§9 Verification 结果**：
  - Install: `cargo build` passed.
  - Typecheck: `cargo check` passed.
  - Unit Test: `cargo test` passed with 211 passed, 0 failed.
  - Build: `cargo build` passed.
  - Metrics Test: `cargo test metrics::` passed with 42 passed, 0 failed.
  - Parity Test: `cargo test parity::` passed with 12 passed, 0 failed.
  - Examples Build: `cargo build --examples` passed.
- **剩余风险 / 未做项**：Metric default CI now proves deterministic golden fixture contracts for all tracked metric families at upstream baseline `298b68274234c060deacab3cf5fb52aa3a20e885`; it still does not claim live LLM, embedding, SQL engine, or multimodal provider execution against external services.
- **下游 task 影响**：Metric release blockers dropped from 25 to 0; consolidated ledger moved from 45 to 20 non-waived blockers and now contains only Testset, Optimizer, and Quality categories.
