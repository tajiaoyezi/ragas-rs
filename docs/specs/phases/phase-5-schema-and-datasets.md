# Phase 5 - schema-and-datasets

**Status**: Draft
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: 1,4

## 1. Goal

完整样本、消息、tool call、多轮数据集、serde schema 与 validation

## 2. Scope

src/schema/ + src/dataset.rs

## 3. Dependencies

1,4

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 5.1 | docs/specs/tasks/task-5.1-schema-core.md | Draft |
| 5.2 | docs/specs/tasks/task-5.2-dataset-io.md | Draft |
| 5.3 | docs/specs/tasks/task-5.3-validation.md | Draft |

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- cargo build passes from repository root.
- cargo test passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
