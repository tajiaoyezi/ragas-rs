# Phase 20 - testset-generation-full-parity

**Status**: Ready
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 19

## 1. Goal

Complete testset-generation parity contracts for graph persistence/query behavior, transform/extractor stages, and deterministic synthesizer prompt fixtures.

## 2. Scope

`src/testset/`, `src/prompts/`, `src/parity/`, and deterministic fixtures for graph, transform, and synthesizer outputs.

## 3. Dependencies

Phase 19 metric fixture and release-blocker rules.

## 4. Risks

- Upstream testset generation is LLM-driven and graph-heavy, making deterministic parity difficult.
- Pre-chunked generation and relationship builders can silently reorder or duplicate samples.
- Graph metadata schemas can drift without fixture-backed round trips.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 20.1 | docs/specs/tasks/task-20.1-graph-persistence-query-parity.md | Done |
| 20.2 | docs/specs/tasks/task-20.2-transform-engine-extractor-parity.md | Done |
| 20.3 | docs/specs/tasks/task-20.3-synthesizer-prompt-fixture-parity.md | Ready |

## 6. Phase Acceptance And Smoke

- Graph save/load/query contracts are fixture-backed.
- Transform and extractor descriptors classify implemented, partial, and known-gap behavior.
- Single-hop, multi-hop, and pre-chunked synthesizer prompt fixtures are deterministic.
- `cargo build`, `cargo check`, `cargo test`, and `cargo test parity::` pass from the repository root.
