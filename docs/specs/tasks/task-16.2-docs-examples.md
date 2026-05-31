# Task 16.2 - docs-examples

**Status**: Done
**Phase**: 16
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

Rust examples mapped to upstream howtos/tutorials

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/docs_examples/`, `examples/`, and `docs/`.
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
- test/features/docs-examples.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Each public workflow has a runnable Rust example
- **AC2**: Examples map to upstream docs section names
- **AC3**: Docs state feature flags and known parity gaps

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-16.2.1 | TEST-16.2.1 | Done |
| AC2 | SCEN-16.2.2 | TEST-16.2.2 | Done |
| AC3 | SCEN-16.2.3 | TEST-16.2.3 | Done |

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
  - `src/docs_examples/mod.rs`（新增 docs/examples registry 与 TEST-16.2.1~16.2.3）
  - `src/lib.rs`（导出 docs_examples public API）
  - `examples/evaluate.rs`、`examples/testset.rs`、`examples/benchmark.rs`（runnable Rust examples）
  - `docs/ragas-rs-user-guide.md`（feature flags、upstream docs mapping、known parity gaps）
- **commit 列表**：
  - `56671ae` docs(spec): task-16.2 Ready
  - `b474b5f` docs(spec): task-16.2 进入实施
  - `8a21f9e` test(docs): 加 task-16.2 RED 测试
  - `f5ca05c` feat(docs): 实现 task-16.2 examples guide
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 118 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
  - extra: ✅ `cargo build --examples`
- **剩余风险 / 未做项**：examples 使用 mock provider/local fixtures；真实 provider howtos 需要用户配置 API key，文档仅声明边界不存储密钥。
- **下游 task 影响**：task 16.3 release 可引用 user guide、examples build 结果和 known parity gaps。
