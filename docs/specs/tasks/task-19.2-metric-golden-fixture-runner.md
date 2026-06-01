# Task 19.2 - metric-golden-fixture-runner

**Status**: In Progress
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
| AC1 | SCEN-19.2.1 | TEST-19.2.1 | Not Started |
| AC2 | SCEN-19.2.2 | TEST-19.2.2 | Not Started |
| AC3 | SCEN-19.2.3 | TEST-19.2.3 | Not Started |

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

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：<TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
