# Task 18.4 - integration-callback-contracts

**Status**: In Progress
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
| AC1 | SCEN-18.4.1 | TEST-18.4.1 | Not Started |
| AC2 | SCEN-18.4.2 | TEST-18.4.2 | Not Started |
| AC3 | SCEN-18.4.3 | TEST-18.4.3 | Not Started |

## 8. Risks

- Generic descriptors must not be misread as vendor-certified integrations.
- Redaction must be conservative because callback payloads can contain prompts or secrets.

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
