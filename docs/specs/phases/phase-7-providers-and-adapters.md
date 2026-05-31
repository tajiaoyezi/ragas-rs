# Phase 7 - providers-and-adapters

**Status**: Done
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: 5,6

## 1. Goal

LLM/embedding provider matrix、adapter registry、mock/local/http providers

## 2. Scope

src/providers/ + src/llm.rs

## 3. Dependencies

5,6

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 7.1 | docs/specs/tasks/task-7.1-provider-core.md | Done |
| 7.2 | docs/specs/tasks/task-7.2-llm-adapters.md | Done |
| 7.3 | docs/specs/tasks/task-7.3-embedding-adapters.md | Done |

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- cargo build passes from repository root.
- cargo test passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
