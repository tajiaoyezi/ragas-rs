# Phase 2 - metric-abstractions

**Status**: Done
**PRD**: docs/prds/ragas-rs.prd.md
**Tasks**: docs/specs/tasks/task-2.1-metric-abstractions.md

## 1. Goal

Define type-safe metric values, metric results, async `Metric` trait, and custom metric helper.

## 2. Scope

- `src/metric.rs`
- `src/lib.rs`

## 3. Dependencies

- Phase 1 dataset and error types.

## 4. Risks

- Trait object ergonomics can conflict with async lifetimes.
- Metric output categories must remain extensible without changing evaluate core.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 2.1 | docs/specs/tasks/task-2.1-metric-abstractions.md | Done |

## 6. Phase Acceptance And Smoke

- `cargo check` succeeds.
- `cargo test metric` succeeds.
- Existing dataset tests remain green.
