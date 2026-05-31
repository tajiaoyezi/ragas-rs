# Task 16.3 - release

**Status**: Done
**Phase**: 16
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

feature flags, crate metadata, CI gates, release checklist

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/release/`, `Cargo.toml`, `.github/workflows/`, and `docs/`.
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
- test/features/release.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Cargo features match optional capability groups
- **AC2**: CI runs build, check, test, and parity gates
- **AC3**: Release checklist includes versioning and rollback steps

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-16.3.1 | TEST-16.3.1 | Done |
| AC2 | SCEN-16.3.2 | TEST-16.3.2 | Done |
| AC3 | SCEN-16.3.3 | TEST-16.3.3 | Done |

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
  - `Cargo.toml`（新增 release feature groups）
  - `.github/workflows/ci.yml`（build/check/test/parity CI gates）
  - `docs/release-checklist.md`（versioning、publish dry-run、rollback steps）
  - `src/release/mod.rs`（release gate file registry 与 TEST-16.3.1~16.3.3）
  - `src/lib.rs`（导出 release public API）
- **commit 列表**：
  - `f3dfd8b` docs(spec): task-16.3 Ready
  - `3e205f6` docs(spec): task-16.3 进入实施
  - `cea1618` test(release): 加 task-16.3 RED 测试
  - `e1b816f` feat(release): 实现 task-16.3 release gates
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 121 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
  - extra: ✅ `cargo test parity::` (3 passed / 0 failed)
- **剩余风险 / 未做项**：CI workflow 尚未在远端 Actions 实跑；本地等价命令已通过。
- **下游 task 影响**：全部 planned S2V tasks 已完成；后续只剩发布/远端 CI 实际执行。
