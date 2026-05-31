# Phase 10 - rag-metrics

**Status**: In Progress
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: 9,7

## 1. Goal

faithfulness/context/answer/factual/noise/RAG 指标全批次迁移

## 2. Scope

src/metrics/rag/

## 3. Dependencies

9,7

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 10.1 | docs/specs/tasks/task-10.1-context-metrics.md | Done |
| 10.2 | docs/specs/tasks/task-10.2-faithfulness-family.md | Done |
| 10.3 | docs/specs/tasks/task-10.3-answer-quality.md | Draft |

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- cargo build passes from repository root.
- cargo test passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
