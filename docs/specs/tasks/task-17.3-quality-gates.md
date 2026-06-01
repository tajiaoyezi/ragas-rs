# Task 17.3 - quality-gates

**Status**: Done
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
| AC1 | SCEN-17.3.1 | TEST-17.3.1 | Done |
| AC2 | SCEN-17.3.2 | TEST-17.3.2 | Done |
| AC3 | SCEN-17.3.3 | TEST-17.3.3 | Done |

## 8. Risks

- Quality gates can become placeholders if not tied to executable checks.
- Coverage/mutation tooling may differ across platforms.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：2026-06-01
- **改动文件**：
  - `src/release/mod.rs`（新增 quality gate kind、evidence status、report summary 和 blocker detection）
  - `src/lib.rs`（导出 release quality gate public API）
- **commit 列表**：
  - `81acc3d` docs(spec): task-17.3 进入实施
  - `8df1e78` test(release): 加 task-17.3 RED 测试
  - `0397952` feat(release): 实现 task-17.3 quality gates
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: ✅ `cargo test` (130 passed / 0 failed)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：当前 blocker 检测覆盖显式 Failed/Missing evidence；后续 task 17.4 需要把 bug ledger 和 release audit 汇总成最终 no-known-bug gate。
- **下游 task 影响**：task 17.4 可把 bug ledger audit 接到 `QualityGateKind::BugLedgerAudit`；phase 22 可把 coverage/fuzz/property/mutation 真实命令结果写入 `QualityGateEvidence`。
