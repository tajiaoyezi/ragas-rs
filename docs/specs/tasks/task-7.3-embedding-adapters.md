# Task 7.3 - embedding-adapters

**Status**: Done
**Phase**: 7
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

OpenAI-compatible embeddings、batching、normalization

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `src/llm.rs` and public exports in `src/lib.rs`.
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
- test/features/embedding-adapters.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Embedding provider batches inputs without reordering outputs
- **AC2**: Optional vector normalization is deterministic
- **AC3**: Embedding errors include request batch position

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-7.3.1 | TEST-7.3.1 | Done |
| AC2 | SCEN-7.3.2 | TEST-7.3.2 | Done |
| AC3 | SCEN-7.3.3 | TEST-7.3.3 | Done |

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
  - `src/llm.rs`（新增 generic `EmbeddingAdapter`、batching、optional L2 normalization、batch-position error wrapping 和 task 7.3 unit tests）
  - `src/lib.rs`（导出 `EmbeddingAdapter` 和 `normalize_embedding_vector`）
  - `docs/specs/tasks/task-7.3-embedding-adapters.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `993f157` docs(spec): task-7.3 Ready
  - `21b6492` docs(spec): task-7.3 进入实施
  - `5128ef1` test(llm): 加 task-7.3 RED 测试
  - `98b39ec` feat(llm): 实现 task-7.3 embedding adapters
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 40 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
  - note: project helper `s2v_verify_full` has the same lowercase generated §9 key parsing issue seen in task 7.2; adapter commands were executed directly and recorded here.
- **剩余风险 / 未做项**：本 task 提供 generic embedding batching/normalization wrapper；真实 HTTP embedding batching 的 rate-limit/backoff 策略和 provider-specific fixture 仍由后续 integration/parity tasks 细化。
- **下游 task 影响**：phase 9/10 metric implementations can wrap mock or HTTP embedding providers with `EmbeddingAdapter` for deterministic batch behavior; phase 15 benchmarks can measure normalization and batching overhead separately from provider latency.
