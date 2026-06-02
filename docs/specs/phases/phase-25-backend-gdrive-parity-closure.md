# Phase 25 - backend-gdrive-parity-closure

**Status**: Ready
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 24

## 1. Goal

Close the remaining backend release blocker by implementing current-upstream Google Drive / Google Sheets backend behavior through a Rust-native transport boundary with deterministic default tests and fixture-backed parity evidence.

## 2. Scope

The phase covers `backend::gdrive` only. It must model upstream `GDriveBackend` configuration, dataset spreadsheet save/load/list/delete behavior, and release-blocking parity claims without making default CI depend on Google credentials or network access.

## 3. Dependencies

Phase 24 release-blocker closures, task 18.3 backend registry contracts, task 24.2 fixture-backed backend blocker policy, and upstream `src/ragas/backends/gdrive_backend.py`.

## 4. Risks

- Overclaiming real Google API compatibility without live credentials would weaken release evidence.
- Google Sheets row serialization can lose nested sample fields if JSON encoding/decoding is incomplete.
- Default deterministic tests must not hide the need for an opt-in live transport before external-service release claims are broadened.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 25.1 | docs/specs/tasks/task-25.1-gdrive-backend-parity-closure.md | Ready |

## 6. Phase Acceptance And Smoke

- `backend::gdrive` is not release-blocking after task completion.
- The Rust implementation has deterministic transport tests for Google Sheets-compatible dataset save/load/list/delete behavior.
- `cargo build`, `cargo check`, `cargo test`, `cargo test parity::`, and `cargo build --examples` pass from the repository root.
