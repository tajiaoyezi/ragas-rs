# Task 22.3 - cross-platform-e2e-matrix

**Status**: Ready
**Phase**: 22
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The Rust crate must support Linux x64, macOS arm64, and Windows x64. Current local verification is Windows-only in this workspace and lacks a cross-platform/E2E evidence matrix.

## 2. Goal

Implement cross-platform and E2E evidence descriptors that make unsupported platform or workflow evidence release-blocking.

## 3. Scope And Out-of-Scope

**In scope**:
- Platform matrix descriptors.
- E2E workflow evidence descriptors.
- Release blockers for missing required platform or E2E evidence.

**Out of scope**:
- Running remote CI from the local default command.

## 4. Actors

- Release owner.
- CI maintainer.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/s2v-adapter.md
- test/features/cross-platform-e2e-matrix.feature

### 5.2 Imports

Use `src/release/` and docs release evidence.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [ ] **AC1**: Platform matrix includes Linux x64, macOS arm64, and Windows x64 with required evidence status.
- [ ] **AC2**: E2E workflow matrix includes evaluate, provider mock, dataset IO, CLI, and docs examples.
- [ ] **AC3**: Missing required platform or E2E evidence blocks release.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-22.3.1 | TEST-22.3.1 | Not Started |
| AC2 | SCEN-22.3.2 | TEST-22.3.2 | Not Started |
| AC3 | SCEN-22.3.3 | TEST-22.3.3 | Not Started |

## 8. Risks

- Local green tests can hide platform-specific failures.
- E2E evidence can become stale if commands are not timestamped.

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
