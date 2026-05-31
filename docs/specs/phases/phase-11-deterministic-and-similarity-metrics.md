# Phase 11 - deterministic-and-similarity-metrics

**Status**: In Progress
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: 9,7

## 1. Goal

BLEU/ROUGE/CHRF/string/semantic similarity/classic metrics

## 2. Scope

src/metrics/traditional/

## 3. Dependencies

9,7

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 11.1 | docs/specs/tasks/task-11.1-lexical.md | Done |
| 11.2 | docs/specs/tasks/task-11.2-semantic.md | Draft |
| 11.3 | docs/specs/tasks/task-11.3-quoted-spans.md | Draft |

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- cargo build passes from repository root.
- cargo test passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
