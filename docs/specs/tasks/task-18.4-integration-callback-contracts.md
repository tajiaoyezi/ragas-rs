# Task 18.4 - integration-callback-contracts

**Status**: Done
**Phase**: 18
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Upstream exposes integrations for LangChain, LangGraph, LangSmith, LlamaIndex, AG-UI, Bedrock, Griptape, Helicone, Opik, R2R, Swarm, and tracing destinations. Current Rust code has generic tracing/redaction only.

## 2. Goal

Implement integration-facing contract descriptors and callback payload normalization so unsupported integrations are visible release blockers and supported contracts are testable without vendor SDKs.

## 3. Scope And Out-of-Scope

**In scope**:
- Integration registry descriptors.
- Callback payload schema and redaction policy.
- Release-blocking parity claims for unsupported integrations.

**Out of scope**:
- Vendor SDK runtime dependencies in default CI.
- Hosted tracing exporters.

## 4. Actors

- Maintainer integrating with observability tools.
- Release owner checking integration parity.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/ragas-latest-gap-analysis.md
- test/features/integration-callback-contracts.feature

### 5.2 Imports

Use `src/integrations/`, `src/runtime.rs`, and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Integration registry lists every upstream integration family with implementation and test mode.
- **AC2**: Callback payload normalization redacts secrets before export and preserves lifecycle event fields.
- **AC3**: Unsupported integration families create release-blocking parity claims.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-18.4.1 | TEST-18.4.1 | Done |
| AC2 | SCEN-18.4.2 | TEST-18.4.2 | Done |
| AC3 | SCEN-18.4.3 | TEST-18.4.3 | Done |

## 8. Risks

- Generic descriptors must not be misread as vendor-certified integrations.
- Redaction must be conservative because callback payloads can contain prompts or secrets.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：2026-06-01
- **改动文件**：src/integrations/mod.rs; src/lib.rs
- **commit 列表**：
  - c366bad docs(spec): task-18.4 进入实施
  - b7ec654 test(integrations): 加 task-18.4 RED 测试
  - fdaa9e5 feat(integrations): 实现 task-18.4 callback contracts
- **RED 结果**：`cargo test test_18_4` failed as expected with 3 failing 18.4 tests because integration descriptors, callback redaction normalization, and unsupported integration blockers were missing.
- **§9 Verification 结果**：
  - install: `cargo build` passed
  - typecheck: `cargo check` passed
  - unit-test: `cargo test` passed, 145 passed / 0 failed
  - build: `cargo build` passed
- **剩余风险 / 未做项**：Generic tracing is deterministic and tested; LangSmith, Langfuse, and Opik remain partial contract descriptors without vendor SDK certification; LangChain, LangGraph, LlamaIndex, AG-UI, Bedrock, Griptape, Helicone, R2R, and Swarm remain release-blocking KnownGap claims.
- **下游 task 影响**：Release gates can consume `integration_parity_claims()`; later full-parity phases must either implement vendor-specific fixtures or keep these claims blocking release.
