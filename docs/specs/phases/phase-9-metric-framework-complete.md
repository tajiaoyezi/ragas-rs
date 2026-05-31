# Phase 9 - metric-framework-complete

**Status**: In Progress
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: 5,6,8

## 1. Goal

metric base、validators、result schema、metric collection registry、parity labels

## 2. Scope

src/metrics/base.rs + src/metrics/result.rs + src/metrics/validators.rs

## 3. Dependencies

5,6,8

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 9.1 | docs/specs/tasks/task-9.1-metric-base.md | Done |
| 9.2 | docs/specs/tasks/task-9.2-metric-result.md | Draft |
| 9.3 | docs/specs/tasks/task-9.3-metric-registry.md | Draft |

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- cargo build passes from repository root.
- cargo test passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
