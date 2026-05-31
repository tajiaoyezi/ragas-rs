# ragas-rs complete refactor · 产品需求文档（PRD）

> 本 PRD 是 `docs/prds/ragas-rs.prd.md` 的完整重构扩展版。原 PRD 只覆盖 Rust core MVP；本文件把目标扩展为对 upstream `vibrantlabsai/ragas` 的完整模块级重构路线。
>
> Source baseline: `vibrantlabsai/ragas` commit `298b682`，采样时间 2026-05-31。

**生成日期**：2026-05-31
**作者**：leafiellune
**版本**：v2.0

---

## Vision｜愿景

把 Python ragas 的评估框架完整重构为 Rust-first、类型安全、可嵌入、可按 feature 裁剪的 RAG/LLM evaluation toolkit。最终交付不再只是 `dataset/metric/llm/eval` MVP，而是覆盖 evaluation runtime、dataset/backends、provider adapters、prompt/judge parser、完整 metric catalog、testset generation、optimizer、CLI、docs/examples、parity verification 和 release packaging 的完整项目。

本 PRD 的成功标准不是“能跑 3 个指标”，而是让 Rust 版本具备可演进到 Python ragas 功能等价的模块拓扑、任务边界和验证闭环。实现中允许按 feature gate 拆分非核心能力，但每个 upstream 模块必须有对应 Rust phase/task、scope 决策和 parity 计划。

---

## Problem Statement｜问题陈述

**谁有这个问题**：
Rust 后端团队、RAG 平台团队、评估平台工程师，以及希望把 ragas 从 Python 离线工具迁移到生产级 Rust 服务或 Rust CLI 的团队。

**痛点**：
当前仓库已完成一个最小核心，但它没有覆盖 Python ragas 的真实项目面：dataset schema、executor/run config、cost/cache/callbacks、prompt 模板、完整 metrics、testset generation、backends、integrations、optimizers、CLI 和 examples 都未进入 S2V task。继续基于 4 个 task 声称“完整重构”会导致范围严重失真。

**现状**：
upstream Python ragas 以 `src/ragas` 为核心，包含 top-level runtime 文件、`backends/`、`embeddings/`、`integrations/`、`llms/`、`metrics/`、`optimizers/`、`prompt/`、`testset/` 等目录，并有大量 docs/howtos/examples。当前 Rust repo 只实现了 `src/dataset.rs`、`src/metric.rs`、`src/llm.rs`、`src/eval.rs` 的 MVP 子集。

**为什么是现在**：
现有 MVP 已把 Rust crate、trait layering、OpenAI-compatible provider、async evaluate 和三项指标跑通，可以作为完整重构的 foundation。现在必须扩展 S2V master spec，否则后续 implementation 会继续围绕过小任务集局部优化。

---

## Users & Context｜用户与场景

**主要用户**：
- **Rust RAG 平台工程师**：需要把 ragas 评估能力嵌入线上服务、CI 或批处理。
- **评估框架维护者**：需要用 S2V 逐模块迁移 Python ragas 能力，并跟踪 parity gap。
- **应用团队开发者**：通过 Rust crate 或 CLI 运行 eval、生成 testset、比较模型和 prompt。

**次要用户 / 利益相关者**：
- **数据科学团队**：需要保留 ragas 指标语义、prompt 可调性和数据集可读性。
- **安全/平台负责人**：需要可审计 provider 调用、成本统计、缓存和错误隔离。
- **文档和 examples 维护者**：需要 Rust examples 与 upstream howtos 对齐。

**关键使用场景**：
1. **完整 RAG eval**：加载 dataset，配置 LLM/embedding provider，运行多指标 evaluate，导出 report。
2. **指标目录迁移**：逐批启用 answer/context/faithfulness/rubrics/agent/tool/sql/summarization 等指标。
3. **testset generation**：从文档构建知识图谱、persona、single-hop/multi-hop synthesizer，生成评估样本。
4. **生产化运行**：配置 run config、timeout/retry/cache/cost/callbacks，集成 tracing/backend。
5. **parity 验证**：用 golden fixtures 对 Rust 输出和 Python ragas baseline 做语义差异登记。

---

## Core Capabilities｜核心能力

1. **完整数据模型与 backends**：SingleTurn/MultiTurn/Message/ToolCall/EvaluationDataset、JSONL/CSV/in-memory backend、schema validation。
2. **执行 runtime**：sync/async evaluate、executor、run config、retry/timeout/cancellation、callbacks、cost usage、cache。
3. **provider/prompt 层**：LLM/Embedding traits、OpenAI-compatible base、provider adapters、prompt template、few-shot、judge output parser、多模态 prompt scaffold。
4. **完整 metric catalog**：按 upstream 分类迁移 deterministic、embedding、LLM judge、RAG context、answer quality、rubrics、agent/tool/sql/multimodal/summarization metrics。
5. **testset generation 与优化器**：knowledge graph、transforms、extractors、splitters、relationship builders、persona、single/multi-hop synthesizers、prompt/model optimizer。
6. **CLI、docs/examples、parity/release**：Rust CLI、examples、feature flags、benchmark/parity suite、release packaging。

