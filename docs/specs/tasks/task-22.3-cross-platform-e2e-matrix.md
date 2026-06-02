# Task 22.3 - cross-platform-e2e-matrix

**Status**: Done
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

- [x] **AC1**: Platform matrix includes Linux x64, macOS arm64, and Windows x64 with required evidence status.
- [x] **AC2**: E2E workflow matrix includes evaluate, provider mock, dataset IO, CLI, and docs examples.
- [x] **AC3**: Missing required platform or E2E evidence blocks release.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-22.3.1 | TEST-22.3.1 | Done |
| AC2 | SCEN-22.3.2 | TEST-22.3.2 | Done |
| AC3 | SCEN-22.3.3 | TEST-22.3.3 | Done |

## 8. Risks

- Local green tests can hide platform-specific failures.
- E2E evidence can become stale if commands are not timestamped.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：
  - `src/release/mod.rs`（新增 platform/e2e evidence descriptors、platform/E2E quality gate conversion 与 TEST-22.3.1~22.3.3）
  - `src/lib.rs`（RED 阶段导出 platform/E2E release evidence API）
- **commit 列表**：
  - `78ab97e` docs(spec): task-22.3 Ready gate format
  - `20685ad` docs(spec): task-22.3 进入实施
  - `b9b0db0` test(release): 加 task-22.3 RED 测试
  - `0c24509` feat(release): 实现 task-22.3 platform e2e gates
- **§9 Verification 结果**：
  - Install: passed (`cargo build`)
  - Typecheck: passed (`cargo check`)
  - Unit Test: passed, 181 passed / 0 failed (`cargo test`)
  - Build: passed (`cargo build`)
- **剩余风险 / 未做项**：Local verification remains Windows-only; Linux and macOS entries are required release evidence descriptors that must be satisfied by CI or block final release readiness.
- **下游 task 影响**：Phase 23 release blocker ledger can aggregate platform and E2E gaps through `platform_e2e_quality_gate_descriptors()` and `required_quality_evidence_blockers()`.
