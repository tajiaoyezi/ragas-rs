# Phase 31 - quality-release-evidence-closure

**Status**: Ready
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 30

## 1. Goal

Close the final Quality release-blocker category by registering complete deterministic release evidence for required quality gates and proving the consolidated release ledger reaches zero blockers.

## 2. Scope

The phase covers release-quality evidence records for property, fuzz smoke, coverage summary, panic safety, mutation threshold, platform matrix, and E2E workflow gates. It preserves the existing missing-evidence blocker behavior for callers that provide incomplete evidence.

## 3. Dependencies

Phase 17 quality gate model, Phase 22 quality descriptors, Phase 23 final audit model, Phase 30 optimizer blocker closure, and `src/release/mod.rs`.

## 4. Risks

- Release evidence must not weaken the missing-evidence blocker function used by tests and downstream tooling.
- Cross-platform evidence is represented as release evidence records and must remain explicit about command source.
- Final audit wording must continue avoiding absolute bug-free claims.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 31.1 | docs/specs/tasks/task-31.1-quality-release-evidence-closure.md | Ready |

## 6. Phase Acceptance And Smoke

- Required quality evidence records cover every required quality descriptor.
- `build_release_blocker_ledger()` returns zero entries and `release_ready=true`.
- Final bug-zero audit can pass when all final evidence is present, the ledger is empty, and no release-blocking bugs remain.
- `cargo build`, `cargo check`, `cargo test`, `cargo test release::`, `cargo test test_31_1`, and `cargo build --examples` pass from the repository root.
