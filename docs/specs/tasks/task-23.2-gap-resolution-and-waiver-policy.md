# Task 23.2 - gap-resolution-and-waiver-policy

**Status**: Ready
**Phase**: 23
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Some gaps may be intentionally excluded from a Rust-native release, but waivers must be visible, scoped, and auditable. Silent gaps are incompatible with the PRD.

## 2. Goal

Implement a gap resolution and waiver policy that distinguishes fixed, blocked, waived, and deferred release blockers.

## 3. Scope And Out-of-Scope

**In scope**:
- Waiver data model.
- Required waiver fields: scope, rationale, owner, expiry, risk, rollback impact.
- Release summary integration.

**Out of scope**:
- Automatically approving waivers.

## 4. Actors

- Release owner.
- Product/engineering approver.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-23.1-release-blocker-ledger.md
- test/features/gap-resolution-and-waiver-policy.feature

### 5.2 Imports

Use `src/release/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Waiver records require scope, rationale, owner, expiry, risk, and rollback impact.
- **AC2**: Expired or incomplete waivers do not unblock release.
- **AC3**: Release summaries show fixed, waived, and still-blocking gaps separately.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-23.2.1 | TEST-23.2.1 | Not Started |
| AC2 | SCEN-23.2.2 | TEST-23.2.2 | Not Started |
| AC3 | SCEN-23.2.3 | TEST-23.2.3 | Not Started |

## 8. Risks

- Waivers can be abused to bypass correctness gaps.
- Expiry checks can drift if dates are stored in ambiguous formats.

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
