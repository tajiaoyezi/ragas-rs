# Task 27.1 - integration-contract-parity-closure

**Status**: Done
**Phase**: 27
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The consolidated release blocker ledger still contains twelve integration entries: `integration::langchain`, `integration::langgraph`, `integration::langsmith`, `integration::llamaindex`, `integration::ag-ui`, `integration::bedrock`, `integration::griptape`, `integration::helicone`, `integration::langfuse`, `integration::opik`, `integration::r2r`, and `integration::swarm`. Task 18.4 intentionally exposed these as release blockers until they had deterministic contract evidence. The active goal requires current-upstream full functional parity evidence, so integration support must move from descriptor labels to executable contract planning and fixture-backed release claims.

## 2. Goal

Close the integration release-blocker category by implementing deterministic integration contract descriptors, export plans, redaction coverage, and fixture-backed complete parity claims for all tracked upstream integration families.

## 3. Scope And Out-of-Scope

**In scope**:
- Integration contract descriptors for every tracked upstream integration family.
- Deterministic export/event plans that preserve lifecycle fields, target operation, boundary mode, and payload redaction.
- Fixture-backed `integration::...` parity claims for every tracked family.
- Release ledger evidence that integration blockers are absent while remaining categories stay visible.

**Out of scope**:
- Vendor SDK runtime dependencies in default CI.
- Live calls to LangSmith, Langfuse, Opik, Bedrock, Helicone, AG-UI endpoints, or framework adapters.
- Persisting API keys, tokens, or credentials.

## 4. Actors

- Rust caller wiring evaluation events into external observability or framework tools.
- Release owner inspecting integration blockers.
- QA engineer reviewing integration contract fixtures against upstream integration files.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-18.4-integration-callback-contracts.md
- test/features/integration-contract-parity-closure.feature
- Upstream baseline files:
  - `src/ragas/integrations/langchain.py`
  - `src/ragas/integrations/langgraph.py`
  - `src/ragas/integrations/langsmith.py`
  - `src/ragas/integrations/llama_index.py`
  - `src/ragas/integrations/ag_ui.py`
  - `src/ragas/integrations/amazon_bedrock.py`
  - `src/ragas/integrations/griptape.py`
  - `src/ragas/integrations/helicone.py`
  - `src/ragas/integrations/tracing/langfuse.py`
  - `src/ragas/integrations/opik.py`
  - `src/ragas/integrations/r2r.py`
  - `src/ragas/integrations/swarm.py`

### 5.2 Imports

Use `src/integrations/mod.rs`, `src/lib.rs`, `src/release/mod.rs`, and `tests/parity/fixtures/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [x] **AC1**: Rust exposes integration contract descriptors for LangChain, LangGraph, LangSmith, LlamaIndex, AG-UI, Bedrock, Griptape, Helicone, Langfuse, Opik, R2R, and Swarm with upstream module, boundary mode, target operation, auth/redaction, lifecycle field, and fixture metadata.
- [x] **AC2**: Deterministic export plans preserve runtime lifecycle fields and redact credentials for observability, endpoint, and delegated framework integrations.
- [x] **AC3**: Integration parity claims are fixture-backed `Complete`, and the consolidated release-blocker ledger has no `Integration` category entries after this task.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-27.1.1 | TEST-27.1.1 | Done |
| AC2 | SCEN-27.1.2 | TEST-27.1.2 | Done |
| AC3 | SCEN-27.1.3 | TEST-27.1.3 | Done |

## 8. Risks

- Deterministic contract parity does not prove live vendor SDK imports, authentication, quotas, or hosted-service availability.
- Wrapper integrations can hide runtime behavior behind third-party SDK objects; Rust contracts must label these as delegated boundaries.
- Existing task 18.4 tests intentionally expect integration blockers; this task must update those expectations only after RED captures the old blocker behavior.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build
- Integrations Test
- Parity Test

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：`src/integrations/mod.rs`; `src/lib.rs`; `src/release/mod.rs`; `tests/parity/fixtures/integration_langchain.json`; `tests/parity/fixtures/integration_langgraph.json`; `tests/parity/fixtures/integration_langsmith.json`; `tests/parity/fixtures/integration_llamaindex.json`; `tests/parity/fixtures/integration_ag_ui.json`; `tests/parity/fixtures/integration_bedrock.json`; `tests/parity/fixtures/integration_griptape.json`; `tests/parity/fixtures/integration_helicone.json`; `tests/parity/fixtures/integration_langfuse.json`; `tests/parity/fixtures/integration_opik.json`; `tests/parity/fixtures/integration_r2r.json`; `tests/parity/fixtures/integration_swarm.json`; `docs/specs/tasks/task-27.1-integration-contract-parity-closure.md`
- **commit 列表**：
  - `65048c8 docs(spec): add task-27.1 integration contract parity closure`
  - `9d66649 docs(spec): task-27.1 进入实施`
  - `2090c5a test(integrations): 加 task-27.1 RED 测试`
  - `433ac1b feat(integrations): 实现 task-27.1 integration contract parity closure`
- **RED 结果**：`cargo test test_27_1` failed as expected with 3 tests discovered, 0 passed, 3 failed. The failures were `TEST-27.1.1`, `TEST-27.1.2`, and `TEST-27.1.3`, covering missing integration contract descriptors, unimplemented export planning, and stale integration release blockers.
- **§9 Verification 结果**：
  - Install: `cargo build` passed.
  - Typecheck: `cargo check` passed.
  - Unit Test: `cargo test` passed with 208 passed, 0 failed.
  - Build: `cargo build` passed.
  - Integrations Test: `cargo test integrations::` passed with 9 passed, 0 failed.
  - Parity Test: `cargo test parity::` passed with 12 passed, 0 failed.
- **剩余风险 / 未做项**：Default CI now proves integration contract descriptors, lifecycle export planning, credential redaction, fixture metadata, and release-ledger closure; live vendor SDK imports, hosted service authentication, quota behavior, and framework-specific runtime execution remain opt-in live-service evidence and are not claimed as default CI coverage.
- **下游 task 影响**：Integration release blockers dropped from 12 to 0; consolidated ledger moved from 57 to 45 non-waived blockers and now starts with Metric, Testset, Optimizer, and Quality categories.
