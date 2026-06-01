# Task 18.2 - provider-adapter-contracts

**Status**: Done
**Phase**: 18
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Upstream providers include OpenAI, Azure-like behavior, LiteLLM, Instructor, Haystack, HuggingFace, Google, OCI, and structured LLM wrappers. Current Rust code has OpenAI-compatible and mock behavior, but lacks explicit provider capability and system-prompt parity metadata.

## 2. Goal

Implement provider adapter descriptors that track upstream provider families, structured-output capability, system prompt support, and deterministic mock/live mode.

## 3. Scope And Out-of-Scope

**In scope**:
- Provider capability descriptors and registry entries.
- System prompt attachment behavior for structured LLM request metadata.
- Deterministic tests for OpenAI-compatible, Instructor/LiteLLM structured, and embedding provider families.

**Out of scope**:
- Live external SDK calls in default CI.
- Complete vendor SDK reimplementation.

## 4. Actors

- Rust caller choosing providers.
- Maintainer tracking provider parity.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/ragas-latest-gap-analysis.md
- test/features/provider-adapter-contracts.feature

### 5.2 Imports

Use `src/llm.rs`, `src/providers.rs`, and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Provider registry exports upstream provider family descriptors and live/deterministic mode.
- **AC2**: Structured LLM descriptors record system prompt support for Instructor and LiteLLM structured families.
- **AC3**: Unsupported live provider families are represented as release-blocking parity claims, not silently marked complete.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-18.2.1 | TEST-18.2.1 | Done |
| AC2 | SCEN-18.2.2 | TEST-18.2.2 | Done |
| AC3 | SCEN-18.2.3 | TEST-18.2.3 | Done |

## 8. Risks

- Provider names can imply stronger runtime compatibility than deterministic descriptors prove.
- System prompts can leak into wrong message positions if not normalized.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：2026-06-01
- **改动文件**：src/providers.rs; src/lib.rs
- **commit 列表**：
  - 0c0f36e docs(spec): task-18.2 进入实施
  - 8e91e03 test(providers): 加 task-18.2 RED 测试
  - f46e76c feat(providers): 实现 task-18.2 provider adapter contracts
- **RED 结果**：`cargo test test_18_2` failed as expected with 3 failing 18.2 tests because provider descriptors, structured descriptors, and release blockers were empty.
- **§9 Verification 结果**：
  - install: `cargo build` passed
  - typecheck: `cargo check` passed
  - unit-test: `cargo test` passed, 139 passed / 0 failed
  - build: `cargo build` passed
- **剩余风险 / 未做项**：Live SDK/protocol parity for Google, Haystack, HuggingFace, OCI, LiteLLM, Instructor, OpenAI-compatible, and Azure families is intentionally not claimed complete without fixtures; non-complete live providers are exposed as release-blocking parity claims.
- **下游 task 影响**：Release gates can now consume `provider_parity_claims()` to block unsupported or unproven provider parity; task 18.3 can proceed without depending on provider implementation details.
