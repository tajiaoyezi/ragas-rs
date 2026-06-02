# Task 26.1 - provider-protocol-parity-closure

**Status**: Ready
**Phase**: 26
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The consolidated release blocker ledger still contains eight provider entries: `provider::azure-openai`, `provider::google`, `provider::haystack`, `provider::huggingface`, `provider::instructor`, `provider::litellm`, `provider::oci-genai`, and `provider::openai-compatible`. Task 18.2 intentionally represented unsupported or unproven live provider families as release blockers. The active goal now requires current-upstream full functional parity evidence, so provider support must move beyond labels into executable request-planning and fixture-backed metadata.

## 2. Goal

Close the provider release-blocker category by implementing deterministic provider protocol contracts and fixture-backed complete parity claims for all tracked upstream provider families.

## 3. Scope And Out-of-Scope

**In scope**:
- Provider protocol descriptors for all tracked upstream provider families and supported kinds.
- Deterministic request plans for LLM, embedding, and structured-output flows, including auth env names, endpoint/path templates, body shape, system prompt handling, schema metadata, and response/usage extraction metadata.
- Fixture-backed `provider::...` parity claims for every tracked family.
- Release ledger evidence that provider blockers are absent while other categories remain visible.

**Out of scope**:
- Live vendor SDK calls in default CI.
- Shipping or persisting API keys.
- Network tests that require external service credentials.

## 4. Actors

- Rust caller selecting a provider family.
- Release owner inspecting provider blockers.
- QA engineer reviewing provider request fixtures against upstream provider files.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-18.2-provider-adapter-contracts.md
- test/features/provider-protocol-parity-closure.feature
- Upstream baseline files:
  - `src/ragas/llms/litellm_llm.py`
  - `src/ragas/llms/haystack_wrapper.py`
  - `src/ragas/llms/oci_genai_wrapper.py`
  - `src/ragas/llms/adapters/instructor.py`
  - `src/ragas/llms/adapters/litellm.py`
  - `src/ragas/embeddings/openai_provider.py`
  - `src/ragas/embeddings/google_provider.py`
  - `src/ragas/embeddings/haystack_wrapper.py`
  - `src/ragas/embeddings/huggingface_provider.py`
  - `src/ragas/embeddings/litellm_provider.py`

### 5.2 Imports

Use `src/providers.rs`, `src/lib.rs`, `src/release/mod.rs`, and `tests/parity/fixtures/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [x] **AC1**: Rust exposes provider protocol descriptors for OpenAI-compatible, Azure OpenAI, LiteLLM, Instructor, Haystack, HuggingFace, Google, and OCI GenAI with explicit auth, endpoint, kind, structured-output, and fixture metadata.
- [x] **AC2**: Deterministic request planning preserves upstream-relevant LLM, embedding, and structured-output payload semantics without leaking authorization secrets.
- [x] **AC3**: Provider parity claims are fixture-backed `Complete`, and the consolidated release-blocker ledger has no `Provider` category entries after this task.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-26.1.1 | TEST-26.1.1 | Spec Ready |
| AC2 | SCEN-26.1.2 | TEST-26.1.2 | Spec Ready |
| AC3 | SCEN-26.1.3 | TEST-26.1.3 | Spec Ready |

## 8. Risks

- Deterministic request planning does not prove live vendor availability, quota behavior, or authentication success.
- Wrapper providers can hide runtime behavior behind third-party objects; the Rust contract must state that delegated wrapper calls are deterministic boundary contracts, not embedded vendor SDK reimplementations.
- Existing task 18.2 tests intentionally expect provider blockers; this task must update those expectations only after RED captures the old blocker behavior.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build
- Providers Test
- Parity Test

## 10. Completion Notes

- **完成日期**：待实施后回填
- **改动文件**：待实施后回填
- **commit 列表**：待实施后回填
- **§9 Verification 结果**：待实施后回填
- **剩余风险 / 未做项**：待实施后回填
- **下游 task 影响**：待实施后回填
