# Phase 14 - backends-integrations-cli

**Status**: Draft
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: 6,9,13

## 1. Goal

JSONL/CSV/in-memory backend、optional integrations、CLI evaluate/testset/benchmark

## 2. Scope

src/backends/ + src/integrations/ + src/cli/

## 3. Dependencies

6,9,13

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 14.1 | docs/specs/tasks/task-14.1-backends.md | Draft |
| 14.2 | docs/specs/tasks/task-14.2-integrations.md | Draft |
| 14.3 | docs/specs/tasks/task-14.3-cli.md | Draft |

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- cargo build passes from repository root.
- cargo test passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
