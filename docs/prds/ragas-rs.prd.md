# ragas-rs · 产品需求文档（PRD）

> 本 PRD 由 `/s2v-prd` 生成。
>
> 注意：不要手工重命名章节标题（`/s2v-init` 按"中文｜English"双语锚点解析）。修改章节内容随时可以；改章节名会让 init 漏读字段。
>
> 解析逻辑：`## Vision` 或 `## 愿景` 任一命中即认。

**生成日期**：2026-05-31
**作者**：leafiellune
**版本**：v1.0

---

## Vision｜愿景

让 Rust 后端团队可以把 RAG/LLM 评估直接嵌入生产推理链路，用高性能、类型安全、单二进制可分发的库替代 Python ragas 在生产路径上的解释器开销、依赖体积和部署复杂度。v1.0 聚焦最小核心：可组合 Metric 抽象、OpenAI 兼容 provider、单轮数据集、异步批量 evaluate，以及 Faithfulness、ResponseRelevancy、ContextPrecision 三个内置指标。

**事实依据**：
- GitHub 项目 `vibrantlabsai/ragas` 是 Python 为主的 LLM application evaluation toolkit，README 强调 objective metrics、test data generation、integration 和 feedback loops。
- 官方文档的 metrics 目录把 Context Precision、Response Relevancy、Faithfulness 列为 RAG 场景的可用指标。
- v1.0 不追求 Python 版功能面全覆盖，优先保留生产可嵌入路径所需的核心抽象和评估闭环。

---

## 15 问自答｜Discovery Answers

1. **谁**：3-20 人 Rust/后端/平台团队，正在把 RAG 或 LLM workflow 接入线上服务，需要在 CI、灰度或推理路径旁路中持续评估输出质量。
2. **痛点**：Python ragas 适合实验和数据科学流程，但在 Rust 服务中引入 Python runtime、virtualenv、跨语言调用和容器镜像膨胀，会增加部署复杂度和线上评估延迟。
3. **现状**：团队通常用 Python 离线脚本、LangChain/Ragas notebook 或自写零散评估脚本。问题是评估链路难以和 Rust 类型系统、CI、服务指标、错误处理统一。
4. **时机**：Rust 异步生态、reqwest/tokio、OpenAI 兼容 API 和 serde 已成熟；RAG 应用开始从实验走向生产，需要把 eval 从离线报告推到可嵌入库。
5. **成功**：核心评估吞吐在基准样本上超过 Python 版 5x；库使用方可以构建单二进制，无需 Python/Node/JVM 等运行时依赖。
6. **竞品**：Python ragas、DeepEval、promptfoo、LangSmith/托管评估面板、自研评估脚本。
7. **差异**：ragas-rs 用 Rust trait、泛型和 async runtime 把评估变成嵌入式库能力，而不是外部脚本或 SaaS。
8. **反对者**：数据科学团队可能更偏好 Python notebook；平台团队可能担心 LLM-as-judge 成本和 provider 错误传播到生产路径。
9. **项目类型**：Library。
10. **技术栈倾向**：Rust 2024 edition；tokio、reqwest、serde、async-trait、thiserror。
11. **目标平台**：Linux x64、macOS arm64、Windows x64；优先服务端嵌入。
12. **性能要求**：评估吞吐超过 Python 版 5x；批量 evaluate 支持并发限制；单样本本地指标开销小于 1 ms 量级。
13. **安全/合规**：不存储 API key，不落盘样本或 provider 响应；调用方负责敏感数据脱敏，库仅做内存内传递。
14. **兼容性**：v1.0 不兼容 Python API；公开 Rust API 采用 semver，数据结构保留 serde 兼容字段。
15. **发布方式**：Cargo crate，嵌入调用方 Rust 二进制；v1.0 不提供服务端、Docker 镜像或托管面板。

---

## Problem Statement｜问题陈述

**谁有这个问题**：
Rust 后端、平台和基础设施团队，他们在 RAG/LLM 应用中负责把检索、生成、评估和回归检查接入 CI 或生产服务。

