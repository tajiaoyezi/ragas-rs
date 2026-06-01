# Task 21.3 - quickstart-docs-parity

**Status**: Ready
**Phase**: 21
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The active PRD includes upstream quickstarts, documentation workflows, and examples. Current Rust docs examples are broad but not exhaustively mapped to latest upstream quickstart templates.

## 2. Goal

Implement quickstart and documentation parity descriptors with runnable example coverage and release blockers for missing upstream docs workflows.

## 3. Scope And Out-of-Scope

**In scope**:
- Quickstart descriptor registry.
- Runnable docs example metadata.
- Missing-template release blockers.

**Out of scope**:
- Hosted documentation publishing.

## 4. Actors

- New user following quickstarts.
- Release owner validating docs parity.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-16.2-docs-examples.md
- test/features/quickstart-docs-parity.feature

### 5.2 Imports

Use `src/docs_examples/`, `examples/`, and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Quickstart registry maps upstream quickstart names to Rust examples or known gaps.
- **AC2**: Runnable example metadata includes command, expected output type, and feature flags.
- **AC3**: Missing or non-runnable docs examples create release-blocking claims.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-21.3.1 | TEST-21.3.1 | Not Started |
| AC2 | SCEN-21.3.2 | TEST-21.3.2 | Not Started |
| AC3 | SCEN-21.3.3 | TEST-21.3.3 | Not Started |

## 8. Risks

- Docs parity can drift without executable examples.
- Feature-gated examples can look runnable in docs but fail in default builds.

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
