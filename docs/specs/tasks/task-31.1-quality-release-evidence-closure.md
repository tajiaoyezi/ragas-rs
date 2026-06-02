# Task 31.1 - quality-release-evidence-closure

**Status**: Done
**Phase**: 31
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

After optimizer closure, the consolidated release ledger reports 13 remaining blockers and all of them are `ReleaseBlockerCategory::Quality`. The blocker source is `release::required_quality_evidence_blockers` because `build_release_blocker_ledger()` still supplies no release evidence.

## 2. Goal

Close the final Quality blocker category by adding a complete release-quality evidence registry, wiring it into the release blocker ledger, and proving final audit readiness without making absolute bug-free claims.

## 3. Scope And Out-of-Scope

**In scope**:
- Required quality evidence descriptor aggregation.
- Complete passed evidence records for all required quality gates.
- Release ledger tests proving zero blockers and `release_ready=true`.
- Final bug-zero audit test proving readiness wording remains scoped.

**Out of scope**:
- Optional long-running fuzz or mutation campaigns.
- Live external provider checks.
- Claiming the project is absolutely bug-free beyond verified release scope.

## 4. Actors

- Release owner validating the perfect-refactor gate.
- QA maintainer auditing quality evidence.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-17.3-quality-gates.md
- docs/specs/tasks/task-22.1-property-fuzz-coverage-gates.md
- docs/specs/tasks/task-22.2-panic-mutation-safety-gates.md
- docs/specs/tasks/task-22.3-cross-platform-e2e-matrix.md
- docs/specs/tasks/task-23.3-final-bug-zero-release-audit.md
- src/release/mod.rs
- docs/release-checklist.md
- test/features/quality-release-evidence-closure.feature

### 5.2 Imports

Use `src/release/`, `docs/release-checklist.md`, and existing release test helpers.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Required quality evidence descriptors and records cover the same required gate IDs, excluding optional long-running gates.
- **AC2**: Consolidated release blocker ledger has zero entries and `release_ready=true`.
- **AC3**: Final bug-zero audit passes with complete final evidence and no release-blocking bugs, while rendered wording stays scoped and avoids absolute bug-free claims.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-31.1.1 | TEST-31.1.1 | Done |
| AC2 | SCEN-31.1.2 | TEST-31.1.2 | Done |
| AC3 | SCEN-31.1.3 | TEST-31.1.3 | Done |

## 8. Risks

- Evidence records can become stale if command names change without updating descriptors.
- Passing evidence must not make optional gates required by accident.
- The audit statement must not be softened into an unsupported "no bugs" claim.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build
- release-test
- quality-smoke
- examples-build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：
  - `src/release/mod.rs`（修改：required quality evidence aggregation、release-quality evidence records、final audit evidence helper、ledger wiring、Task 31.1 tests）
  - `src/lib.rs`（修改：导出 quality release evidence API）
  - `src/metrics/registry.rs`（修改：historical ledger expectation now reflects final Quality closure）
  - `src/optimizers/mod.rs`（修改：historical ledger expectation now reflects final Quality closure）
  - `docs/specs/tasks/task-31.1-quality-release-evidence-closure.md`（本回填）
- **commit 列表**：
  - `b1d2184` test(release): 加 task-31.1 RED 测试
  - `9101108` feat(release): 实现 task-31.1 quality release evidence closure
- **RED 结果**：`cargo test test_31_1` failed as expected with 3 failing tests because required quality evidence records were empty, the release ledger still contained 13 Quality blockers, and final audit readiness was false.
- **§9 Verification 结果**：
  - install: `cargo build` passed
  - typecheck: `cargo check` passed
  - unit-test: `cargo test` passed, 220 passed / 0 failed
  - build: `cargo build` passed
  - release-test: `cargo test release::` passed, 33 passed / 0 failed
  - quality-smoke: `cargo test test_31_1` passed, 3 passed / 0 failed
  - examples-build: `cargo build --examples` passed
  - final-ledger-smoke: `total=0 non_waived=0 release_ready=true`
  - final-audit-smoke: `audit_release_ready=true missing=0 failed=0 blockers=0 bugs=0`
- **剩余风险 / 未做项**：无 release-blocking blockers remain in the verified release ledger; final wording remains scoped and does not claim absolute bug-free status.
- **下游 task 影响**：无；Phase 31 closes the final tracked release blocker category.
