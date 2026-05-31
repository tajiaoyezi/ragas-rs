# Task 5.1 - schema-core

**Status**: Done
**Phase**: 5
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

MultiTurnSample、Message、ToolCall、rubric/reference/metadata schema

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: src/schema.rs and public exports in src/lib.rs.
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
| AC1 | SCEN-5.1.1 | TEST-5.1.1 | Done |
| AC2 | SCEN-5.1.2 | TEST-5.1.2 | Done |
| AC3 | SCEN-5.1.3 | TEST-5.1.3 | Done |

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
- **改动文件**：src/lib.rs, src/schema.rs, docs/specs/tasks/task-5.1-schema-core.md
- **commit 列表**：2d382c5 docs(spec): task-5.1 Ready; 1005bd3 docs(spec): task-5.1 进入实施; 7d23266 test(schema): 加 task-5.1 RED 测试; 3389ec3 feat(schema): 实现 task-5.1 多轮样本 schema
- **§9 Verification 结果**：RED: cargo test 预期失败，13 passed / 3 failed，失败点为 schema constructors 的 unimplemented!；GREEN: cargo test 通过，16 passed / 0 failed；install/build: cargo build 通过；typecheck: cargo check 通过；unit-test: cargo test 通过。
- **剩余风险 / 未做项**：完整 Python ragas dataset schema parity、文件 IO 与 validation 仍由 task 5.2/5.3 覆盖；本 task 只完成多轮样本核心 DTO。
- **下游 task 影响**：task 5.2 可复用 Message、ToolCall、Rubric、MultiTurnSample 做 dataset serde IO；task 5.3 可在这些类型之上实现 schema validation。
