# Phase 24 - release-blocker-closure

**Status**: Ready
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 23

## 1. Goal

Resolve concrete release blockers emitted by the final audit ledger until the project can make a scoped no-known-release-blocking-bugs claim.

## 2. Scope

Release-blocker closure tasks that convert specific `KnownGap`, `Partial`, or missing-evidence blockers into implemented, tested, fixture-backed Rust functionality.

## 3. Dependencies

Phase 23 release blocker ledger, gap resolution policy, final audit, and the module-specific parity claim functions that feed them.

## 4. Risks

- Marking a blocker complete without executable evidence would weaken the final audit.
- Small closure tasks can reduce blocker count while leaving large provider, integration, and testset blockers unresolved.
- Upstream baseline can drift after a closure task; release must re-check `vibrantlabsai/ragas` `main` and latest tag before final readiness.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 24.1 | docs/specs/tasks/task-24.1-experiment-quickstart-closure.md | Done |
| 24.2 | docs/specs/tasks/task-24.2-disk-cache-persistence-closure.md | Ready |

## 6. Phase Acceptance And Smoke

- Each completed task removes or validly resolves at least one release-blocking ledger entry.
- Any parity status changed to `Complete` has deterministic Rust evidence and fixture metadata.
- `cargo build`, `cargo check`, `cargo test`, `cargo test parity::`, and `cargo build --examples` pass from the repository root.
