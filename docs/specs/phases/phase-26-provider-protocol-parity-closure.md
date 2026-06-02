# Phase 26 - provider-protocol-parity-closure

**Status**: Ready
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 25

## 1. Goal

Close the provider release-blocker category by replacing live-provider `Partial` / `KnownGap` claims with deterministic Rust protocol contracts, fixture-backed parity metadata, and request-planning tests for every upstream provider family tracked by the current baseline.

## 2. Scope

The phase covers provider family parity for OpenAI-compatible, Azure OpenAI, LiteLLM, Instructor, Haystack, HuggingFace, Google, and OCI GenAI. It must prove request/response contract compatibility in default CI without depending on live credentials, vendor SDKs, or network access.

## 3. Dependencies

Phase 18 provider descriptors, phase 23 release-blocker ledger, task 24.3 SDK closure, task 25.1 backend closure, and upstream provider files under `src/ragas/llms/` and `src/ragas/embeddings/`.

## 4. Risks

- Provider fixture-backed parity can overclaim live service compatibility if endpoint/auth/error semantics are not modeled explicitly.
- Wrapper providers such as Haystack and HuggingFace do not map one-to-one to a single HTTP API, so the Rust contract must distinguish delegated wrapper behavior from direct HTTP calls.
- Structured-output providers must preserve system prompt and schema metadata without leaking secrets in diagnostics.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 26.1 | docs/specs/tasks/task-26.1-provider-protocol-parity-closure.md | Ready |

## 6. Phase Acceptance And Smoke

- Provider release-blocker ledger category is empty after task completion.
- Every tracked live provider family has a fixture-backed `Complete` parity claim.
- Provider protocol plans cover auth environment names, endpoint/path templates, request body shape, response extraction, usage extraction, and structured-output metadata where applicable.
- `cargo build`, `cargo check`, `cargo test`, `cargo test providers::`, `cargo test parity::`, and `cargo build --examples` pass from the repository root.
