# Task 7.2 - llm-adapters

**Status**: Done
**Phase**: 7
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

OpenAI-compatible completion polish、Azure/local-compatible config

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
- test/features/llm-adapters.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: OpenAI-compatible chat client supports base URL, model, and headers
- **AC2**: Azure-compatible config maps deployment name and API version
- **AC3**: HTTP errors are sanitized and preserve status/body summary

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-7.2.1 | TEST-7.2.1 | Done |
| AC2 | SCEN-7.2.2 | TEST-7.2.2 | Done |
| AC3 | SCEN-7.2.3 | TEST-7.2.3 | Done |

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
  - `src/llm.rs`（新增 OpenAI-compatible config、Azure config 映射、headers/query URL 支持、HTTP error sanitization 和 task 7.2 unit tests）
  - `src/lib.rs`（导出 `OpenAiCompatibleConfig` 和 `AzureOpenAiConfig`）
  - `docs/specs/tasks/task-7.2-llm-adapters.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `6168605` docs(spec): task-7.2 Ready
  - `544f3af` docs(spec): task-7.2 进入实施
  - `7c129e9` test(llm): 加 task-7.2 RED 测试
  - `076112c` feat(llm): 实现 task-7.2 llm adapters
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 37 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
  - note: project helper `s2v_verify_full` could not parse lowercase generated §9 keys, so adapter commands were executed directly and recorded here.
- **剩余风险 / 未做项**：本 task 覆盖 OpenAI-compatible/Azure adapter config 和错误脱敏；真实网络端到端 fixture、local model protocol variants 和 embedding-specific adapter details 留给 task 7.3/后续 integration task。
- **下游 task 影响**：task 7.3 可复用 `OpenAiCompatibleConfig` 的 base URL/header/query 支持实现 embedding adapters；phase 9/10 metric judge flows 可通过 `provider_http_error` 获得脱敏后的 provider failure。
