# Task 23.2 - gap-resolution-and-waiver-policy

**Status**: Done
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

- [x] **AC1**: Waiver records require scope, rationale, owner, expiry, risk, and rollback impact.
- [x] **AC2**: Expired or incomplete waivers do not unblock release.
- [x] **AC3**: Release summaries show fixed, waived, and still-blocking gaps separately.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-23.2.1 | TEST-23.2.1 | Done |
| AC2 | SCEN-23.2.2 | TEST-23.2.2 | Done |
| AC3 | SCEN-23.2.3 | TEST-23.2.3 | Done |

## 8. Risks

- Waivers can be abused to bypass correctness gaps.
- Expiry checks can drift if dates are stored in ambiguous formats.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：
  - `src/release/mod.rs`（新增 release waiver、gap resolution record/summary、waiver validation 与 TEST-23.2.1~23.2.3）
  - `src/lib.rs`（RED 阶段导出 waiver/gap resolution API）
- **commit 列表**：
  - `0479097` docs(spec): task-23.2 Ready gate format
  - `07b1cff` docs(spec): task-23.2 进入实施
  - `e79e2d9` test(release): 加 task-23.2 RED 测试
  - `0b1d66a` feat(release): 实现 task-23.2 waiver policy
- **§9 Verification 结果**：
  - Install: passed (`cargo build`)
  - Typecheck: passed (`cargo check`)
  - Unit Test: passed, 187 passed / 0 failed (`cargo test`)
  - Build: passed (`cargo build`)
- **剩余风险 / 未做项**：Waiver policy validates auditability and expiry, but task 23.3 still must run the final audit and refuse release when non-waived blockers remain.
- **下游 task 影响**：task 23.3 can combine `build_release_blocker_ledger()`, `summarize_gap_resolutions()`, and final verification evidence to produce the release-candidate bug-zero audit.
