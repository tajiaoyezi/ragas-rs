# Task 17.4 - bug-zero-release-audit

**Status**: In Progress
**Phase**: 17
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The active goal requires no known unresolved bugs. The repository needs a release-blocking bug and risk ledger instead of relying on passing tests alone.

## 2. Goal

Create a no-known-bug audit model that records open defects, severity, affected upstream feature, regression coverage, and release-blocking status.

## 3. Scope And Out-of-Scope

**In scope**:
- Bug ledger structure and tests.
- Release-readiness policy for unresolved defects.
- Documentation in the release checklist.

**Out of scope**:
- Claiming absolute absence of latent defects.
- Waiving critical bugs without explicit release-blocking rationale.

## 4. Actors

- Release owner.
- Maintainer triaging parity and implementation defects.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/release-checklist.md
- test/features/bug-zero-release-audit.feature

### 5.2 Imports

Use `src/release/`.

### 5.3 Function Signatures

RED tests own concrete signatures.

## 6. Acceptance Criteria

- **AC1**: Bug ledger entries record id, severity, status, affected feature, evidence, and regression test reference.
- **AC2**: Any unresolved critical/high correctness, safety, data-loss, panic, security, or parity bug blocks release.
- **AC3**: Release audit output lists zero unresolved release-blocking bugs before reporting readiness.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-17.4.1 | TEST-17.4.1 | Not Started |
| AC2 | SCEN-17.4.2 | TEST-17.4.2 | Not Started |
| AC3 | SCEN-17.4.3 | TEST-17.4.3 | Not Started |

## 8. Risks

- A ledger without enforced release checks can become documentation only.
- Severity classification needs conservative defaults.

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
