# Task 17.2 - parity-fixture-policy

**Status**: In Progress
**Phase**: 17
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Current parity evidence is too small: one tracked fixture cannot support a full upstream rewrite claim.

## 2. Goal

Define and implement a fixture policy that prevents `ParityComplete` claims unless the relevant upstream feature has golden Python baseline data and Rust output evidence.

## 3. Scope And Out-of-Scope

**In scope**:
- `src/parity/` fixture policy types.
- Tests for missing fixture evidence, tolerance policy, and release-blocking labels.
- Fixture naming and storage conventions under `tests/parity/fixtures/`.

**Out of scope**:
- Writing every future metric/provider/testset fixture in this task.
- Live API calls in default CI.

## 4. Actors

- Maintainer adding parity fixtures.
- Release owner verifying claims.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/ragas-latest-gap-analysis.md
- test/features/parity-fixture-policy.feature

### 5.2 Imports

Extend the existing `parity` module.

### 5.3 Function Signatures

RED tests own concrete signatures.

## 6. Acceptance Criteria

- **AC1**: `ParityComplete` validation requires at least one fixture reference.
- **AC2**: Fixture metadata records upstream module path, upstream test source when available, deterministic/mock/live mode, and tolerance policy.
- **AC3**: Known gaps and partial items are allowed during development but release-blocking by default.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-17.2.1 | TEST-17.2.1 | Not Started |
| AC2 | SCEN-17.2.2 | TEST-17.2.2 | Not Started |
| AC3 | SCEN-17.2.3 | TEST-17.2.3 | Not Started |

## 8. Risks

- Fixture count can grow quickly; structure must remain searchable.
- Live provider behavior must not destabilize deterministic CI.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build
- extra: `cargo test parity::`

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：<TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
