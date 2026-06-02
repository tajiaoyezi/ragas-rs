# Phase 30 - optimizer-contract-parity-closure

**Status**: Done
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 29

## 1. Goal

Close the optimizer release-blocker category with deterministic Rust contracts for DSPy and MIPROv2 optimizer planning, cache keys, and fixture-backed parity evidence.

## 2. Scope

The phase covers DSPy and MIPROv2 contract descriptors, deterministic cache planning, deterministic trial scheduling, and complete optimizer parity claims. Default CI must not require the Python DSPy runtime.

## 3. Dependencies

Phase 21 optimizer descriptors, Phase 23 release blocker ledger, Phase 29 testset closure, and upstream files under `src/ragas/optimizers/` at baseline `298b68274234c060deacab3cf5fb52aa3a20e885`.

## 4. Risks

- Contract parity can be mistaken for embedded Python DSPy execution if runtime limits are not explicit.
- MIPROv2 trial schedules must be deterministic by seed to avoid fixture drift.
- Cache keys must continue redacting secret payload fields.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 30.1 | docs/specs/tasks/task-30.1-optimizer-contract-parity-closure.md | Done |

## 6. Phase Acceptance And Smoke

- Optimizer release-blocker ledger category is empty after task completion.
- DSPy and MIPROv2 descriptors are fixture-backed `Complete` claims.
- Deterministic contract planning covers optimizer family, upstream module, cache namespace, trial scheduling, prompt candidates, and runtime limitation text.
- `cargo build`, `cargo check`, `cargo test`, `cargo test optimizers::`, `cargo test parity::`, and `cargo build --examples` pass from the repository root.
