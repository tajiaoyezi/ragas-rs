# Task 23.1 - release-blocker-ledger

**Status**: Ready
**Phase**: 23
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Provider, backend, integration, metric, testset, optimizer, docs, and quality gates can each produce release-blocking claims. A release candidate needs one consolidated ledger.

## 2. Goal

Implement a release blocker ledger that aggregates all parity and quality blockers into a single auditable report.

## 3. Scope And Out-of-Scope

**In scope**:
- Aggregation of blocker claims across modules.
- Stable blocker identifiers and severity.
- Summary counts by category and status.

**Out of scope**:
- Waiver mechanics, which belong to task 23.2.

## 4. Actors

- Release owner.
- QA engineer.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-17.4-bug-zero-release-audit.md
- test/features/release-blocker-ledger.feature

### 5.2 Imports

Use `src/release/`, `src/parity/`, and module-level parity claim functions.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Ledger aggregates provider, backend, integration, metric, testset, optimizer, docs, and quality blockers.
- **AC2**: Each blocker has category, feature, severity, source, and release impact.
- **AC3**: Release readiness fails when any non-waived blocker remains.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-23.1.1 | TEST-23.1.1 | Not Started |
| AC2 | SCEN-23.1.2 | TEST-23.1.2 | Not Started |
| AC3 | SCEN-23.1.3 | TEST-23.1.3 | Not Started |

## 8. Risks

- Missing one source registry can produce a false-ready release.
- Severity mapping can understate correctness or safety blockers.

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