**痛点**：
Python ragas 在 notebook 和离线评估中可用，但嵌入 Rust 生产服务时需要额外 runtime、跨语言边界和部署约束。线上旁路评估一旦依赖 Python 进程，会增加容器体积、冷启动时间、错误面和观测复杂度。

**现状**：
现有方案包括直接运行 Python ragas、使用 LangSmith 等托管评估、用 promptfoo 做 prompt/eval 流程、或在服务内自写 metrics。它们要么语言/runtime 不贴合 Rust 服务，要么偏托管/CLI 流程，要么缺少可复用的 metric/provider/dataset 抽象。

**为什么是现在**：
RAG 系统进入生产后，团队不再只需要离线报告，而需要在 CI、灰度、回归测试和线上旁路中持续评估。Rust 生态已经具备稳定 async HTTP、serde 数据模型和强类型错误处理，足以承载一个最小 ragas 核心重构。

---

## Users & Context｜用户与场景

**主要用户**：
- **Rust 后端工程师**：在服务或 CLI 中调用评估库，对每次 RAG 输出计算质量分数。
- **平台/基础设施工程师**：把 evaluate 接到 CI、灰度、批处理任务或可观测性流水线。

**次要用户 / 利益相关者**：
- **数据科学/评估工程师**：定义指标和样本格式，并消费评估结果。
- **产品/安全负责人**：关心评估结果是否能发现幻觉、无关回答和低质量检索。

**关键使用场景**：
1. **CI 回归评估**：开发者提交 RAG prompt 或检索改动后，对固定 EvaluationDataset 运行 evaluate 并比较指标。
2. **线上旁路抽样评估**：服务采样用户请求和模型回复，在后台并发执行内置或自定义指标。
3. **Provider 统一封装**：团队使用 OpenAI 兼容 endpoint，同时在测试中用 mock provider 保持 deterministic。

---

## Core Capabilities｜核心能力

1. **Metric 抽象**：支持 Discrete、Numeric、Ranking 三类输出和自定义异步指标，例如用一个 closure 实现字符串包含率。
2. **LLM/Embedding provider 包装**：先支持 OpenAI 兼容 HTTP chat completions 和 embeddings，测试中可注入 mock provider。
3. **EvaluationDataset + SingleTurnSample**：表达 user input、response、retrieved contexts、reference 和 metadata。
4. **异步 evaluate() 批量执行**：对 dataset x metrics 做并发调度，返回结构化 EvaluationReport。
5. **2-3 个内置指标**：v1.0 内置 Faithfulness、ResponseRelevancy、ContextPrecision，并保证可测试、可组合、可替换 provider。

**明确不做（Out of Scope，至少列 3 项）**：
- Python 互操作、Python API 兼容层、pyo3 或 FFI bridge。
- Python ragas 全套 40+ 指标完整复刻。
- 知识图谱测试集生成、synthetic dataset generation。
- 托管面板、Web UI、长期存储、可视化分析。
- LangChain/LlamaIndex 深度集成。

---

## User Flow｜用户流程

**主流程（happy path）**：
1. 调用方构建 `EvaluationDataset<SingleTurnSample>`，包含 question、answer、contexts。
2. 调用方配置 OpenAI 兼容 LLM/Embedding provider 或测试 mock provider。
3. 调用方选择 Faithfulness、ResponseRelevancy、ContextPrecision 或自定义 Metric。
4. 调用 `evaluate(dataset, metrics, options).await`，获得每样本/每指标结果和汇总。

**异常流（≥ 2 项）**：
- **Provider HTTP 失败**：返回结构化 `RagasError::Provider`，该样本/指标记录 error，批量评估可继续处理其他项。
- **样本字段缺失**：dataset validation 返回 `RagasError::InvalidSample`，调用方能定位 sample index 和缺失字段。
- **LLM judge 输出无法解析**：metric 返回 error result 并记录原始 reason，不把不可解析结果静默当 0 分。

