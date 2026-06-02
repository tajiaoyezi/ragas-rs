# Task 26.1 - provider-protocol-parity-closure

**Status**: Done
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
| AC1 | SCEN-26.1.1 | TEST-26.1.1 | Done |
| AC2 | SCEN-26.1.2 | TEST-26.1.2 | Done |
| AC3 | SCEN-26.1.3 | TEST-26.1.3 | Done |

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

- **完成日期**：2026-06-02
- **改动文件**：`src/providers.rs`; `src/lib.rs`; `src/release/mod.rs`; `tests/parity/fixtures/provider_openai_compatible.json`; `tests/parity/fixtures/provider_azure_openai.json`; `tests/parity/fixtures/provider_litellm.json`; `tests/parity/fixtures/provider_instructor.json`; `tests/parity/fixtures/provider_haystack.json`; `tests/parity/fixtures/provider_huggingface.json`; `tests/parity/fixtures/provider_google.json`; `tests/parity/fixtures/provider_oci_genai.json`; `docs/specs/tasks/task-26.1-provider-protocol-parity-closure.md`
- **commit 列表**：
  - `68b8746 docs(spec): add task-26.1 provider protocol parity closure`
  - `37daa10 docs(spec): task-26.1 进入实施`
  - `f008a25 test(providers): 加 task-26.1 RED 测试`
  - `f9e096e feat(providers): 实现 task-26.1 provider protocol parity closure`
- **RED 结果**：`cargo test test_26_1` failed as expected with 3 tests discovered, 0 passed, 3 failed. The failures were `TEST-26.1.1`, `TEST-26.1.2`, and `TEST-26.1.3`, covering missing provider protocol descriptors, unimplemented request planning, and stale provider release blockers.
- **§9 Verification 结果**：
  - Install: `cargo build` passed.
  - Typecheck: `cargo check` passed.
  - Unit Test: `cargo test` passed with 205 passed, 0 failed.
  - Build: `cargo build` passed.
  - Providers Test: `cargo test providers::` passed with 9 passed, 0 failed.
  - Parity Test: `cargo test parity::` passed with 12 passed, 0 failed.
- **剩余风险 / 未做项**：Default CI now proves provider request-planning, auth redaction, fixture metadata, and release-ledger closure; live vendor authentication, quotas, SDK-specific runtime behavior, and network failures remain opt-in live-service evidence and are not claimed as default CI coverage.
- **下游 task 影响**：Provider release blockers dropped from 8 to 0; consolidated ledger moved from 65 to 57 non-waived blockers and now starts with Integration, Metric, Testset, Optimizer, and Quality categories.
