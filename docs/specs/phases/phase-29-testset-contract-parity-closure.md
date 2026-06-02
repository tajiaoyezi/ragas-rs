# Phase 29 - testset-contract-parity-closure

**Status**: Done
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 28

## 1. Goal

Close the testset release-blocker category by implementing deterministic Rust contracts for the remaining graph, transform, and synthesizer gaps tracked from the current upstream baseline.

## 2. Scope

The phase covers graph clustering, advanced graph query contracts, fixture-backed LLM extractor output parsing, deterministic graph filtering, and pre-chunked synthesizer generation. All default CI evidence must remain deterministic and must not require live LLM calls.

## 3. Dependencies

Phase 20 testset contracts, Phase 23 release blocker ledger, Phase 28 metric fixture closure, and upstream files under `src/ragas/testset/` at baseline `298b68274234c060deacab3cf5fb52aa3a20e885`.

## 4. Risks

- Live upstream transform behavior can be overclaimed if captured LLM extractor fixtures are not clearly marked deterministic.
- Graph cluster ordering must be stable to avoid noisy fixture drift.
- Pre-chunked generation can diverge from single-hop and multi-hop sample metadata if provenance fields are not explicit.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 29.1 | docs/specs/tasks/task-29.1-testset-contract-parity-closure.md | Done |

## 6. Phase Acceptance And Smoke

- Testset release-blocker ledger category is empty after task completion.
- Graph, transform, and synthesizer parity claims are all fixture-backed `Complete` claims.
- New tests prove deterministic graph clustering/query, captured LLM extractor parsing, graph filtering, and pre-chunked synthesis.
- `cargo build`, `cargo check`, `cargo test`, `cargo test testset::`, `cargo test parity::`, and `cargo build --examples` pass from the repository root.
