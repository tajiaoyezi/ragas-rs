# 🏛️ ragas-rs Architecture

This document maps the internal structure of the `ragas` crate for contributors: the module layout, the core trait abstractions you implement or inject, how an evaluation flows from a dataset to a report, and the provider/resilience and test-set-generation subsystems. It reflects source at commit `931fda3` (branch `main`); see [../README.md](../README.md) for the user-facing overview and [parity-roadmap.md](parity-roadmap.md) for scope.

## 🗺️ Module map

The crate exposes 21 public modules (`src/lib.rs:1-21`); a flat `pub use` surface re-exports the common types so callers import `ragas::Foo` directly.

| Module / dir | Responsibility |
|---|---|
| `config` (`src/config.rs`) | Single source of truth for provider env vars (`OPENAI_API_KEY`/`_BASE_URL`/`_MODEL`, `OPENAI_EMBEDDING_*`) + defaults. Resolves env → `.env` → built-in default into `ProviderConfig`; redacts secrets. |
| `dataset` (`src/dataset.rs`) | `EvaluationDataset`, `SingleTurnSample`, `EvaluationSample`, builder; JSONL/CSV IO + column-map remapping. |
| `eval` (`src/eval.rs`) | Evaluation orchestration: `evaluate`/`evaluate_with`/`evaluate_with_config`, `EvaluationReport`, `EvaluationConfig`, per-cell fan-out (`run_scoring`). |
| `llm` (`src/llm.rs`) | Provider traits + `OpenAiCompatibleClient`, `AzureOpenAiConfig`, `EmbeddingAdapter`, response parsers. |
| `metric` (`src/metric.rs`) | The single-turn `Metric` trait + 29 single-turn metric impls + `generate_and_parse`/`fix_output_format`. |
| `metrics/` (`src/metrics/{base,result,registry}.rs` + `rag/`, `advanced/`, `traditional/` dir modules) | `SingleTurnMetric`/`MultiTurnMetric` traits, `MetricMetadata`, `DetailedMetricResult`, scoring-math primitives, `MetricRegistry`. |
| `agentic` (`src/agentic.rs`) | The 5 multi-turn metrics on `MultiTurnMetric` (tool-call + LLM-judge agentic). |
| `providers` (`src/providers.rs`) | Provider *registry/descriptor* layer: `ProviderRegistry`, `plan_provider_request`, protocol descriptors, header redaction (`<redacted>`) and `safe_debug`. Distinct from the live `llm` client. |
| `resilience` (`src/resilience.rs`) | Retry/timeout/cache/usage decorator wrappers + `CacheBackend`. |
| `runtime` (`src/runtime.rs`) | `RunConfig`, `AsyncExecutor`, `CallbackManager`/`RuntimeEvent`, `UsageTracker`/`UsageSummary`, cost helpers, lazy tokenizer. |
| `schema` (`src/schema.rs`) | `Message`/`MessageRole`, `MultiTurnSample`, `ToolCall`, `Rubric`. |
| `testset/` (`src/testset/mod.rs`) | Knowledge-graph test-set generation: graph, extractors, transforms, synthesizers, `TestsetGenerator`. |
| `optimizers/` (`src/optimizers/mod.rs`) | Prompt-optimization scaffolding: `GeneticOptimizer`, MIPROv2 trial planning, DSPy cache contract, `Optimizer` trait. |
| `prompts/` (`src/prompts/mod.rs`) | `PromptTemplate`, `JudgeOutputParser`/`ParsedJudgeOutput`, few-shot/multimodal/language-adapter scaffolding, repair strategy. |
| `validation` (`src/validation.rs`) | Pre-flight: `MetricRequirements`, `SampleField`, `validate_before_evaluate`, `validate_dataset_requirements`. |
| `error` (`src/error.rs`) | `RagasError` enum (`thiserror`). |
| `cli/` (`src/cli/mod.rs`) | Tested library CLI handlers: `CliCommand` enum, `CliRuntime`, `run_cli_command_with_provider`, contract snapshots. |
| `bin/` (`src/bin/ragas.rs`) | The thin `ragas` executable (arg parsing + file IO; delegates to `cli`). |
| `backends` (`src/backends/mod.rs`) | Dataset persistence: CSV / JSONL / in-memory / Google-Drive. |
| `benchmarks` (`src/benchmarks/mod.rs`) | Provider micro-benchmark + cost (`CostRates`, `run_provider_benchmark`). |
| `experiments` (`src/experiments/mod.rs`) | Run comparison / summary (`compare_runs`, `summarize_experiment`). |
| `integrations` (`src/integrations/mod.rs`) | Tracing export descriptors, payload redaction. |

## 🧩 Core abstractions

These traits are the semver surface (see [decisions/adr-001-trait-layering.md](decisions/adr-001-trait-layering.md)). Callers inject custom metrics and mock providers by implementing them — there is no global registry.

