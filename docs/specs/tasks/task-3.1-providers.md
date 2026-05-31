# Task 3.1 - providers

**Status**: Done
**Phase**: 3 - providers
**PRD**: docs/prds/ragas-rs.prd.md

## 1. Background

The PRD requires LLM and embedding provider wrappers with initial OpenAI-compatible HTTP support. Built-in metrics need these traits but tests must avoid network access.

## 2. Goal

Define provider traits, request/response DTOs, OpenAI-compatible clients, and deterministic parsing helpers.

## 3. Scope And Out-of-Scope

**In scope**:
- Add `src/llm.rs`.
- Define `LlmProvider` and `EmbeddingProvider`.
- Define OpenAI-compatible chat and embedding request/response DTOs.
- Implement request builders and response parsers.
- Implement HTTP client methods using reqwest without storing API keys.

**Out of scope**:
- Retrying/rate-limit backoff.
- Streaming chat completions.
- Provider SDK integrations beyond OpenAI-compatible HTTP.

## 4. Actors

- Built-in metrics requesting LLM judgement or embeddings.
- Tests injecting mock provider implementations.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/specs/tasks/task-1.1-foundation-dataset.md
- docs/specs/tasks/task-2.1-metric-abstractions.md
- docs/decisions/adr-002-rust-async-http-dependencies.md
- docs/decisions/adr-004-openai-compatible-provider-protocol.md
- test/features/llm.feature

### 5.2 Imports

Uses `RagasError` from phase 1 and async-trait for provider traits.

### 5.3 Function Signatures

- `#[async_trait] pub trait LlmProvider { async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, RagasError>; }`
- `#[async_trait] pub trait EmbeddingProvider { async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError>; }`
- `OpenAiCompatibleClient::new(base_url: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self`
- `OpenAiCompatibleClient::with_embedding_model(self, model: impl Into<String>) -> Self`
- `parse_chat_response(body: &str) -> Result<LlmResponse, RagasError>`
- `parse_embedding_response(body: &str) -> Result<EmbeddingResponse, RagasError>`

## 6. Acceptance Criteria

- **AC1**: Chat response parser extracts assistant content and usage from OpenAI-compatible JSON.
- **AC2**: Embedding response parser extracts vectors in response order.
- **AC3**: OpenAI-compatible request builders use bearer auth internally but provider errors do not expose the API key.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-3.1.1 | TEST-3.1.1 | Done |
| AC2 | SCEN-3.1.2 | TEST-3.1.2 | Done |
| AC3 | SCEN-3.1.3 | TEST-3.1.3 | Done |

## 8. Risks

- Provider JSON schema drift can break parsing.
- HTTP error text may include sensitive information if not sanitized.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：2026-05-31
- **改动文件**：
  - `Cargo.toml`（新增 reqwest 和 serde_json）
  - `Cargo.lock`（更新锁文件）
  - `src/error.rs`（新增 Provider 和 Parse 错误）
  - `src/lib.rs`（导出 provider API）
  - `src/llm.rs`（新增 provider traits、OpenAI-compatible client、DTO parser、unit tests）
- **commit 列表**：
  - `7499961` docs(spec): task-3.1 Ready
  - `badea1f` docs(spec): task-3.1 进入实施
  - `12fc7a6` test(llm): 加 task-3.1 RED 测试
  - `87321d6` feat(llm): 实现 task-3.1 OpenAI 兼容 provider
- **§9 Verification 结果**：
  - install: pass (`cargo build`)
  - typecheck: pass (`cargo check`)
  - unit-test: 9 passed / 0 failed (`cargo test`)
  - build: pass (`cargo build`)
- **剩余风险 / 未做项**：无；真实 provider endpoint 兼容性仍需后续集成样本登记
- **下游 task 影响**：task-4.1 可使用 `LlmProvider` 和 `EmbeddingProvider` 实现内置指标
