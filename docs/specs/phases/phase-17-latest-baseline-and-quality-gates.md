# Phase 17 - latest-baseline-and-quality-gates

**Status**: Ready
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 16

## 1. Goal

Freeze the current upstream baseline, convert informal parity gaps into a tracked inventory, and establish stronger quality gates for the perfect refactor objective.

## 2. Scope

`src/parity/`, `src/release/`, `docs/specs/`, `tests/parity/`, and quality gate docs.

## 3. Dependencies

Phase 16 parity/docs/release groundwork.

## 4. Risks

- Upstream may move after the baseline is frozen.
- Existing "KnownGap" labels may hide release-blocking work if not made machine-readable.
- Testing gates can become performative unless tied to upstream feature coverage.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 17.1 | docs/specs/tasks/task-17.1-upstream-latest-inventory.md | Ready |
| 17.2 | docs/specs/tasks/task-17.2-parity-fixture-policy.md | Ready |
| 17.3 | docs/specs/tasks/task-17.3-quality-gates.md | Ready |
| 17.4 | docs/specs/tasks/task-17.4-bug-zero-release-audit.md | Ready |

## 6. Phase Acceptance And Smoke

- Upstream main and latest release hashes are stored in Rust-readable structures and docs.
- Every upstream source category is represented in a parity inventory.
- Release gates fail when an inventory item remains unclassified or release-blocking.
- `cargo build`, `cargo check`, `cargo test`, and `cargo test parity::` pass from the repository root.

