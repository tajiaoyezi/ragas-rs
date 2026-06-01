# Task 17.1 - upstream-latest-inventory

**Status**: In Progress
**Phase**: 17
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The prior complete-refactor pass targeted upstream commit `298b682` structurally, but release-quality parity now needs an explicit feature inventory covering both upstream main and latest release tag `v0.4.3`.

## 2. Goal

Create a Rust-readable upstream baseline and feature inventory that can classify every upstream category as `ParityComplete`, `Partial`, `KnownGap`, `NotStarted`, or `Blocked`.

## 3. Scope And Out-of-Scope

**In scope**:
- `src/parity/` inventory types and default latest-baseline data.
- Tests proving the baseline hashes and category coverage.
- Gap-analysis documentation updates if implementation reveals missing categories.

**Out of scope**:
- Implementing the feature gaps themselves.
- Claiming `ParityComplete` for categories without fixture evidence.

## 4. Actors

- Evaluation framework maintainer tracking Python ragas parity.
- Release owner checking whether the Rust refactor can replace upstream behavior.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/ragas-latest-gap-analysis.md
- test/features/upstream-latest-inventory.feature

### 5.2 Imports

Use the existing `parity` public module and expose new types through `src/lib.rs`.

### 5.3 Function Signatures

RED tests own the final signatures, but the task must provide a stable query for latest upstream baseline metadata and feature inventory entries.

## 6. Acceptance Criteria

- **AC1**: Latest upstream baseline records both `main` commit `298b68274234c060deacab3cf5fb52aa3a20e885` and release `v0.4.3` commit `4ecab384fda829ca50bec3f07cc49589d756e172`.
- **AC2**: Inventory covers the upstream categories `top-level`, `backends`, `embeddings`, `integrations`, `llms`, `metrics`, `optimizers`, `prompt`, and `testset`.
- **AC3**: Inventory summary fails release readiness when any category is `NotStarted`, `Partial`, `KnownGap`, or `Blocked`.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-17.1.1 | TEST-17.1.1 | Not Started |
| AC2 | SCEN-17.1.2 | TEST-17.1.2 | Not Started |
| AC3 | SCEN-17.1.3 | TEST-17.1.3 | Not Started |

## 8. Risks

- The latest release tag and main branch can diverge; both must remain visible.
- A broad category inventory is not enough for final parity; later tasks must break categories down into feature fixtures.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build
- extra: `cargo test parity::`

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：<TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
