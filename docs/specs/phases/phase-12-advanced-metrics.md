# Phase 12 - advanced-metrics

**Status**: In Progress
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: 9,7,8

## 1. Goal

rubrics、agent、tool call、SQL、多模态、summarization metrics

## 2. Scope

src/metrics/advanced/

## 3. Dependencies

9,7,8

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 12.1 | docs/specs/tasks/task-12.1-rubrics.md | Done |
| 12.2 | docs/specs/tasks/task-12.2-agents-tools.md | Done |
| 12.3 | docs/specs/tasks/task-12.3-sql-multimodal-summary.md | Done |

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- cargo build passes from repository root.
- cargo test passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
