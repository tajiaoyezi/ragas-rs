# Task 14.2 - integrations

**Status**: Done
**Phase**: 14
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

tracing hooks and optional LangSmith/Langfuse/Opik-style adapters

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/integrations/`.
- Behavior listed in §6 acceptance criteria.
- Unit tests and, where applicable, parity fixtures for upstream ragas semantics.

**Out of scope**:
- Unrelated phases from the complete refactor matrix.
- Hidden Python runtime dependency or pyo3 bridge.
- Marking parity complete without explicit fixture evidence.

## 4. Actors

- Rust caller using ragas-rs.
- Evaluation framework maintainer tracking Python ragas parity.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-complete-refactor.prd.md
- docs/specs/ragas-complete-refactor-breakdown.md
- test/features/integrations.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Tracing integration receives callback events
- **AC2**: External integrations are feature-gated
- **AC3**: Payload redaction is applied before export

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-14.2.1 | TEST-14.2.1 | Done |
| AC2 | SCEN-14.2.2 | TEST-14.2.2 | Done |
| AC3 | SCEN-14.2.3 | TEST-14.2.3 | Done |

## 8. Risks

- Upstream Python semantics may not map one-to-one to Rust types.
- Optional external integrations must not leak into the default dependency set.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：2026-05-31
- **改动文件**：
  - `src/integrations/mod.rs`（新增 tracing integration、feature gate registry、payload redaction 与 TEST-14.2.1~14.2.3）
  - `src/lib.rs`（导出 integrations public API）
- **commit 列表**：
  - `3d0e7f4` docs(spec): task-14.2 Ready
  - `0280497` docs(spec): task-14.2 进入实施
  - `53e0770` test(integrations): 加 task-14.2 RED 测试
  - `da38e37` feat(integrations): 实现 task-14.2 tracing hooks
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 100 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：LangSmith/Langfuse/Opik 目前是 feature-gated protocol placeholder，不绑定外部 SDK；真实 SDK adapter 需要后续 feature-specific task 或 ADR。
- **下游 task 影响**：task 14.3 CLI 可复用 tracing integration 做 export/redaction smoke；phase 16 docs 需记录 external integration feature-gate 行为。
