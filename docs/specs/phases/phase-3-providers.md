# Phase 3 - providers

**Status**: Draft
**PRD**: docs/prds/ragas-rs.prd.md
**Tasks**: docs/specs/tasks/task-3.1-providers.md

## 1. Goal

Provide LLM and embedding provider traits plus an OpenAI-compatible HTTP client and DTO parsing surface.

## 2. Scope

- `src/llm.rs`
- `src/lib.rs`

## 3. Dependencies

- Phase 1 error types.
- Phase 2 metric abstractions for downstream built-in metrics.

## 4. Risks

- External providers differ in JSON details.
- Error reporting must avoid leaking authorization headers.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 3.1 | docs/specs/tasks/task-3.1-providers.md | Draft |

## 6. Phase Acceptance And Smoke

- `cargo check` succeeds.
- `cargo test llm` succeeds.
- Provider DTO tests run without network access.
