# Phase 4 - evaluator-builtins

**Status**: Done
**PRD**: docs/prds/ragas-rs.prd.md
**Tasks**: docs/specs/tasks/task-4.1-evaluator-builtins.md

## 1. Goal

Implement async batch evaluation and built-in Faithfulness, ResponseRelevancy, and ContextPrecision metrics.

## 2. Scope

- `src/eval.rs`
- `src/metric.rs`
- `src/llm.rs`
- `src/lib.rs`

## 3. Dependencies

- Phase 1 dataset and error types.
- Phase 2 metric abstractions.
- Phase 3 provider traits and DTO helpers.

## 4. Risks

- Concurrency must isolate sample-level errors instead of failing the whole report.
- Built-in metric semantics must be explicit about v1.0 heuristic compatibility.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 4.1 | docs/specs/tasks/task-4.1-evaluator-builtins.md | Done |

## 6. Phase Acceptance And Smoke

- `cargo build` succeeds.
- `cargo test` succeeds.
- `docs/s2v-adapter.md` indexes all tasks as Done when the phase closes.