**明确不做（Out of Scope）**：
- 不做 Python runtime 依赖或 pyo3 直接桥接。
- 不承诺每个 Python integration 都有 Rust SDK 等价；外部生态 integration 以 feature gate 和 protocol adapter 方式分层。
- 不在核心 crate 内实现托管 Web dashboard。
- 不把 provider API key、dataset 或 tracing payload 自动落盘。

---

## Technical Approach｜技术方案

- **项目类型**：Rust workspace / library + optional CLI。
- **技术栈**：Rust 2024, tokio, reqwest, serde, async-trait, thiserror; 后续按 task 引入 clap、csv、jsonl、tracing、tokenizer、feature-gated optional crates。
- **关键模块边界**：
  - `src/schema/`：samples、messages、tool calls、dataset schema、validation。
  - `src/runtime/`：executor、run config、callbacks、cost、cache、cancellation。
  - `src/providers/`：LLM/embedding traits、OpenAI-compatible HTTP、adapter registry、mock providers。
  - `src/prompts/`：prompt templates、few-shot、typed output parsing、judge JSON parser。
  - `src/metrics/`：metric traits、result model、validators、metric collections。
  - `src/testset/`：knowledge graph、transforms、synthesizers、persona。
  - `src/backends/`：in-memory、local JSONL/CSV、optional external backends。
  - `src/cli/`：evaluate/testset/benchmark commands。
  - `tests/parity/`：golden fixtures and upstream parity matrix.
- **架构风格**：workspace-ready modular monolith，core traits in library, optional capabilities behind features.
- **数据流**：source dataset/backend -> schema validation -> runtime executor -> metric/provider/prompt/cache/callbacks -> report/backend/export -> parity/benchmark assertions.

---

## Constraints｜约束

- **运行时**：Rust 1.95+；async paths use tokio.
- **平台**：Linux x64, macOS arm64, Windows x64.
- **性能**：mock-provider evaluate throughput target remains > Python baseline 5x; provider-bound metrics expose concurrency and retry controls.
- **安全**：no credential persistence; provider errors redact auth; tracing/callback payload filtering documented.
- **兼容性**：Python API is not binary-compatible; semantic parity tracked by fixtures and task-level parity status.
- **发布**：Cargo library, optional `ragas` CLI behind feature, semver release with feature gates.

---

## Implementation Phases｜实施阶段

| # | Phase 名称（kebab）| 描述（完成后能做什么）| 范围（涉及模块 / 文件）| 依赖 | 可并行 |
|---|---|---|---|---|---|
| 0 | core-mvp-completed | 当前 4-task MVP 已完成，作为完整重构 foundation | `src/dataset.rs` + `src/metric.rs` + `src/llm.rs` + `src/eval.rs` | - | 否 |
| 1 | schema-and-datasets | 完整样本、消息、tool call、多轮数据集、serde schema 与 validation | `src/schema/` + `src/dataset.rs` | 0 | 否 |
| 2 | runtime-executor | executor、run config、retry、timeout、cancellation、callbacks、cost、cache | `src/runtime/` + `src/eval.rs` | 1 | 否 |
| 3 | providers-and-adapters | LLM/embedding provider matrix、adapter registry、mock/local/http providers | `src/providers/` + `src/llm.rs` | 1,2 | 是（可与 phase 4 并行） |
| 4 | prompts-and-parsers | prompt templates、few-shot、typed output parser、judge JSON parser、多模态 prompt scaffold | `src/prompts/` | 1,2 | 是（可与 phase 3 并行） |
| 5 | metric-framework-complete | metric base、validators、result schema、metric collection registry、parity labels | `src/metrics/base.rs` + `src/metrics/result.rs` + `src/metrics/validators.rs` | 1,2,4 | 否 |
| 6 | rag-metrics | faithfulness/context/answer/factual/noise/RAG 指标全批次迁移 | `src/metrics/rag/` | 5,3 | 是（可与 phase 7 并行） |
| 7 | deterministic-and-similarity-metrics | BLEU/ROUGE/CHRF/string/semantic similarity/classic metrics | `src/metrics/traditional/` | 5,3 | 是（可与 phase 6 并行） |
| 8 | advanced-metrics | rubrics、agent、tool call、SQL、多模态、summarization metrics | `src/metrics/advanced/` | 5,3,4 | 否 |
| 9 | testset-generation | graph、transforms、extractors、splitters、relationship builders、persona、single/multi-hop synthesizers | `src/testset/` | 1,3,4 | 否 |
| 10 | backends-integrations-cli | JSONL/CSV/in-memory backend、optional integrations、CLI evaluate/testset/benchmark | `src/backends/` + `src/integrations/` + `src/cli/` | 2,5,9 | 否 |
| 11 | optimizers-experiments | experiment model、prompt/model optimizer、benchmark llm/embedding flows | `src/experiments/` + `src/optimizers/` | 5,10 | 是（可与 phase 12 并行） |
| 12 | parity-docs-release | upstream parity fixtures、docs/examples、feature flags、release packaging | `tests/parity/` + `examples/` + `docs/` | 6,7,8,9,10 | 否 |

