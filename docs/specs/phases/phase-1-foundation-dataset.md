# Phase 1 - foundation-dataset

**Status**: Draft
**PRD**: docs/prds/ragas-rs.prd.md
**Tasks**: docs/specs/tasks/task-1.1-foundation-dataset.md

## 1. Goal

Create the Rust crate foundation, public module exports, structured errors, `SingleTurnSample`, and `EvaluationDataset` validation.

## 2. Scope

- `Cargo.toml`
- `src/lib.rs`
- `src/error.rs`
- `src/dataset.rs`

## 3. Dependencies

None.

## 4. Risks

- Validation must be strict enough to catch malformed samples but not block optional references.
- Public types need serde support without leaking future provider details.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 1.1 | docs/specs/tasks/task-1.1-foundation-dataset.md | Draft |

## 6. Phase Acceptance And Smoke

- `cargo build` succeeds from repository root.
- `cargo test dataset` succeeds from repository root.
- `docs/s2v-adapter.md` indexes task 1.1 as Done when the phase closes.
