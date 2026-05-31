# Phase 8 - prompts-and-parsers

**Status**: In Progress
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: 5,6

## 1. Goal

prompt templates、few-shot、typed output parser、judge JSON parser、多模态 prompt scaffold

## 2. Scope

src/prompts/

## 3. Dependencies

5,6

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 8.1 | docs/specs/tasks/task-8.1-prompt-core.md | Done |
| 8.2 | docs/specs/tasks/task-8.2-output-parser.md | Done |
| 8.3 | docs/specs/tasks/task-8.3-multimodal-prompt.md | Draft |

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- cargo build passes from repository root.
- cargo test passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
