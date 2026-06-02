# Phase 27 - integration-contract-parity-closure

**Status**: Ready
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 26

## 1. Goal

Close the integration release-blocker category by implementing deterministic Rust integration contracts and fixture-backed parity claims for every upstream integration family tracked by the current baseline.

## 2. Scope

The phase covers LangChain, LangGraph, LangSmith, LlamaIndex, AG-UI, Bedrock, Griptape, Helicone, Langfuse, Opik, R2R, and Swarm integration parity. It must model upstream integration boundaries, event payload normalization, redaction, export operation metadata, and fixture-backed release evidence without making default CI depend on vendor SDKs or network calls.

## 3. Dependencies

Phase 18 integration descriptors, phase 23 release-blocker ledger, phase 26 provider closure, and upstream files under `src/ragas/integrations/`.

## 4. Risks

- Generic Rust contracts can overclaim vendor SDK compatibility if boundary type and default-CI limits are not explicit.
- Observability integrations must preserve lifecycle fields while redacting credentials and tokens.
- Framework wrappers such as LangChain, LangGraph, LlamaIndex, Griptape, R2R, and Swarm require delegated boundary semantics rather than embedded Python SDK behavior.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 27.1 | docs/specs/tasks/task-27.1-integration-contract-parity-closure.md | Ready |

## 6. Phase Acceptance And Smoke

- Integration release-blocker ledger category is empty after task completion.
- Every tracked integration family has a fixture-backed `Complete` parity claim.
- Integration contract plans cover upstream module path, boundary mode, target operation, auth/redaction behavior, and lifecycle event field mapping.
- `cargo build`, `cargo check`, `cargo test`, `cargo test integrations::`, `cargo test parity::`, and `cargo build --examples` pass from the repository root.
