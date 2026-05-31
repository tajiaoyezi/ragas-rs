# Task 7.1 - provider-core

**Status**: Done
**Phase**: 7
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

provider registry、mock providers、usage accounting

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: src/providers.rs, src/llm.rs, src/runtime.rs, src/lib.rs.
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
- test/features/provider-core.feature

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

- **AC1**: Provider registry resolves LLM and embedding providers by name
- **AC2**: Mock providers support deterministic unit tests
- **AC3**: Provider responses carry usage accounting when available

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-7.1.1 | TEST-7.1.1 | Done |
| AC2 | SCEN-7.1.2 | TEST-7.1.2 | Done |
| AC3 | SCEN-7.1.3 | TEST-7.1.3 | Done |

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
  - `src/providers.rs`（新增 ProviderRegistry、MockLlmProvider、MockEmbeddingProvider、record_provider_usage 与 unit tests）
  - `src/lib.rs`（导出 provider core API）
  - `docs/specs/tasks/task-7.1-provider-core.md`（Status/traceability/Completion Notes 回填）
- **commit 列表**：
  - `c9bdde4` docs(spec): task-7.1 Ready
  - `0f1eedb` docs(spec): task-7.1 进入实施
  - `f9559c9` test(providers): 加 task-7.1 RED 测试
  - `fc96059` feat(providers): 实现 task-7.1 provider core
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 34 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
- **剩余风险 / 未做项**：本 task 只实现 provider registry、mock provider 和 usage 记录；OpenAI/Azure/local/http adapter 细节由 task 7.2/7.3 继续实现。
- **下游 task 影响**：task 7.2/7.3 可把具体 LLM/embedding adapters 注册进 ProviderRegistry；phase 9/10 metrics 可用 mock providers 做 deterministic testing；phase 15 benchmarks 可复用 provider usage accounting。
