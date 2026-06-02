# Task 23.1 - release-blocker-ledger

**Status**: Done
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

- [x] **AC1**: Ledger aggregates provider, backend, integration, metric, testset, optimizer, docs, and quality blockers.
- [x] **AC2**: Each blocker has category, feature, severity, source, and release impact.
- [x] **AC3**: Release readiness fails when any non-waived blocker remains.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-23.1.1 | TEST-23.1.1 | Done |
| AC2 | SCEN-23.1.2 | TEST-23.1.2 | Done |
| AC3 | SCEN-23.1.3 | TEST-23.1.3 | Done |

## 8. Risks

- Missing one source registry can produce a false-ready release.
- Severity mapping can understate correctness or safety blockers.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：
  - `src/release/mod.rs`（新增 release blocker ledger entry/category/summary，聚合 provider/backend/integration/metric/testset/optimizer/docs/quality blockers 与 TEST-23.1.1~23.1.3）
  - `src/lib.rs`（RED 阶段导出 release blocker ledger API）
- **commit 列表**：
  - `c83ed21` docs(spec): task-23.1 Ready gate format
  - `ffb5745` docs(spec): task-23.1 进入实施
  - `71702ab` test(release): 加 task-23.1 RED 测试
  - `39213ae` feat(release): 实现 task-23.1 blocker ledger
- **§9 Verification 结果**：
  - Install: passed (`cargo build`)
  - Typecheck: passed (`cargo check`)
  - Unit Test: passed, 184 passed / 0 failed (`cargo test`)
  - Build: passed (`cargo build`)
- **剩余风险 / 未做项**：Waiver mechanics are intentionally out of scope for task 23.1 and remain in task 23.2; current ledger can show release-blocking gaps but does not resolve them.
- **下游 task 影响**：task 23.2 can attach scoped waivers to `ReleaseBlockerEntry`; task 23.3 can use `summarize_release_blocker_ledger()` to refuse final release when non-waived blockers remain.
