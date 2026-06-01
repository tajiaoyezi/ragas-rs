# Task 19.2 - metric-golden-fixture-runner

**Status**: Done
**Phase**: 19
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Existing parity fixtures cover only a small subset of metric behavior. Full upstream parity requires a deterministic fixture runner that compares Rust outputs against Python baselines with explicit tolerances and drift reporting.

## 2. Goal

Implement metric golden fixture loading and comparison contracts so parity-complete metric claims require executable fixture evidence.

## 3. Scope And Out-of-Scope

**In scope**:
- Metric fixture metadata and parsing.
- Deterministic comparison for numeric, discrete, ranking, and structured evidence outputs.
- Drift diagnostics suitable for release blockers.

**Out of scope**:
- Generating Python baselines during default Rust CI.
- Live provider calls.

## 4. Actors

- Metric maintainer adding golden fixtures.
- CI/release owner validating parity claims.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-17.2-parity-fixture-policy.md
- test/features/metric-golden-fixture-runner.feature

### 5.2 Imports

Use `src/parity/`, `src/metrics/`, and `tests/parity/fixtures/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Metric golden fixtures load baseline output, Rust output, tolerance, and upstream source metadata.
- **AC2**: Fixture comparison distinguishes exact match, tolerated numeric drift, known gap, and undeclared drift.
- **AC3**: `ParityComplete` metric claims without fixture metadata fail validation.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-19.2.1 | TEST-19.2.1 | Done |
| AC2 | SCEN-19.2.2 | TEST-19.2.2 | Done |
| AC3 | SCEN-19.2.3 | TEST-19.2.3 | Done |

## 8. Risks

- Fixture schemas can become too narrow for non-numeric metric outputs.
- Tolerances can hide semantic regressions if not explicit.

## 9. Verification Plan

- install
- typecheck
- unit-test
- parity-test
- build

## 10. Completion Notes

- **完成日期**：2026-06-01
- **改动文件**：src/parity/mod.rs; src/lib.rs
- **commit 列表**：
  - ba7e607 docs(spec): task-19.2 进入实施
  - 43dd230 test(parity): 加 task-19.2 RED 测试
  - 80c8ec6 feat(parity): 实现 task-19.2 metric golden fixture runner
- **RED 结果**：`cargo test test_19_2` failed as expected with 3 failing 19.2 tests because metric fixture parsing, drift classification, and fixture-required complete-claim validation were not implemented.
- **§9 Verification 结果**：
  - install: `cargo build` passed
  - typecheck: `cargo check` passed
  - unit-test: `cargo test` passed, 151 passed / 0 failed
  - parity-test: `cargo test parity::` passed, 12 passed / 0 failed
  - build: `cargo build` passed
- **剩余风险 / 未做项**：The runner compares deterministic fixture payloads; generating or refreshing Python baselines remains outside default CI and must be handled by later fixture-authoring work.
- **下游 task 影响**：task 19.3 can aggregate metric catalog blockers with `validate_metric_golden_claim()` and `compare_metric_golden_fixture()` evidence.