| Trait | File:line | Bounds + required items |
|---|---|---|
| `Metric` | `src/metric.rs:2970-2978` | `Send + Sync`. Requires `fn name(&self) -> &str` and `async fn score(&self, &SingleTurnSample) -> Result<MetricResult, RagasError>`; provides a default `fn requirements(&self) -> MetricRequirements`. This is the single-turn interface `evaluate` drives. |
| `MultiTurnMetric` | `src/metrics/base.rs:100-115` | `Send + Sync`. Requires `fn metadata(&self) -> MetricMetadata` and `async fn score_multi_turn(&self, &MultiTurnSample)`; provides a default `score_batch` (per-sample loop). |
| `LlmProvider` | `src/llm.rs:55-57` | `Send + Sync`. Requires `async fn generate(&self, LlmRequest) -> Result<LlmResponse, RagasError>`. |
| `EmbeddingProvider` | `src/llm.rs:60-62` | `Send + Sync`. Requires `async fn embed(&self, EmbeddingRequest) -> Result<EmbeddingResponse, RagasError>`. `EmbeddingAdapter<P>` (`src/llm.rs:65-132`) is a batching/normalizing wrapper that also impls this trait. |
| `CacheBackend` | `src/resilience.rs:255-266` | `Send + Sync`. Requires `fn get(&str) -> Option<String>`, `fn set(&str, &str)`, `fn len`; provides a default `is_empty`. Contract: must degrade gracefully (errors → miss / no-op). Impls: `InMemoryCacheBackend` (`:268-302`), `DiskCacheBackend` (`:317-381`). |

(There is also a companion `SingleTurnMetric` trait at `src/metrics/base.rs:82-97` — `metadata` + `score_single` + default `score_batch` — used by the `metrics/` layer.)

**Metric inventory (verified by grep):** **30** `impl Metric` (29 in `src/metric.rs` + `RougeScore` in `src/metrics/traditional/mod.rs:407`) + **5** real `impl MultiTurnMetric` in `src/agentic.rs` (`ToolCallAccuracyMetric` :63, `ToolCallF1Metric` :102, `AgentGoalAccuracyWithReferenceMetric` :254, `AgentGoalAccuracyWithoutReferenceMetric` :295, `TopicAdherenceMetric` :448) = **35 real metrics**. A 6th `impl MultiTurnMetric` at `src/metrics/base.rs:150` (`MultiTurnOnlyMetric`) is a `#[cfg(test)]` mock and is **not** counted.

## 🔀 Evaluation data flow

`src/eval.rs` exposes three entrypoints, layered from minimal to full-carrier:

- `evaluate(dataset, metrics: &[Arc<dyn Metric>], options: EvaluationOptions) -> EvaluationReport` (`eval.rs:54-77`) — minimal path; `raise_exceptions = false`, an empty `CallbackManager` (every emit a no-op), `usage = UsageSummary::default()`. Never returns `Err`.
- `evaluate_with(dataset, metrics, run_config: &RunConfig) -> EvaluationReport` (`eval.rs:231-242`) — derives `EvaluationOptions::from_run_config` (`concurrency = config.concurrency.max(1)`) and calls `evaluate`. Provider retry/timeout from `RunConfig` is applied at provider-construction time (wrap with `ResilientLlmProvider`), not inside this loop.
- `evaluate_with_config(dataset, metrics, config: &EvaluationConfig) -> Result<EvaluationReport, RagasError>` (`eval.rs:296-334`) — the full carrier. Emits `RuntimeEvent::evaluation_started` (run id `eval-{N}` from the process-wide `NEXT_RUN_ID` atomic), runs `run_scoring`, emits `evaluation_finished` on the success path, then fills `report.usage` from `config.usage_tracker`.

The async executor (`run_scoring`, `eval.rs:103-220`) spawns one `tokio::spawn` task per `(sample, metric)` cell, bounded by `Arc<Semaphore::new(concurrency.max(1))>` via `acquire_owned`. Results collect into a `Vec<Vec<Option<Result<...>>>>`; a `None` cell is a panicked/cancelled join. With `raise_exceptions = false` (the default), errors and panics become `MetricResult::failure` per cell and scoring continues (the Python `np.nan` sentinel analog). With `raise_exceptions = true`, the first error in deterministic `(sample_index, metric_index)` order is returned — a documented divergence from Python's nondeterministic first-completing error (`eval.rs:90-95`). Per-cell callbacks fire `metric_started` before scoring, then `metric_succeeded`/`metric_failed`; a panicked task that already emitted `metric_started` gets a compensating `metric_failed` from the joining side (`eval.rs:164-176`).

