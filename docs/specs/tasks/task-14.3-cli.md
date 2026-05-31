# Task 14.3 - cli

**Status**: Done
**Phase**: 14
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

ragas evaluate, ragas testset, ragas benchmark

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/cli/`.
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
- test/features/cli.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: CLI evaluate reads dataset and writes report
- **AC2**: CLI testset invokes synthesizer flow
- **AC3**: CLI benchmark prints machine-readable summary

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-14.3.1 | TEST-14.3.1 | Done |
| AC2 | SCEN-14.3.2 | TEST-14.3.2 | Done |
| AC3 | SCEN-14.3.3 | TEST-14.3.3 | Done |

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
  - `src/cli/mod.rs`（新增库内 CLI command runtime、evaluate/testset/benchmark JSON 输出与 TEST-14.3.1~14.3.3）
  - `src/lib.rs`（导出 cli public API）
- **commit 列表**：
  - `903f855` docs(spec): task-14.3 Ready
  - `3f62850` docs(spec): task-14.3 进入实施
  - `18b7baf` test(cli): 加 task-14.3 RED 测试
  - `daeb97d` feat(cli): 实现 task-14.3 command runtime
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 103 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：当前实现是 embeddable library CLI runtime，不绑定 `clap`/二进制入口；如发布真正 `ragas` executable，需要在 release task 中补 binary wrapper。
- **下游 task 影响**：task 15.3 benchmark 可复用 benchmark JSON contract；phase 16 docs/release 需记录 CLI runtime API 与二进制 wrapper 边界。
