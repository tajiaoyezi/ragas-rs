# Task 5.2 - dataset-io

**Status**: Done
**Phase**: 5
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

JSONL/CSV serde roundtrip、dataset builders、validation diagnostics

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: src/dataset.rs, src/error.rs, src/lib.rs, Cargo.toml, Cargo.lock.
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
- test/features/dataset-io.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Dataset can load and save JSONL for single-turn and multi-turn samples
- **AC2**: CSV import maps required columns into SingleTurnSample with clear errors
- **AC3**: Dataset builders preserve sample order and metadata

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-5.2.1 | TEST-5.2.1 | Done |
| AC2 | SCEN-5.2.2 | TEST-5.2.2 | Done |
| AC3 | SCEN-5.2.3 | TEST-5.2.3 | Done |

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
  - `Cargo.toml`（新增 csv 依赖）
  - `Cargo.lock`（锁定 csv/csv-core）
  - `src/dataset.rs`（新增 EvaluationSample、EvaluationDatasetBuilder、JSONL/CSV serde IO、dataset metadata）
  - `src/error.rs`（新增 DatasetIo 诊断错误）
  - `src/lib.rs`（导出 dataset IO 类型）
  - `docs/specs/tasks/task-5.2-dataset-io.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `54a63c7` docs(spec): task-5.2 Ready
  - `c6872cd` docs(spec): task-5.2 进入实施
  - `79102fe` test(dataset): 加 task-5.2 RED 测试
  - `68ffaa8` feat(dataset): 实现 task-5.2 dataset IO
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 19 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
- **剩余风险 / 未做项**：文件路径级 backend、完整 Python ragas dataset field parity、sample/metric compatibility validation 仍由 task 5.3/14.1/16.1 覆盖；本 task 只声明核心 serde IO 和 builder 行为。
- **下游 task 影响**：task 5.3 可复用 DatasetIo 诊断和 EvaluationSample validation；task 14.1 可在字符串 IO 之上封装本地 JSONL/CSV backend；task 16.1 可使用 JSONL roundtrip 作为 parity fixture 基础。