**边界场景（≥ 1 项）**：
- **空 dataset 或空 contexts**：空 dataset 返回空 report；需要 contexts 的 metric 对缺失 contexts 返回可诊断错误或 0 分策略，具体由 metric 文档固定。

---

## Technical Approach｜技术方案

- **项目类型**：Library。
- **技术栈**：Rust 2024 edition + tokio + reqwest + serde + async-trait + thiserror。
- **关键模块边界**（≥ 3 个，越具体越好）：
  - `dataset/`：定义 `SingleTurnSample`、`EvaluationDataset`、validation 和 dataset 迭代接口。
  - `metric/`：定义 `Metric` trait、`MetricValue`、`MetricResult`、自定义 metric helper 和内置指标。
  - `llm/`：定义 LLM/Embedding trait、OpenAI 兼容 HTTP client、request/response DTO。
  - `eval/`：定义 `evaluate()`、并发选项、error isolation、report aggregation。
- **架构风格**：模块化 Rust library，trait 分层隔离核心评估、provider IO 和 orchestration。
- **数据流（如适用）**：调用方内存样本输入 → dataset validation → evaluate 调度 metric → metric 调 provider 或本地算法 → EvaluationReport 返回；v1.0 无持久化和缓存层。

---

## Constraints｜约束

- **运行时**：Rust 1.95+；tokio async runtime 由调用方或测试启用。
- **平台**：Linux x64、macOS arm64、Windows x64。
- **性能**：评估吞吐超过 Python 版 5x；并发度可配置；无 Python/Node/JVM 运行时依赖。
- **安全**：不存储 API key；不落盘样本、prompt 或 provider 响应；错误信息避免包含 Authorization header。
- **兼容性**：不兼容 Python API；Rust crate 遵循 semver；serde 字段保持向后兼容扩展。
- **发布**：Cargo crate；回滚方式为 semver patch/yank 和调用方锁版本。

---

## Implementation Phases｜实施阶段

> `/s2v-init` 会读这张表批量生成 phase spec 和 task spec。

| # | Phase 名称（kebab）| 描述（完成后能做什么）| 范围（涉及模块 / 文件）| 依赖 | 可并行 |
|---|---|---|---|---|---|
| 1 | foundation-dataset | 创建 Rust crate、基础错误模型、EvaluationDataset 与 SingleTurnSample，可构建和校验样本 | `Cargo.toml` + `src/lib.rs` + `src/dataset.rs` + `src/error.rs` | - | 否 |
| 2 | metric-abstractions | 定义 Metric trait、MetricValue/MetricResult 和自定义 metric helper，可写类型安全指标 | `src/metric.rs` + `src/lib.rs` | 1 | 否 |
| 3 | providers | 定义 LLM/Embedding provider trait 与 OpenAI 兼容 HTTP DTO/client，可注入 mock 或真实 endpoint | `src/llm.rs` + `src/lib.rs` | 1, 2 | 否 |
| 4 | evaluator-builtins | 实现异步 evaluate 批量调度、report 汇总和三项内置 RAG 指标 | `src/eval.rs` + `src/metric.rs` + `src/llm.rs` + `src/lib.rs` | 1, 2, 3 | 否 |

---

## Decisions Log｜决策日志

> 至少 3 条；至少覆盖 S2V 8 类决策中的任 3 类。

