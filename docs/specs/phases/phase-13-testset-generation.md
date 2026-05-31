# Phase 13 - testset-generation

**Status**: Draft
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: 5,7,8

## 1. Goal

graph、transforms、extractors、splitters、relationship builders、persona、single/multi-hop synthesizers

## 2. Scope

src/testset/

## 3. Dependencies

5,7,8

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 13.1 | docs/specs/tasks/task-13.1-graph-core.md | Draft |
| 13.2 | docs/specs/tasks/task-13.2-transforms.md | Draft |
| 13.3 | docs/specs/tasks/task-13.3-synthesizers.md | Draft |

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- cargo build passes from repository root.
- cargo test passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
