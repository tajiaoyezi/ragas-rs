# Task 22.2 - panic-mutation-safety-gates

**Status**: Done
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

- [x] **AC1**: Panic-safety gates declare scope, command, and failure classes.
- [x] **AC2**: Mutation gates declare tool, threshold, and optional/required status.
- [x] **AC3**: Missing required panic or mutation evidence creates release blockers.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-22.2.1 | TEST-22.2.1 | Done |
| AC2 | SCEN-22.2.2 | TEST-22.2.2 | Done |
| AC3 | SCEN-22.2.3 | TEST-22.2.3 | Done |

## 8. Risks

- Panic checks can miss async task panics if they only inspect direct calls.
- Mutation thresholds can be meaningless without tracked scope.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：
  - `src/release/mod.rs`（新增 panic safety/mutation descriptor、failure class、required release evidence threshold 与 TEST-22.2.1~22.2.3）
  - `src/lib.rs`（RED 阶段导出新增 safety gate API）
- **commit 列表**：
  - `6ebe59f` docs(spec): task-22.2 Ready gate format
  - `b2f1b55` docs(spec): task-22.2 进入实施
  - `2aa19ac` test(release): 加 task-22.2 RED 测试
  - `97eb67f` feat(release): 实现 task-22.2 safety gates
- **§9 Verification 结果**：
  - Install: passed (`cargo build`)
  - Typecheck: passed (`cargo check`)
  - Unit Test: passed, 178 passed / 0 failed (`cargo test`)
  - Build: passed (`cargo build`)
- **剩余风险 / 未做项**：Mutation release threshold is represented as required release evidence, but cargo-mutants remains outside default deterministic CI; task 22.3 must add platform and E2E evidence before Phase 22 can close.
- **下游 task 影响**：task 22.3 and phase 23 release ledger can aggregate panic and mutation blockers through `panic_mutation_quality_gate_descriptors()` and `required_quality_evidence_blockers()`.
