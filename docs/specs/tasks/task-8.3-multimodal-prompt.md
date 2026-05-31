# Task 8.3 - multimodal-prompt

**Status**: Done
**Phase**: 8
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

image/text prompt scaffold and typed multimodal message model

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/prompts/mod.rs` and public exports in `src/lib.rs`.
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
- test/features/multimodal-prompt.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Multimodal message supports text and image parts
- **AC2**: Prompt rendering preserves part order
- **AC3**: Unsupported media returns structured error

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-8.3.1 | TEST-8.3.1 | Done |
| AC2 | SCEN-8.3.2 | TEST-8.3.2 | Done |
| AC3 | SCEN-8.3.3 | TEST-8.3.3 | Done |

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
  - `src/prompts/mod.rs`（新增 `MultimodalPromptMessage`、`MultimodalPromptPart`、ordered text scaffold rendering、unsupported media diagnostics 和 task 8.3 unit tests）
  - `src/lib.rs`（导出 multimodal prompt API）
  - `docs/specs/tasks/task-8.3-multimodal-prompt.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `60aab1f` docs(spec): task-8.3 Ready
  - `005fa30` docs(spec): task-8.3 进入实施
  - `c6acbd2` test(prompts): 加 task-8.3 RED 测试
  - `043919c` feat(prompts): 实现 task-8.3 multimodal prompt
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 49 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
  - note: project helper `s2v_verify_full` still cannot parse lowercase generated §9 keys, so adapter commands were executed directly and recorded here.
- **剩余风险 / 未做项**：本 task 提供 text/image URL prompt scaffold；binary image payloads、audio/video parts 和 provider-specific multimodal DTO conversion 留给 advanced metric/integration tasks。
- **下游 task 影响**：phase 12 multimodal metrics can reuse `MultimodalPromptMessage`; phase 16 parity fixtures can verify ordered prompt part rendering without introducing external media dependencies.
