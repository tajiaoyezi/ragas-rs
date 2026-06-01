# Task 18.2 - provider-adapter-contracts

**Status**: Ready
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
| AC1 | SCEN-18.2.1 | TEST-18.2.1 | Not Started |
| AC2 | SCEN-18.2.2 | TEST-18.2.2 | Not Started |
| AC3 | SCEN-18.2.3 | TEST-18.2.3 | Not Started |

## 8. Risks

- Provider names can imply stronger runtime compatibility than deterministic descriptors prove.
- System prompts can leak into wrong message positions if not normalized.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：<TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>

