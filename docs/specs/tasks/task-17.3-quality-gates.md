# Task 17.3 - quality-gates

**Status**: Ready
**Phase**: 17
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

`cargo test` is necessary but insufficient for the requested no-known-bug release standard.

## 2. Goal

Define and implement a quality gate model that tracks required unit, integration, parity, fuzz/property, coverage, mutation, and cross-platform evidence.

## 3. Scope And Out-of-Scope

**In scope**:
- Release/quality gate data structures.
- Documentation for deterministic default gates and opt-in live gates.
- Tests proving missing required evidence blocks release.

**Out of scope**:
- Installing heavyweight optional tools globally.
- Requiring live provider credentials in default CI.

## 4. Actors

- QA/release owner.
- Maintainer extending the test matrix.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/release-checklist.md
- test/features/quality-gates.feature

### 5.2 Imports

Use `src/release/` and existing crate exports.

### 5.3 Function Signatures

RED tests own concrete signatures.

## 6. Acceptance Criteria

- **AC1**: Required gate types include build, typecheck, unit, integration, parity, examples, coverage, fuzz/property, and bug-ledger audit.
- **AC2**: A release gate report distinguishes passed, failed, skipped-with-justification, and missing evidence.
- **AC3**: Missing evidence for a required gate blocks release readiness.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-17.3.1 | TEST-17.3.1 | Not Started |
| AC2 | SCEN-17.3.2 | TEST-17.3.2 | Not Started |
| AC3 | SCEN-17.3.3 | TEST-17.3.3 | Not Started |

## 8. Risks

- Quality gates can become placeholders if not tied to executable checks.
- Coverage/mutation tooling may differ across platforms.

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

