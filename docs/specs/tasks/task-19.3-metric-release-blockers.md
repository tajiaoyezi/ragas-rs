# Task 19.3 - metric-release-blockers

**Status**: Done
**Phase**: 19
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The release model must prevent any metric from being presented as production-ready unless it has owner, implementation, fixture, and verification evidence.

## 2. Goal

Aggregate metric catalog and fixture claims into release-blocking evidence for every partial, missing, unclassified, or drifted metric.

## 3. Scope And Out-of-Scope

**In scope**:
- Metric release blocker aggregation.
- Human-readable blocker summaries for release audits.
- Tests that fail if unclassified metrics are omitted.

**Out of scope**:
- Waiving blockers without Phase 23 waiver policy.

## 4. Actors

- Release owner.
- Metric parity maintainer.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-19.1-metric-catalog-inventory.md
- docs/specs/tasks/task-19.2-metric-golden-fixture-runner.md
- test/features/metric-release-blockers.feature

### 5.2 Imports

Use `src/metrics/`, `src/parity/`, and `src/release/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Metric release blocker aggregation includes catalog, fixture, and drift failures.
- **AC2**: Unclassified metric names are release blockers by default.
- **AC3**: Release summary exposes metric blocker count and feature names.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-19.3.1 | TEST-19.3.1 | Done |
| AC2 | SCEN-19.3.2 | TEST-19.3.2 | Done |
| AC3 | SCEN-19.3.3 | TEST-19.3.3 | Done |

## 8. Risks

- Aggregators can become stale if new descriptors are not included.
- A summary can look green if unclassified entries are ignored.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：2026-06-01
- **改动文件**：src/release/mod.rs; src/lib.rs
- **commit 列表**：
  - b488785 docs(spec): task-19.3 进入实施
  - 28be2d1 test(release): 加 task-19.3 RED 测试
  - 284eb56 feat(release): 实现 task-19.3 metric release blockers
- **RED 结果**：`cargo test test_19_3` failed as expected because metric release blocker aggregation did not include catalog or unclassified blockers.
- **§9 Verification 结果**：
  - install: `cargo build` passed
  - typecheck: `cargo check` passed
  - unit-test: `cargo test` passed, 154 passed / 0 failed
  - build: `cargo build` passed
- **剩余风险 / 未做项**：Most metric families remain release-blocking until golden fixtures are added and drift is resolved; Phase 19 intentionally prevents release claims from hiding those gaps.
- **下游 task 影响**：Later release ledger tasks can consume `metric_release_blockers()` and `summarize_metric_release_blockers()` as one source of release blockers.
