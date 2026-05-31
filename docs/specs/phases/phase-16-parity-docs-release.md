# Phase 16 - parity-docs-release

**Status**: In Progress
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: 10,11,12,13,14

## 1. Goal

upstream parity fixtures、docs/examples、feature flags、release packaging

## 2. Scope

tests/parity/ + examples/ + docs/

## 3. Dependencies

10,11,12,13,14

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 16.1 | docs/specs/tasks/task-16.1-parity-suite.md | Done |
| 16.2 | docs/specs/tasks/task-16.2-docs-examples.md | In Progress |
| 16.3 | docs/specs/tasks/task-16.3-release.md | Draft |

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- cargo build passes from repository root.
- cargo test passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