`column_map` is applied at **dataset construction** (e.g. `EvaluationDataset::from_jsonl_str_with_column_map`, used by the CLI at `bin/ragas.rs:111-115`), not inside `evaluate`. `RunConfig` defaults to `concurrency = 16`, `seed = 42` (`runtime.rs:180-255`); the lighter `EvaluationOptions::default` concurrency is 4.

```mermaid
flowchart TD
    DS["EvaluationDataset<br/>(column_map applied at construction)"] --> EV
    OPT["EvaluationOptions / RunConfig / EvaluationConfig<br/>raise_exceptions, concurrency"] --> EV
    EV["evaluate / evaluate_with / evaluate_with_config"] --> EX["run_scoring async executor<br/>one tokio task per (sample, metric) cell<br/>bounded by Semaphore(concurrency.max(1))<br/>per-cell failure isolation → None / MetricResult::failure"]
    EX -->|score(&SingleTurnSample)| M["Metric impl"]
    M -->|"LLM/embedding metrics"| P["LlmProvider / EmbeddingProvider<br/>(optionally Resilient/Caching/UsageRecording)"]
    P --> UT["UsageTracker → UsageSummary"]
    EX --> CB["CallbackManager / RuntimeEvent<br/>started · metric_started · succeeded/failed · finished"]
    EX --> RPT["EvaluationReport<br/>results, metric_names, usage"]
    UT --> RPT
```

The `UsageTracker` (`runtime.rs:610-645`) aggregates each `record(provider, metric, TokenUsage)` into `UsageSummary { total, by_provider, by_metric }`, populated in-pipeline by `UsageRecordingLlmProvider` sharing the same `Arc<Mutex<UsageTracker>>`. Per-token cost comes from `UsageSummary::estimated_cost(per_input_token, Option<per_output_token>)` (`runtime.rs:655-665`).

## 🔌 Provider & resilience layer

