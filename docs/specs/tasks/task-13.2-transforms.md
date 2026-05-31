# Task 13.2 - transforms

**Status**: Done
**Phase**: 13
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

splitters, extractors, filters, relationship builders

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
- test/features/transforms.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Splitters produce stable chunks with source metadata
- **AC2**: Extractors attach entities/themes/summaries
- **AC3**: Relationship builders create deterministic edges

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-13.2.1 | TEST-13.2.1 | Done |
| AC2 | SCEN-13.2.2 | TEST-13.2.2 | Done |
| AC3 | SCEN-13.2.3 | TEST-13.2.3 | Done |

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
  - `src/testset/mod.rs`（新增 TextChunk、ExtractionBundle、splitter/extractor/relationship builder 与 TEST-13.2.1~13.2.3）
  - `src/lib.rs`（导出 task-13.2 public crate API）
- **commit 列表**：
  - `6645996` docs(spec): task-13.2 Ready
  - `76ab3a4` docs(spec): task-13.2 进入实施
  - `7e62242` test(testset): 加 task-13.2 RED 测试
  - `3de98dc` feat(testset): 实现 task-13.2 transforms
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 91 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：splitter 是确定性词边界实现，不是 tokenizer-aware 或 language-aware splitter；复杂 extractor/provider 调用和 Python ragas parity 由后续 synthesizer/parity task 扩展。
- **下游 task 影响**：task 13.3 可直接复用 TextChunk、ExtractionBundle、contains/next edges 作为 synthesizer 输入。
