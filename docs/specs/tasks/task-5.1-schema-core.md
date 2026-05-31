# Task 5.1 - schema-core

**Status**: Ready
**Phase**: 5
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

MultiTurnSample、Message、ToolCall、rubric/reference/metadata schema

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: $(System.Collections.Hashtable.Module).
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
- test/features/schema-core.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Message and ToolCall model supports user/assistant/system/tool roles and tool-call IDs
- **AC2**: MultiTurnSample preserves ordered messages, reference, rubrics, and metadata
- **AC3**: Schema types serialize and deserialize without losing optional fields

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-5.1.1 | TEST-5.1.1 | Not Started |
| AC2 | SCEN-5.1.2 | TEST-5.1.2 | Not Started |
| AC3 | SCEN-5.1.3 | TEST-5.1.3 | Not Started |

## 8. Risks

- Upstream Python semantics may not map one-to-one to Rust types.
- Optional external integrations must not leak into the default dependency set.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：待实施
- **改动文件**：待实施
- **commit 列表**：待实施
- **§9 Verification 结果**：待实施
- **剩余风险 / 未做项**：待实施
- **下游 task 影响**：待实施
