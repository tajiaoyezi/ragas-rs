# Task 13.1 - graph-core

**Status**: Done
**Phase**: 13
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

knowledge graph node/edge model and graph queries

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/testset/`.
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
- test/features/graph-core.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Graph stores nodes, relationships, and typed properties
- **AC2**: Graph queries filter by type and relationship
- **AC3**: Graph serialization roundtrips fixtures

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-13.1.1 | TEST-13.1.1 | Done |
| AC2 | SCEN-13.1.2 | TEST-13.1.2 | Done |
| AC3 | SCEN-13.1.3 | TEST-13.1.3 | Done |

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
  - `src/testset/mod.rs`（新增 KnowledgeGraph、GraphNode、GraphEdge、GraphProperty 与 TEST-13.1.1~13.1.3）
  - `src/lib.rs`（导出 testset graph core public API）
- **commit 列表**：
  - `8f7a7bb` docs(spec): task-13.1 Ready
  - `b73db80` docs(spec): task-13.1 进入实施
  - `378a70c` test(testset): 加 task-13.1 RED 测试
  - `e863d3c` feat(testset): 实现 task-13.1 graph core
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 88 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：当前 graph core 使用轻量 Vec-backed model，不引入 petgraph；复杂图遍历和 Python ragas graph parity 后续由 transforms/synthesizers/parity task 扩展。
- **下游 task 影响**：task 13.2 可基于 KnowledgeGraph 增加 splitters/extractors/relationship builders；task 13.3 可复用 graph query 作为 synthesizer 输入。
