# Task 31.1 - quality-release-evidence-closure

**Status**: Ready
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
| AC1 | SCEN-31.1.1 | TEST-31.1.1 | Spec Ready |
| AC2 | SCEN-31.1.2 | TEST-31.1.2 | Spec Ready |
| AC3 | SCEN-31.1.3 | TEST-31.1.3 | Spec Ready |

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

- **完成日期**：待实施后回填
- **改动文件**：待实施后回填
- **commit 列表**：待实施后回填
- **RED 结果**：待实施后回填
- **§9 Verification 结果**：待实施后回填
- **剩余风险 / 未做项**：待实施后回填
- **下游 task 影响**：待实施后回填
