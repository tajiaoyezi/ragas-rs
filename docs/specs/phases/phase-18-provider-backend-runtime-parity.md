# Phase 18 - provider-backend-runtime-parity

**Status**: Done
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 17

## 1. Goal

Move provider, backend, runtime, cache, tokenizer, cost, callback, and integration-facing behavior from broad scaffolding toward upstream-compatible parity contracts.

## 2. Scope

`src/runtime.rs`, `src/llm.rs`, `src/providers.rs`, `src/backends/`, `src/integrations/`, `src/release/`, and related parity fixtures.

## 3. Dependencies

Phase 17 latest baseline, fixture policy, quality gates, and bug-zero audit.

## 4. Risks

- Upstream provider integrations depend on external SDK behavior that cannot run in deterministic default CI.
- Cache compatibility can accidentally include sensitive callback or credential fields in keys.
- Tokenizer parity must preserve lazy initialization semantics without importing Python or tokenizer runtimes.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 18.1 | docs/specs/tasks/task-18.1-runtime-cache-tokenizer-cost.md | Done |
| 18.2 | docs/specs/tasks/task-18.2-provider-adapter-contracts.md | Done |
| 18.3 | docs/specs/tasks/task-18.3-backend-registry-diskcache.md | Done |
| 18.4 | docs/specs/tasks/task-18.4-integration-callback-contracts.md | Done |

## 6. Phase Acceptance And Smoke

- Upstream runtime/cache/tokenizer/cost/provider/backend/integration categories have task-level parity evidence or release-blocking entries.
- Default deterministic CI does not require external provider credentials or Python runtime.
- `cargo build`, `cargo check`, `cargo test`, and `cargo test parity::` pass from the repository root.
