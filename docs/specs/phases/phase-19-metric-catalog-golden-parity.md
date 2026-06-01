# Phase 19 - metric-catalog-golden-parity

**Status**: Done
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 18

## 1. Goal

Drive the upstream metric catalog from broad Rust approximations to explicit owner descriptors, golden fixture coverage, and release-blocking parity claims for every missing or unproven metric behavior.

## 2. Scope

`src/metrics/`, `src/metric.rs`, `src/parity/`, `tests/parity/fixtures/`, and metric-related release evidence.

## 3. Dependencies

Phase 17 fixture policy and Phase 18 provider/backend/runtime parity contracts.

## 4. Risks

- Upstream metric names, prompt contracts, and output parsers can drift faster than Rust fixtures.
- LLM-judged metrics need deterministic captured outputs to avoid flaky default CI.
- A metric can be structurally implemented while still failing semantic parity.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 19.1 | docs/specs/tasks/task-19.1-metric-catalog-inventory.md | Done |
| 19.2 | docs/specs/tasks/task-19.2-metric-golden-fixture-runner.md | Done |
| 19.3 | docs/specs/tasks/task-19.3-metric-release-blockers.md | Done |

## 6. Phase Acceptance And Smoke

- Every upstream metric family has a Rust owner descriptor and implementation status.
- Golden fixture metadata exists for parity-complete metric claims.
- Missing, partial, or unclassified metrics block release.
- `cargo build`, `cargo check`, `cargo test`, and `cargo test parity::` pass from the repository root.
