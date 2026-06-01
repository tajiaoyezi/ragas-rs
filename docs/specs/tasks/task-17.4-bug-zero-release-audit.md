# Task 17.4 - bug-zero-release-audit

**Status**: Done
**Phase**: 17
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The active goal requires no known unresolved bugs. The repository needs a release-blocking bug and risk ledger instead of relying on passing tests alone.

## 2. Goal

Create a no-known-bug audit model that records open defects, severity, affected upstream feature, regression coverage, and release-blocking status.

## 3. Scope And Out-of-Scope

**In scope**:
- Bug ledger structure and tests.
- Release-readiness policy for unresolved defects.
- Documentation in the release checklist.

**Out of scope**:
- Claiming absolute absence of latent defects.
- Waiving critical bugs without explicit release-blocking rationale.

## 4. Actors

- Release owner.
- Maintainer triaging parity and implementation defects.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/release-checklist.md
- test/features/bug-zero-release-audit.feature

### 5.2 Imports

Use `src/release/`.

### 5.3 Function Signatures

RED tests own concrete signatures.

## 6. Acceptance Criteria

- **AC1**: Bug ledger entries record id, severity, status, affected feature, evidence, and regression test reference.
- **AC2**: Any unresolved critical/high correctness, safety, data-loss, panic, security, or parity bug blocks release.
- **AC3**: Release audit output lists zero unresolved release-blocking bugs before reporting readiness.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-17.4.1 | TEST-17.4.1 | Done |
| AC2 | SCEN-17.4.2 | TEST-17.4.2 | Done |
| AC3 | SCEN-17.4.3 | TEST-17.4.3 | Done |

## 8. Risks

- A ledger without enforced release checks can become documentation only.
- Severity classification needs conservative defaults.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：2026-06-01
- **改动文件**：
  - `src/release/mod.rs`（新增 bug ledger、severity/status/class、release blocker 和 bug-zero audit summary）
  - `src/lib.rs`（导出 bug-zero audit public API）
  - `docs/release-checklist.md`（补充 No-known-bug audit release gate）
- **commit 列表**：
  - `7bbfacd` docs(spec): task-17.4 进入实施
  - `b9c30e1` test(release): 加 task-17.4 RED 测试
  - `8a2edf9` feat(release): 实现 task-17.4 bug zero audit
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: ✅ `cargo test` (133 passed / 0 failed)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：bug-zero audit 现在能阻断显式 ledger entries；后续 phase 18-23 仍必须把发现的 parity/provider/testset/metric 缺陷写入 ledger，而不是只保留在文档叙述里。
- **下游 task 影响**：phase 18-23 的实现 task 发现 correctness/security/parity defect 时必须登记 bug ledger，并在 release gate 前解决或保持阻断。
