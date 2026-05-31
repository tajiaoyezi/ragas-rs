# Phase 15 - optimizers-experiments

**Status**: In Progress
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: 9,14

## 1. Goal

experiment model、prompt/model optimizer、benchmark llm/embedding flows

## 2. Scope

src/experiments/ + src/optimizers/

## 3. Dependencies

9,14

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 15.1 | docs/specs/tasks/task-15.1-experiments.md | In Progress |
| 15.2 | docs/specs/tasks/task-15.2-optimizers.md | Draft |
| 15.3 | docs/specs/tasks/task-15.3-benchmarks.md | Draft |

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- cargo build passes from repository root.
- cargo test passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
