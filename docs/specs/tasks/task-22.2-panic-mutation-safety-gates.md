# Task 22.2 - panic-mutation-safety-gates

**Status**: Ready
**Phase**: 22
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The "no known bugs" target requires safety-focused evidence, not only happy-path tests. Panic safety and mutation-test policies must be visible release gates.

## 2. Goal

Implement panic-safety and mutation-test gate contracts that block release when required safety evidence is missing or failing.

## 3. Scope And Out-of-Scope

**In scope**:
- Panic-safety evidence descriptors.
- Mutation-test policy descriptors.
- Release blockers for missing or failing safety gates.

**Out of scope**:
- Mandatory mutation testing on every local default run.

## 4. Actors

- QA engineer.
- Release owner.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-22.1-property-fuzz-coverage-gates.md
- test/features/panic-mutation-safety-gates.feature

### 5.2 Imports

Use `src/release/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Panic-safety gates declare scope, command, and failure classes.
- **AC2**: Mutation gates declare tool, threshold, and optional/required status.
- **AC3**: Missing required panic or mutation evidence creates release blockers.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-22.2.1 | TEST-22.2.1 | Not Started |
| AC2 | SCEN-22.2.2 | TEST-22.2.2 | Not Started |
| AC3 | SCEN-22.2.3 | TEST-22.2.3 | Not Started |

## 8. Risks

- Panic checks can miss async task panics if they only inspect direct calls.
- Mutation thresholds can be meaningless without tracked scope.

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