---

## Complete Task Matrix｜完整任务矩阵

| Task | Phase | 模块 | 目标 |
|---|---|---|---|
| 1.1 | schema-and-datasets | schema-core | MultiTurnSample、Message、ToolCall、rubric/reference/metadata schema |
| 1.2 | schema-and-datasets | dataset-io | JSONL/CSV serde roundtrip、dataset builders、validation diagnostics |
| 1.3 | schema-and-datasets | validation | sample/metric compatibility validator、required column checker |
| 2.1 | runtime-executor | run-config | timeout/retry/concurrency/cancellation model |
| 2.2 | runtime-executor | executor | ordered async executor、partial failure isolation、progress events |
| 2.3 | runtime-executor | callbacks-cost-cache | callbacks、token usage/cost model、cache key/value abstraction |
| 3.1 | providers-and-adapters | provider-core | provider registry、mock providers、usage accounting |
| 3.2 | providers-and-adapters | llm-adapters | OpenAI-compatible completion polish、Azure/local-compatible config |
| 3.3 | providers-and-adapters | embedding-adapters | OpenAI-compatible embeddings、batching、normalization |
| 4.1 | prompts-and-parsers | prompt-core | typed prompt template、few-shot examples、language adaptation hooks |
| 4.2 | prompts-and-parsers | output-parser | JSON/schema parser、repair strategy、malformed output diagnostics |
| 4.3 | prompts-and-parsers | multimodal-prompt | image/text prompt scaffold and typed multimodal message model |
| 5.1 | metric-framework-complete | metric-base | full metric traits: single/multi-turn, LLM/embedding requirements, batch hooks |
| 5.2 | metric-framework-complete | metric-result | result schema, score normalization, reason/evidence, error taxonomy |
| 5.3 | metric-framework-complete | metric-registry | metric collection registry, feature flags, parity status labels |
| 6.1 | rag-metrics | context-metrics | context precision/recall/entity recall/relevance variants |
| 6.2 | rag-metrics | faithfulness-family | faithfulness, response groundedness, factual correctness |
| 6.3 | rag-metrics | answer-quality | answer relevancy/correctness/similarity/noise sensitivity |
| 7.1 | deterministic-and-similarity-metrics | lexical | exact match/string distance/BLEU/ROUGE/CHRF |
| 7.2 | deterministic-and-similarity-metrics | semantic | embedding similarity and thresholded semantic metrics |
| 7.3 | deterministic-and-similarity-metrics | quoted-spans | quoted spans and citation overlap metrics |
| 8.1 | advanced-metrics | rubrics | aspect critic, simple criteria, domain/instance rubrics |
| 8.2 | advanced-metrics | agents-tools | goal accuracy, tool call accuracy, tool call F1, topic adherence |
| 8.3 | advanced-metrics | sql-multimodal-summary | SQL semantic equivalence, multimodal faithfulness/relevance, summarization |
| 9.1 | testset-generation | graph-core | knowledge graph node/edge model and graph queries |
| 9.2 | testset-generation | transforms | splitters, extractors, filters, relationship builders |
| 9.3 | testset-generation | synthesizers | persona, single-hop, multi-hop synthesizers |
| 10.1 | backends-integrations-cli | backends | in-memory, JSONL, CSV backend registry |
| 10.2 | backends-integrations-cli | integrations | tracing hooks and optional LangSmith/Langfuse/Opik-style adapters |
| 10.3 | backends-integrations-cli | cli | `ragas evaluate`, `ragas testset`, `ragas benchmark` |
| 11.1 | optimizers-experiments | experiments | experiment record model, compare runs, report summaries |
| 11.2 | optimizers-experiments | optimizers | prompt/model optimization abstractions and genetic optimizer scaffold |
| 11.3 | optimizers-experiments | benchmarks | LLM/embedding benchmark runner and cost summaries |
| 12.1 | parity-docs-release | parity-suite | upstream golden fixtures, gap matrix, parity status reports |
| 12.2 | parity-docs-release | docs-examples | Rust examples mapped to upstream howtos/tutorials |
| 12.3 | parity-docs-release | release | feature flags, crate metadata, CI gates, release checklist |