| ID (D1, D2...) | 类别 | 决策（一句话）| 选择 | 候选方案 | 拒绝候选的理由 |
|---|---|---|---|---|---|
| D1 | 架构 | 用 trait 分层隔离 metric、provider、dataset 和 eval 调度 | Rust trait + async_trait + 模块化 library | 大型框架式 runtime / 全局 registry | v1.0 目标是可嵌入核心，registry 会增加全局状态和测试复杂度 |
| D2 | 依赖 | 基础 async/HTTP/serde 依赖采用 Rust 常见稳定组合 | tokio + reqwest + serde + async-trait + thiserror | hyper 低层手写 / ureq 同步 HTTP / 自定义 JSON | reqwest/tokio 能最快交付 async provider；手写 HTTP 会扩大 scope |
| D3 | 测试工具链 | 以 Cargo 原生命令作为唯一基线绿三件套 | cargo build / cargo check / cargo test | nextest / tarpaulin / cargo-deny | v1.0 greenfield 不引入额外测试 runtime，保持单一 Rust 工具链 |
| D4 | 协议接口 | v1.0 provider 协议以 OpenAI 兼容 HTTP 为先 | chat completions + embeddings DTO | Python ragas provider API / LangChain adapter / 多厂商 SDK | OpenAI 兼容面最常见，减少 SDK 锁定和 runtime 依赖 |
| D5 | 部署发布 | 交付形态为 Cargo library，嵌入调用方二进制 | crate library，无服务进程 | Docker service / CLI-only / 托管 API | 目标是生产路径嵌入和零额外运行时，而不是部署新服务 |

---

## Success Metrics｜成功指标

**主要指标**（Primary，≥ 1 个，必须可测量）：
- **评估吞吐**：在相同样本和 provider mock 基准下，吞吐超过 Python 版 5x。
- **运行时依赖**：调用方可构建单二进制，无 Python/Node/JVM 运行时依赖。

**次要指标**（Secondary，≥ 2 个）：
- **API 可组合性**：新增自定义 Metric 不需要修改 eval 调度核心。
- **错误可诊断性**：provider、dataset、parse、metric 错误均落到结构化 error/result。
- **测试确定性**：核心行为通过 cargo test，provider 逻辑可用 mock/DTO 测试覆盖。

**反指标**（Anti-metrics — 优化主指标时不能牺牲的，≥ 1 项）：
- 不能为追求吞吐吞掉单样本错误或丢失 metric 级诊断。
- 不能为减少依赖牺牲 OpenAI 兼容 HTTP 的可用性和 TLS 支持。

---

## Open Questions｜开放问题

> ≥ 1 项。零 open question 通常是危险信号。

- [ ] Python ragas 的指标语义存在版本漂移；v1.0 先登记为"语义启发式兼容"，后续是否需要 golden parity suite 需另开 PRD/phase。
- [ ] Faithfulness 的 LLM judge prompt 是否要和 Python ragas 完全对齐，还是保留 Rust crate 自己的 prompt contract。
- [ ] Benchmarks 对 Python 版 5x 的具体样本集和硬件环境需在 v1.1 明确。

---

## Technical Risks｜技术风险

> ≥ 3 项。

| # | 风险 | 概率 | 影响 | 缓解策略 |
|---|---|---|---|---|
| R1 | LLM-as-judge 输出不可控导致 score 解析不稳定 | 中 | 高 | 定义 JSON score parser、测试 malformed 输出、不可解析时返回结构化错误 |
| R2 | OpenAI 兼容厂商字段差异导致 DTO 不够兼容 | 中 | 中 | DTO 保留常见字段并使用 serde default；provider error 保留 status/body 摘要 |
| R3 | async 并发过高触发 rate limit 或成本失控 | 中 | 高 | evaluate options 支持 concurrency limit；provider 错误按样本隔离 |
| R4 | "超过 Python 版 5x" 需要严谨 benchmark 才能证明 | 高 | 中 | v1.0 实现可测结构和 mock provider 基准入口；PRD 登记为 release gate 假设 |
| R5 | 内置指标过度简化导致用户误以为和 Python ragas 完全等价 | 中 | 中 | 文档和类型命名声明 v1.0 语义启发式兼容，不承诺全量 parity |

---

## Next Steps｜后续步骤

1. 使用 `/s2v-init` 从本 PRD 生成 adapter、AGENTS、phase/task specs、ADR、feature 文件和 S2V 快照。
2. 按依赖顺序执行 `/s2v-implement`：task-1.1 → task-2.1 → task-3.1 → task-4.1。
3. 每个 task 保持 RED → GREEN → §9 Verification → §10 回填，不归档 task spec。

> task spec 实施完后留在原地不归档（SDD 单一事实源核心要求）。
