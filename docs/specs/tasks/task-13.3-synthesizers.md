# Task 13.3 - synthesizers

**Status**: Done
**Phase**: 13
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

persona, single-hop, multi-hop synthesizers

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
- test/features/synthesizers.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Persona generator stores name, role, and goals
- **AC2**: Single-hop synthesizer creates samples from one chunk
- **AC3**: Multi-hop synthesizer combines related graph nodes

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-13.3.1 | TEST-13.3.1 | Done |
| AC2 | SCEN-13.3.2 | TEST-13.3.2 | Done |
| AC3 | SCEN-13.3.3 | TEST-13.3.3 | Done |

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
  - `src/testset/mod.rs`（新增 Persona、PersonaGenerator、SynthesizedSample、single-hop/multi-hop synthesizer 与 TEST-13.3.1~13.3.3）
  - `src/lib.rs`（导出 task-13.3 public crate API）
- **commit 列表**：
  - `7063ff2` docs(spec): task-13.3 Ready
  - `dcfcfcf` docs(spec): task-13.3 进入实施
  - `862e81e` test(testset): 加 task-13.3 RED 测试
  - `9effd1c` feat(testset): 实现 task-13.3 synthesizers
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: 94 passed / 0 failed (`cargo test`)
  - build: ✅ `cargo build`
- **剩余风险 / 未做项**：当前 synthesizer 是 deterministic template scaffold，不调用 LLM 生成自然语言问题；Python ragas synthesizer prompt parity 和采样策略由 parity/docs task 登记或扩展。
- **下游 task 影响**：phase 14 backends/CLI 可把 SynthesizedSample 序列化或导出；phase 16 parity/docs 需要说明 deterministic template 与 Python LLM-driven synthesizer 的差异。