- **`OpenAiCompatibleClient`** (`llm.rs:255-485`): `generate` POSTs `chat/completions` with `{model, messages, temperature}`; `embed` POSTs `embeddings` with `{model, input}`. `AzureOpenAiConfig::into_openai_compatible_config` maps deployment + api-version into header-only auth + a query param. See [decisions/adr-004-openai-compatible-provider-protocol.md](decisions/adr-004-openai-compatible-provider-protocol.md).
- **Response parsing:** `parse_chat_response` (`llm.rs:487-535`) extracts the first choice + `usage`; if `finish_reason` is present and not in the success set (`stop|STOP|MAX_TOKENS|eos_token|end_turn`) it returns `RagasError::LlmDidNotFinish` (the Python `LLMDidNotFinishException` analog), while a *missing* `finish_reason` is treated leniently as finished. `parse_embedding_response` (`llm.rs:549-578`) sorts by `index` to preserve vector order.
- **Key redaction (two layers):** the live client's `sanitize_provider_error` (`llm.rs:359-435`) replaces the api-key → `[redacted-api-key]`, header values → `[redacted-header]`, and `Bearer <tok>` → `[redacted-bearer-token]` on every provider error. Separately, the provider/integration **descriptor** layer emits `safe_debug` strings and `<redacted>` auth headers (`providers.rs:355-418`, `integrations/mod.rs:235-295`).
- **`ResilientLlmProvider` / `ResilientEmbeddingProvider`** (`resilience.rs:116-161, 204-248`): wrap any provider with retry (exponential backoff capped at `max_backoff_ms`) + per-operation `timeout` via `run_with_resilience`; `from_run_config` adopts `RunConfig.retry` + `timeout`. A `LlmDidNotFinish` is non-transient and surfaced immediately (never retried).
- **`CachingLlmProvider` / `CachingEmbeddingProvider`** (`resilience.rs:387-431, 436-480`): memoize successful responses keyed on the serialized request via a `CacheBackend` — default `InMemoryCacheBackend`, or `with_backend(DiskCacheBackend)` (`resilience.rs:317-381`: FNV-1a-named JSON files, atomic temp+rename, key verified on read). An unserializable request bypasses the cache; an undeserializable hit falls through to the inner provider.
- **`UsageRecordingLlmProvider`** (`resilience.rs:167-201`): records each successful response's `usage` into the shared `UsageTracker` under `(provider_label, metric_label)`; a response with no usage passes through unrecorded.
- **`generate_and_parse<T>`** (`metric.rs:315-329`): generate → `parse_json::<T>`; on failure, feed the malformed output + original prompt back through `fix_output_format` (`metric.rs:334-357`, the `FixOutputFormat`/`StringIO` analog returning `{ "text": ... }`) then re-parse. Bounded to **one** repair attempt (documented divergence from Python's recursion ≤3). All LLM metrics route through this path.

## 🧪 Test-set generation pipeline

The pipeline (`src/testset/mod.rs`) builds a `KnowledgeGraph`, enriches it with LLM/embedding extractors and relationships, then runs synthesizers over it.

- **`KnowledgeGraph`** (`:76`) with `GraphNode`/`GraphEdge` (`:25-53`); node/edge values are `GraphProperty` (`Text | Number | Boolean | TextList | Vector`).
- **Extractors:** `LlmExtractor` over `LlmExtractorKind` (`Summary, Keyphrases, Title, Headlines, Ner, Themes, TopicDescription`, `:1311-1328`) reads node text → chunks → LLM → JSON-repair parse → writes a graph property; `EmbeddingExtractor` produces vectors.
- **Transforms engine:** `apply_transforms(...)` (`:2081`) runs a `GraphTransform` pipeline (extract / build / filter, with node-type filtering); `default_transforms(...)` (`:3546`) is the canonical raw-text → graph pipeline.
- **Relationship builders:** `build_cosine_relationships` (`:1668`) / `build_cosine_relationships_with`, `build_overlap_relationships` (`:1753`), and `build_chunk_relationships`.
- **Filtering / splitting:** `CustomNodeFilter` (`:1875`, LLM-scored relevance drop), `split_by_headlines` (`:826`), `split_text_into_chunks`, `extract_markdown_headings`.
- **Personas:** `generate_personas_from_kg(...)` (`:2175`) clusters KG nodes and turns the representative summary into a `Persona` via the LLM.
- **Synthesizers (3 structs):** `SingleHopSpecificSynthesizer`, `MultiHopSpecificSynthesizer`, `MultiHopAbstractSynthesizer` — each a standalone struct with its own `generate` path (the separate `Synthesizer` struct is a provider-holding helper, not a shared trait).
- **Orchestrator:** `TestsetGenerator` (`:3304`) picks the query distribution, splits the requested size across synthesizers, tags samples with `synthesizer_name`, and merges them into one dataset.

```text
raw text
   │ split_text_into_chunks / split_by_headlines
   ▼
KnowledgeGraph (GraphNode + GraphEdge)
   │ LlmExtractor (summary/themes/ner/…) + EmbeddingExtractor
   │ apply_transforms / default_transforms
   ▼
relationships: build_cosine_* / build_overlap_* / build_chunk_*
   │ CustomNodeFilter (LLM relevance drop)
   ▼
generate_personas_from_kg → Persona
   │
   ▼
Synthesizers: SingleHopSpecific · MultiHopSpecific · MultiHopAbstract
   │
   ▼
TestsetGenerator → EvaluationDataset (samples tagged synthesizer_name)
```

## 📜 Design decisions (ADRs)

The five accepted ADRs live in `docs/decisions/`:

- [decisions/adr-001-trait-layering.md](decisions/adr-001-trait-layering.md) — trait-layered module boundaries (`dataset`, `metric`, `llm`, `eval`) so callers inject custom metrics/mock providers without global registries; public traits are the semver surface.
- [decisions/adr-002-rust-async-http-dependencies.md](decisions/adr-002-rust-async-http-dependencies.md) — standardize on `tokio`, `reqwest`, `serde`, `async-trait`, `thiserror` for async HTTP/JSON (reject raw `hyper`, sync `ureq`, custom JSON); callers need a tokio runtime.
- [decisions/adr-003-cargo-native-test-toolchain.md](decisions/adr-003-cargo-native-test-toolchain.md) — plain `cargo build`/`check`/`test` as the baseline green suite for v1.0; coverage and lint tooling explicitly N/A.
- [decisions/adr-004-openai-compatible-provider-protocol.md](decisions/adr-004-openai-compatible-provider-protocol.md) — OpenAI-compatible HTTP chat-completions + embeddings DTOs first, no vendor SDK lock-in; vendor adapters added later without changing base traits.
- [decisions/adr-005-cargo-library-release-model.md](decisions/adr-005-cargo-library-release-model.md) — ship as a Cargo library crate embeddable into downstream binaries; no server, Docker image, or hosted panel in v1.0.

## 🚫 Non-goals

This crate targets **functional** parity (each metric exists, works, and passes a live discrimination gate), **not numeric** parity with Python ragas. Byte-exact agreement is explicitly out of scope: NumPy RNG, Python rounding, and tiktoken bin-boundary parity are non-goals, as are cross-provider robustness guarantees and a set of deferred/infeasible items (FaithfulnessWithHHEM, multimodal metrics, DSPy/MIPROv2, the real 4-LLM-stage `GeneticOptimizer`, framework integrations, cloud backends). See [parity-roadmap.md](parity-roadmap.md) for the full scope, deferred tail, and rationale, and [live-verification/results.md](live-verification/results.md) for what the 37 live gates (22 LLM-metric + 14 end-to-end/testset + 1 runtime) do and do not prove.

> This is an early (0.1) library wired to a single OpenAI-compatible provider family; scores are this crate's own, not numeric reproductions of Python ragas.
