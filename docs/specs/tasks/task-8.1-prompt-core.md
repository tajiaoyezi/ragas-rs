# Task 8.1 - prompt-core

**Status**: Done
**Phase**: 8
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

typed prompt template、few-shot examples、language adaptation hooks

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/prompts/mod.rs`, `src/error.rs`, and public exports in `src/lib.rs`.
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
- test/features/prompt-core.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Prompt template renders typed variables with missing-variable diagnostics
- **AC2**: Few-shot examples can be attached and serialized
- **AC3**: Language adaptation hook can rewrite prompt text deterministically

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-8.1.1 | TEST-8.1.1 | Done |
| AC2 | SCEN-8.1.2 | TEST-8.1.2 | Done |
| AC3 | SCEN-8.1.3 | TEST-8.1.3 | Done |

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
  - `src/prompts/mod.rs`（新增 typed prompt variables、PromptTemplate、FewShotExample、LanguageAdapterRule、render diagnostics 和 task 8.1 unit tests）
  - `src/error.rs`（新增 `RagasError::Prompt`）
  - `src/lib.rs`（导出 prompt core API）
  - `docs/specs/tasks/task-8.1-prompt-core.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `7623fc6` docs(spec): task-8.1 Ready
  - `43032cf` docs(spec): task-8.1 进入实施
  - `142619e` test(prompts): 加 task-8.1 RED 测试
  - `d80c590` feat(prompts): 实现 task-8.1 prompt core
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 43 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
  - note: project helper `s2v_verify_full` still cannot parse lowercase generated §9 keys, so adapter commands were executed directly and recorded here.
- **剩余风险 / 未做项**：本 task 只实现 prompt core data model/rendering；typed output parser、repair strategy 和 multimodal prompt payload 留给 task 8.2/8.3。
- **下游 task 影响**：task 8.2 can consume `RenderedPrompt` and `RagasError::Prompt` for parser diagnostics; task 9/10 metric prompts can reuse `PromptTemplate` and few-shot examples without adding Python prompt dependencies.