---

## Decisions Log｜决策日志

| ID (D1, D2...) | 类别 | 决策（一句话）| 选择 | 候选方案 | 拒绝候选的理由 |
|---|---|---|---|---|---|
| D1 | 架构 | 完整重构采用 workspace-ready modular monolith | core crate + feature-gated modules | 多 crate 立即拆分 / 单文件 MVP 延续 | 立即多 crate 增加发布复杂度；继续 MVP 结构无法承载完整模块面 |
| D2 | 兼容性 | 用 semantic parity matrix 跟踪 Python ragas 对齐 | task 级 parity status + golden fixtures | 声称完全 API 兼容 | Python API 和 Rust API 模型差异大，API 兼容会扭曲 Rust 设计 |
| D3 | 依赖 | 非核心能力按 feature gate 引入依赖 | minimal default + optional features | all-in default dependency set | 完整 ragas 功能依赖面很大，默认全开会破坏单二进制轻量目标 |
| D4 | 协议接口 | provider/integration 以 trait + adapter registry 表达 | typed traits and registry | 直接绑定外部 SDK 类型 | 外部 SDK 版本漂移会污染核心 API |
| D5 | 测试工具链 | 每批迁移必须有 unit + parity fixture | cargo test + tests/parity fixtures | 只做单元测试 | 完整重构需要证明语义迁移，不只证明 Rust 代码能跑 |
| D6 | 部署发布 | CLI 是 optional binary，不是核心 runtime 前置 | `cli` feature + library-first | CLI-first rewrite | 生产嵌入是主目标，CLI 只做操作入口 |

---

## Success Metrics｜成功指标

**主要指标**：
- **范围覆盖**：upstream `src/ragas` top-level 文件和 8 个主要目录全部有 phase/task 映射。
- **任务闭环**：Complete Task Matrix 中每个 task 最终 Status=Done，且 §10 无占位。
- **验证闭环**：root `cargo build`、`cargo test`、parity suite 全部通过。

**次要指标**：
- metric catalog 至少覆盖 upstream docs 中所有 available metrics 类别。
- default feature 仍保持可嵌入，不引入 Python/Node/JVM runtime。
- provider-bound metrics 支持 mock provider deterministic testing。

**反指标**：
- 不能为了“完整”把所有外部集成硬塞进默认依赖。
- 不能把未验证 parity 的指标标成 parity-complete。
- 不能通过降低 S2V task 粒度来隐藏风险。

---

## Open Questions｜开放问题

- [ ] 每个 Python integration 是否都需要 Rust 等价，还是只提供 protocol-level adapter 和 docs mapping。
- [ ] 完整 metric parity 的 golden dataset 从 upstream 哪个版本冻结，是否固定为 commit `298b682`。
- [ ] 多模态指标需要选择 Rust image/audio payload 表达，可能需要单独 ADR。
- [ ] testset graph 是否使用 `petgraph`，还是先自实现轻量 graph model。

---

## Technical Risks｜技术风险

| # | 风险 | 概率 | 影响 | 缓解策略 |
|---|---|---|---|---|
| R1 | upstream Python ragas 模块面持续变化，完整范围可能漂移 | 高 | 高 | 冻结 baseline commit，后续变更另开 delta PRD |
| R2 | 完整 metrics 依赖大量 LLM prompt/parser 语义，Rust 实现可能与 Python 结果偏差 | 高 | 高 | 每个 metric task 要有 golden fixture 和 parity status |
| R3 | optional integrations 可能引入过重依赖或不稳定 SDK | 中 | 中 | feature gate + adapter trait + integration-specific ADR |
| R4 | testset generation 涉及 graph/transforms/synthesizers，scope 比 core eval 大很多 | 高 | 高 | 独立 phase 9，并先交付 graph core 再 synthesizer |
| R5 | CLI/docs/examples 容易变成“最后补文档”而脱离实现 | 中 | 中 | phase 12 显式列 task，examples 必须跑 cargo test 或 smoke |
| R6 | 继续在单个文件中累加代码会快速失控 | 中 | 高 | 从 phase 1 开始迁移到 `schema/ runtime/ providers/ prompts/ metrics/ testset/` 目录结构 |

---

## Next Steps｜后续步骤

1. 把本 PRD 作为新的 master scope，更新 `docs/s2v-adapter.md` 的 phase/task 索引。
2. 生成 phase 1-12 的 phase specs 和 task specs；原 4 个 task 保留 Done，映射为 phase 0 foundation。
3. 从 task 1.1 `schema-core` 开始执行 S2V RED/GREEN/§9/§10。

