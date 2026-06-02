# Task 27.1 - integration-contract-parity-closure

**Status**: In Progress
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
| AC1 | SCEN-27.1.1 | TEST-27.1.1 | Spec Ready |
| AC2 | SCEN-27.1.2 | TEST-27.1.2 | Spec Ready |
| AC3 | SCEN-27.1.3 | TEST-27.1.3 | Spec Ready |

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

- **完成日期**：待实施后回填
- **改动文件**：待实施后回填
- **commit 列表**：待实施后回填
- **§9 Verification 结果**：待实施后回填
- **剩余风险 / 未做项**：待实施后回填
- **下游 task 影响**：待实施后回填
