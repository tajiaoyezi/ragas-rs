# Phase 28 - metric-fixture-parity-closure

**Status**: Ready
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 27

## 1. Goal

Close the metric release-blocker category by converting every tracked upstream metric family into fixture-backed complete parity evidence against the current upstream baseline.

## 2. Scope

The phase covers the 25 non-complete metric families still reported by the release blocker ledger after Phase 27. It must add deterministic golden fixture metadata, execute fixture validation in Rust default CI, and keep metric catalog status honest by tying every `Complete` claim to concrete fixture paths and upstream module paths.

## 3. Dependencies

Phase 19 metric catalog and golden fixture runner, Phase 23 release blocker ledger, Phase 27 integration closure, and upstream files under `src/ragas/metrics/` at baseline `298b68274234c060deacab3cf5fb52aa3a20e885`.

## 4. Risks

- Fixture-backed contract parity can be mistaken for live LLM/provider parity if fixture mode is not explicit.
- Metric families combine deterministic, embedding, LLM judge, tool, SQL, and multimodal semantics; fixture coverage must preserve value type and provider requirement metadata for each family.
- Large fixture tables can drift from upstream source paths if new upstream metric modules are added.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 28.1 | docs/specs/tasks/task-28.1-metric-fixture-parity-closure.md | Ready |

## 6. Phase Acceptance And Smoke

- Metric release-blocker ledger category is empty after task completion.
- Every tracked metric family has a fixture-backed `Complete` parity claim.
- Metric fixture metadata records upstream module path, optional upstream test path, fixture path, fixture mode, tolerance, provider requirement, sample kind, and value type.
- `cargo build`, `cargo check`, `cargo test`, `cargo test metrics::`, `cargo test parity::`, and `cargo build --examples` pass from the repository root.
