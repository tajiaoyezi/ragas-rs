# Task 21.2 - experiment-sdk-cli-contracts

**Status**: Ready
**Phase**: 21
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Upstream ragas includes experiment tracking, SDK-facing workflows, and CLI commands. Current Rust support covers deterministic experiment summaries and a small CLI harness, but not full upstream workflow contracts.

## 2. Goal

Implement experiment, SDK-facing, and CLI contract descriptors with deterministic workflow tests and release blockers for missing commands or SDK flows.

## 3. Scope And Out-of-Scope

**In scope**:
- Experiment workflow descriptors.
- SDK-facing contract metadata.
- CLI workflow descriptors and deterministic command tests.
- Release blockers for unsupported workflows.

**Out of scope**:
- Hosted SDK services or remote dashboards.

## 4. Actors

- Application developer using Rust workflows.
- Release owner.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-14.3-cli.md
- test/features/experiment-sdk-cli-contracts.feature

### 5.2 Imports

Use `src/experiments/`, `src/cli/`, `src/release/`, and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [ ] **AC1**: Workflow registry lists evaluate, testset, benchmark, experiment, and SDK-facing flows.
- [ ] **AC2**: CLI contract tests preserve stable machine-readable outputs and errors.
- [ ] **AC3**: Missing upstream CLI/SDK workflows create release-blocking claims.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-21.2.1 | TEST-21.2.1 | Not Started |
| AC2 | SCEN-21.2.2 | TEST-21.2.2 | Not Started |
| AC3 | SCEN-21.2.3 | TEST-21.2.3 | Not Started |

## 8. Risks

- CLI snapshots can become brittle if they include unstable paths or timings.
- SDK-facing descriptors can overclaim compatibility without end-to-end evidence.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：<TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
