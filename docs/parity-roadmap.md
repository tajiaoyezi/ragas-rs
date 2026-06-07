# ragas-rs → Python `ragas` functional-parity roadmap

A prioritized plan to maximize **functional replication** of the Python
[`ragas`](https://github.com/explodinggradients/ragas) library in Rust.

> Generated 2026-06-04 from a research pass (current ragas catalog from docs + GitHub,
> ragas-rs surface read from the actual code) followed by an adversarial completeness/correctness
> review. Baseline numbers were verified directly against the source.

**Status:** Phase 1 ✅; Phase 2 ✅ (complete, incl. ID-based context precision/recall); Phase 3 ✅
(7/7) — all LLM metrics live-verified vs DeepSeek; `context_utilization` wired into the
`ragas evaluate` CLI default. Phase 4 ✅ (7/7) — DataCompy + deterministic tool-call metrics plus the
LLM agentic metrics (`AgentGoalAccuracy` with/without reference, `TopicAdherence`,
`InstanceSpecificRubrics`), all live-verified vs DeepSeek (2026-06-06). Phase 5 ✅ (parity-complete with an honest N/A/deferred tail — closed PR #22; retry/timeout +
caching decorators incl. a persistent `DiskCacheBackend`; all 3 critical bugs fixed — incl. the FixOutputFormat repair half (PR #20); plus `evaluate()` options — `column_map`,
`raise_exceptions`, `usage_summary` on `EvaluationReport`, per-token cost API, optional tiktoken
token counts; `LLMDidNotFinish` truncation detection; lifecycle callbacks). Added
`SemanticSimilarityMetric` (ragas `answer_similarity` — embedding cosine of response vs reference,
optional threshold; live-verified vs SiliconFlow embeddings 2026-06-06). Wired count
**30 single-turn `Metric`** of ~39 +
**5 multi-turn metrics** (tool-call ×2, agent-goal-accuracy ×2, topic-adherence) (2026-06-06); all on
`main`. `StringSimilarityMetric` now exposes a full `DistanceMeasure` selector
(Levenshtein/Hamming/Jaro/Jaro-Winkler, rapidfuzz-verified) — the last purely-deterministic parity
gap (2026-06-06); the metric count is **unchanged** for that one (a variant on an existing `Metric`).
**Phase 6 started 2026-06-06** — the first real LLM testset extractors landed: `LlmExtractor` +
`LlmExtractorKind` (Summary/Keyphrases/Title/Headlines/NER/Themes/TopicDescription), a faithful port
of Python's seven `LLMBasedExtractor` subclasses, each driving a real `LlmProvider::generate` and
parsing via the shared outermost-`{ .. }` JSON-repair path; plus `extract_bundle` (sequential
NER→Themes→Summary) which finally wires the previously hand-fed `ExtractionBundle`/`attach_extractions`
substrate to a live model. Live-verified vs DeepSeek (`live_llm_extractor_pulls_named_entities_and_summary`).
Then the embedding side of Phase 6: `EmbeddingExtractor` (embed a node's text → new `GraphProperty::Vector`
property; errors on missing/non-text per Python) + `build_cosine_relationships` (faithful `CosineSimilarityBuilder`
— pairwise cosine over embedded nodes, `cosine_similarity` edges above a threshold). Live-verified vs SiliconFlow
embeddings (`live_cosine_relationships_link_semantically_similar_nodes`). Then `build_overlap_relationships`
(faithful `OverlapScoreBuilder` — the default-pipeline entity-overlap builder: per-pair fuzzy Jaro-Winkler
entity matching with the top-5% "noisy item" exclusion → directed `entities_overlap` edges; reuses the
Phase-2 `string_distance_similarity_with`; deterministic, hand-computed tests). The relationship layer
(cosine + overlap) now has both edge types the multi-hop synthesizers traverse.

## Goal & non-goals

- **Goal — functional parity:** each ragas metric/feature *exists and works* in Rust (and passes a
  live discrimination test against a real LLM).
- **Non-goal — numeric parity:** we do **not** chase byte-exact agreement with Python. NumPy RNG,
  Python rounding, and tiktoken bin-boundary parity are explicitly out of scope. Treat ragas-rs
  scores as this library's own.
- **Hard-infeasible in Rust (honest tail):** downloaded-model metrics (HHEM, cross-encoder NLI),
  multimodal metrics (need a vision-provider abstraction), DSPy/MIPROv2 (Python-only ecosystem),
  live framework integrations (LangChain/LlamaIndex), and cloud backends (real Google Drive API).

## Honest baseline (verified, not advertised)

Python ragas exposes **~39 metric classes**. ragas-rs today has **5 metrics actually wired through
the `Metric` trait** (i.e. usable in `evaluate()`):

| Metric | Location |
|---|---|
| `FaithfulnessMetric` | `src/metric.rs:105` |
| `ResponseRelevancyMetric` (Answer Relevancy) | `src/metric.rs:272` |
| `ContextPrecisionMetric` (LLM, with reference) | `src/metric.rs:399` |
| `LlmContextRecallMetric` | `src/metric.rs:519` |
| `RougeScore` (rouge-L recall) | `src/metrics/traditional/mod.rs:332` |

**Correction to an automated inventory:** a first machine pass reported "~34 real metrics." That
**over-counted** — it treated free functions, math helpers, `*_from_vectors` variants, and
judge-contract stubs as finished metrics. Ground truth: `grep "impl Metric for" src` returns
exactly **5**. This over-count is the same fake-completeness failure this project was rescued from;
the roadmap below is built on the verified 5, not the inflated 34.

**What *does* exist and lowers future cost** — a real but un-wired substrate of helpers:
`factual_correctness` F1 math, `answer_correctness` weighting, cosine / `semantic_similarity_batch`,
BLEU-unigram, char-unigram CHRF, the AP@k accumulator, Levenshtein, `context_precision_from_relevance`,
JSON-repair parsing. Many missing metrics are "add the LLM orchestration + a `Metric` wrapper around
math that's already here," not greenfield.

**Advertised-but-stub subsystems (treat as NOT done):** DSPy/MIPRO (metadata-only contracts),
`GeneticOptimizer` (a generic caller-driven GA, *not* the ragas 4-LLM-stage algorithm),
`PersonaGenerator` (manual seed strings, not KG-derived), Google Drive backend (in-memory fake
transport).

## Execution rules (anti-gaming — mandatory)

1. Every new metric ships as a real `impl Metric` **plus** an env-gated `#[ignore]` **live
   discrimination test** (a good sample scores high, an adversarial one scores low). No
   `complete` / `parity` / `bug-zero` labels until that test has passed against a real LLM.
2. Where Python has both an LLM and a non-LLM variant, keep the existing deterministic lexical
   function as a no-LLM **fallback** — don't delete it when adding the LLM version.
3. Don't chase NumPy RNG / rounding / tiktoken parity. Document any deliberate divergence.

---

## Phase 1 — LLM metric variants that clone existing scaffolding ✅ DONE

*Highest value-to-effort. Each reuses a working LLM metric's machinery (provider, JSON-repair parse,
AP@k accumulator, `Metric` trait). No new subsystems.*

> ✅ **Shipped 2026-06-04** — all six are real `impl Metric` in `src/metric.rs`
> (`ContextUtilizationMetric`, `AspectCriticMetric`, `SimpleCriteriaScoreMetric`,
> `SqlSemanticEquivalenceMetric`, `AnswerCorrectnessMetric`, `AnswerAccuracyMetric`), exported from
> `lib.rs`, with 10 offline unit tests + 6 `#[ignore]` live discrimination gates **all passing**
> against DeepSeek + SiliconFlow. Not yet wired into the `ragas evaluate` CLI path (deliberate
> follow-up — the CLI still runs only ROUGE + faithfulness + context_recall).

| Metric | Effort | Value | Approach |
|---|---|---|---|
| **ContextPrecisionWithoutReference / ContextUtilization** | S | high | Clone `ContextPrecisionMetric`, drop the reference, judge usefulness vs the **response**; reuse the AP@k accumulator. The single most valuable gap (no ground truth needed — the common production case). |
| **AspectCritic** | S | high | Single-call binary LLM judge over a user criterion. Reuse existing `score_aspect_critic` normalizer. Simpler than the working Faithfulness 2-step. |
| **SimpleCriteriaScore** | S | medium | Same single-call skeleton as AspectCritic; returns an integer (needs a raw/unnormalized numeric `MetricValue`). |
| **SQLSemanticEquivalence (LLM judge)** | M | high | Add the actual explain-then-judge LLM call (the Rust fn only normalizes strings today); keep normalized exact-match as a pre-check short-circuit. |
| **AnswerCorrectness (orchestration)** | M | high | Math + cosine + `FactualCorrectnessCounts` already exist; add the LLM TP/FP/FN statement classifier + embedding similarity, feed the existing `answer_correctness()` formula (default 0.75/0.25). |
| **AnswerAccuracy (Nvidia)** | M | medium | Two LLM rating passes (question swapped), normalize 0/2/4 → 0/0.5/1, average. Embedding-free. |

## Phase 2 — Deterministic NLP + string-distance metrics (no provider at all) ✅ DONE

*Zero provider dependency, pure arithmetic, fully offline-testable. Fast to land and verify.*

> ✅ **Shipped 2026-06-05** — `ExactMatchMetric`, `StringPresenceMetric`, `StringSimilarityMetric`,
> `BleuScoreMetric` (real BLEU-4), `ChrfScoreMetric` (real chrF, char n-gram F-β=2), plus
> `NonLlmContextPrecisionMetric` + `NonLlmContextRecallMetric` (enabled by the new
> `SingleTurnSample.reference_contexts` field) — all real `impl Metric`, offline unit-tested.
> **Phase 2 complete** — `IdBasedContextPrecisionMetric` + `IdBasedContextRecallMetric` shipped
> 2026-06-06 (new `SingleTurnSample.retrieved_context_ids` / `reference_context_ids` fields).
> **DistanceMeasure 2026-06-06** — `StringSimilarityMetric` gained a `with_distance_measure`
> selector + `string_distance_similarity_with` free fn: Hamming (rapidfuzz `pad=True`, max-len
> denominator), Jaro, and Jaro-Winkler (p=0.1, prefix cap 4, **boost only when Jaro > 0.7**),
> mirroring Python ragas's `NonLLMStringSimilarity`. Oracle cross-verified two ways (empirical
> rapidfuzz 3.14.5 + independent hand derivation, zero disagreements); an adversarial differential
> test then caught a missing Winkler boost threshold before merge. Case-sensitive; both-empty → 1.0.
> The original Levenshtein-only `string_distance_similarity` is untouched (still backs the NonLLM
> context metrics).

| Metric | Effort | Value | Approach |
|---|---|---|---|
| **StringPresence** | S | medium | `response.contains(reference)` + `Metric` wrapper. |
| **NonLLMContextPrecisionWithReference + NonLLMContextRecall** | S | medium | Per context, max `string_distance_similarity` over the other list → threshold → route through existing `context_precision_from_relevance()` / symmetric recall. Implement as a pair. |
| **NonLLMStringSimilarity (DistanceMeasure enum)** ✅ | M | medium | DONE 2026-06-06 — Levenshtein + Hamming + Jaro + Jaro-Winkler (rapidfuzz-verified, boost-threshold-correct). Jaro-Winkler reused by the Phase-6 OverlapScoreBuilder. |
| **BleuScore (BLEU-4)** | M | medium | Generalize `bleu_unigram` to n=1..4 modified precision + geometric mean + brevity penalty + smoothing. Functional, not sacrebleu byte-parity. |
| **ChrfScore (real chrF/chrF++)** | M | medium | Current fn is char-**unigram** only; generalize to char n=1..N (default 6) F-beta, add word n-grams for chrF++. |
| **IDBasedContextRecall** | S | low | `BTreeSet` intersection mirroring `id_based_context_precision`. Low value (datasets rarely carry context IDs). |
| **ExactMatch / SemanticSimilarity / QuotedSpansAlignment `Metric` wrappers** | S | medium | Logic exists as free functions; add thin `Metric` impls so they run uniformly (closes the `exists_in_rs=partial` wrappers). |

## Phase 3 — LLM metrics that REPLACE lexical placeholders ✅ DONE

*These Rust functions exist but implement weak lexical proxies, not the Python algorithm. Replacing
them is multi-call and the bookkeeping must match Python. Keep the lexical versions as fallbacks.*

> ✅ **Shipped 2026-06-05** (real `impl Metric` in `src/metric.rs`, offline + live-verified vs
> DeepSeek): `FactualCorrectnessMetric`, `ContextEntityRecallMetric`, `ContextRelevanceMetric`
> (nv dual-judge), `ResponseGroundednessMetric` (nv dual-judge), `RubricsScoreMetric`,
> `SummarizationScoreMetric`, and `NoiseSensitivityMetric` (faithful claim×context attribution
> matrix with relevant/irrelevant modes; hand-computed offline tests + live gate). **All 7 done.**

| Metric | Effort | Value | Approach |
|---|---|---|---|
| **ContextEntityRecall (LLM)** | M | medium | Replace the uppercase-token heuristic with an LLM "extract entities → JSON" prompt over reference and over joined contexts; lowercase set-intersect. Keep deterministic fallback. |
| **FactualCorrectness (bidirectional NLI)** | L | high | Two decompositions + two NLI passes (response→reference precision, reference→response recall) → `FactualCorrectnessCounts` → existing F1; surface precision/recall too. |
| **ContextRelevance (Nvidia)** | M | medium | Dual-LLM 0/1/2 rating averaged, replacing the lexical token-overlap fn. |
| **ResponseGroundedness (Nvidia)** | M | medium | Dual-LLM grounding rating averaged; reuse the AnswerAccuracy dual-judge skeleton. Keep lexical fallback. |
| **NoiseSensitivity (real algorithm)** | L | medium | *Corrected up from M.* Claim decomposition + per-context relevance + per-claim→context attribution **matrix** + relevant/irrelevant modes. The most intricate LLM pipeline in the catalog. |
| **SummarizationScore** | L | medium | 3-call chain: keyphrase extraction → QA-pair generation → answer-from-summary-only; coverage = fraction yes, optional conciseness blend. |
| **RubricsScore (DomainSpecificRubrics)** | M | high | Add a `ScoreRubric` type (ordered score→description; current `Rubric` shape is wrong), single LLM call → `{feedback, score 1-5}`, ship the two Python default rubrics. |

## Phase 4 — Agentic + multi-turn metrics ✅ DONE (7/7)

*Counting functions exist; the LLM inference steps are missing and some need `MultiTurnSample` schema
additions (`reference_topics`) + transcript renderers. Share a transcript renderer + infer-outcome prompt.*

> 🔶 **Shipped 2026-06-06** — `DataCompyScoreMetric` (deterministic CSV row precision/recall/F1,
> `DataCompyMode`) on the single-turn `Metric` trait, plus `ToolCallAccuracyMetric` +
> `ToolCallF1Metric` (`src/agentic.rs`, `impl MultiTurnMetric`, deterministic; new
> `MultiTurnSample.reference_tool_calls` field; actual calls extracted from assistant messages,
> matched on name+arguments).

> ✅ **LLM agentic metrics shipped + live-verified 2026-06-06** — the rest of Phase 4, all faithful
> ports of the Python `collections` source (specs cross-checked, then an adversarial Rust-vs-Python
> review found zero real divergences before any live call):
> - `AgentGoalAccuracyWithReferenceMetric` / `AgentGoalAccuracyWithoutReferenceMetric`
>   (`src/agentic.rs`, `impl MultiTurnMetric`) — render the transcript, infer `{user_goal, end_state}`,
>   then a binary compare call. With-reference compares the reference to the end-state; without-reference
>   compares the inferred goal to the end-state. Shared `render_transcript` (Human:/AI:/Tools:/ToolOutput:).
> - `TopicAdherenceMetric` + `TopicAdherenceMode {Precision, Recall, F1}` (`src/agentic.rs`) — 3-stage
>   pipeline (extract topics → per-topic refusal detection → in-scope classification vs the new
>   `MultiTurnSample.reference_topics` field) reduced to a TP/FP/FN confusion matrix with ragas' `eps`
>   guard. Named distinctly from the pre-existing deterministic `TopicAdherence` helper.
> - `InstanceSpecificRubricsMetric` (`src/metric.rs`, `impl Metric`) — like `RubricsScoreMetric` but the
>   rubric is carried **per-sample** via the new `SingleTurnSample.rubrics: Vec<(i64, String)>` field;
>   raw score passthrough, feedback surfaced as the reason.
>
> Live gates `live_agent_goal_accuracy_discriminates_achieved_from_failed`,
> `live_topic_adherence_scores_adherent_above_non_adherent`, and
> `live_instance_specific_rubrics_scores_better_answer_higher` all pass vs DeepSeek. **Phase 4 = 7/7.**

| Metric | Effort | Value | Approach |
|---|---|---|---|
| **ToolCallAccuracy** | M | high | Extractor that flattens assistant tool_calls in order; per-key arg averaging × name_match ÷ reference length. No LLM for the default. |
| **ToolCallF1** | S | high | Set-based P/R/F1 already correct; needs the shared extractor + `(name, frozenset(args))` equality + registry wiring. |
| **AgentGoalAccuracyWithReference** | M | high | 2-call: render transcript → infer outcome → compare to reference → binary verdict. |
| **AgentGoalAccuracyWithoutReference** | M | medium | Adds a goal-inference call in place of the reference; shares the renderer/outcome prompt. |
| **InstanceRubrics** | M | medium | Reuses the RubricsScore engine but adds an optional per-sample `ScoreRubric` to `SingleTurnSample` (schema change touching dataset/CSV/JSONL/tests). |
| **TopicAdherenceScore** | L | medium | 3-stage pipeline (topic extraction, refusal detection, in-scope classification) + P/R/F1, plus `reference_topics` on `MultiTurnSample`. |
| **DataCompyScore** | M | medium | Parse reference/response as CSV strings, on-index row equality + per-column unequal count, P/R/F1 modes. Needs a `csv` crate (already a dep). |

## Phase 5 — Runtime/backend hardening + prompt-system fidelity ✅ DONE (parity-complete; honest N/A/deferred tail)

*Makes the existing pipeline robust and faithful rather than adding metrics. Several are **CRITICAL
latent bugs** — high value because they affect every metric.*

> 🔶 **Shipped 2026-06-05** — `src/resilience.rs`: `ResilientLlmProvider` /
> `ResilientEmbeddingProvider` (retry w/ exponential backoff + per-operation timeout, built from
> `RetryConfig`/`TimeoutConfig` or a `RunConfig`) and `CachingLlmProvider` /
> `CachingEmbeddingProvider` (in-memory memoization) — composable, opt-in, deterministically
> tested. This makes the previously-dead `RunConfig.retry`/`timeout` take effect and adds the
> caching layer nothing wired before (2 of the 3 critical bugs). **Remaining:** FixOutputFormat
> LLM-repair on parse failure + typed `LLMDidNotFinishException` (a parse-path change touching the
> metrics' `parse_json` calls — deferred for a careful refactor), `evaluate()` options
> (column_map / raise_exceptions / token parser / init), per-provider TokenUsageParser + cost on
> report, tiktoken-rs tokenizer, and the PydanticPrompt renderer + Loss foundation.

> ✅ **Wired 2026-06-06** — added `evaluate_with(dataset, metrics, &RunConfig)` (RunConfig-driven
> carrier; minimal `evaluate()` unchanged), and the **CLI evaluate path now wraps the chat provider
> in `ResilientLlmProvider`** (retry + per-op timeout) before building metrics, so retry/timeout is
> no longer opt-in dead config on the default path. Uses a **conservative** eval config (3 attempts /
> 250ms→2s backoff / 60s per-op timeout, concurrency 1), deliberately not `RunConfig::default`'s
> aggressive 10×/60s/180s. Proven by a flaky-provider test (first call fails → metrics still score,
> 0 errors, exactly one retry). Caching/usage decorators remain opt-in (next Bucket-A slices).

> ✅ **Usage tracking wired 2026-06-06** — new `UsageRecordingLlmProvider` decorator; the CLI
> evaluate report now carries a `usage` summary (`{total, by_provider, by_metric}` with
> prompt/completion/total tokens) from the LLM metrics' real calls (per-metric attribution; offline
> → all-zero). Also fixed a test-hygiene bug: four "offline" CLI unit tests were resolving a live
> provider from `.env` and making real API calls during `cargo test` (~40s, cost tokens, network-
> dependent) — switched them to `run_cli_command_with_provider(None)`; lib suite ~40s → ~0.26s,
> truly offline. **Still opt-in / TODO:** caching decorator into eval, callbacks/progress events,
> `usage_summary` on the `EvaluationReport` struct (currently CLI-output only, since `evaluate_with`
> can't see providers), tiktoken token counts, cost (rates) on the summary.

> ✅ **`evaluate()` options + usage/cost wired 2026-06-06** — `EvaluationConfig` +
> `evaluate_with_config(...) -> Result<EvaluationReport, _>`: `raise_exceptions` (fail-fast on the
> first failing cell in `(sample, metric)` order — incl. panicked tasks — vs the default
> collect-and-continue; `evaluate`/`evaluate_with` unchanged, refactored onto a shared
> `run_scoring`). `EvaluationReport` gained a `usage: UsageSummary` field (`#[serde(default)]`),
> now **populated by the library** from a shared `UsageTracker`; the CLI reads `report.usage`.
> `column_map` dataset loaders (`from_jsonl_str_with_column_map` / `from_csv_str_with_column_map`,
> faithful `{canonical: actual}` rename **at the load boundary** — a typed `EvaluationDataset` is
> already mapped, mirroring Python's pre-validation column rename) + a binary `--column-map` flag.
> `UsageSummary::estimated_cost` (raw per-single-token, matching Python `TokenUsage.cost`). Real
> offline BPE token counting via **tiktoken-rs behind an optional `tokenizer` feature**
> (`num_tokens_from_string` + `tiktoken_encoding_for_model`; off by default to keep the build lean,
> covered by a dedicated CI step). Adversarially reviewed vs the ragas 0.4.3 source (orientation,
> raise, per-token cost confirmed).

> ✅ **`LLMDidNotFinishException` shipped 2026-06-06 (PR #3)** — the truncation-detection half of
> FixOutputFormat. New `RagasError::LlmDidNotFinish { reason }`; `parse_chat_response` flags a
> generation cut off by the model (OpenAI `finish_reason == "length"`/`"content_filter"`/…) as a
> distinct typed error instead of returning truncated content that fails an opaque downstream JSON
> parse. Finished set `{stop, STOP, MAX_TOKENS, eos_token, end_turn}`; a **missing** finish_reason
> is lenient (finished), matching ragas's `all([]) == True`. `ResilientLlmProvider` does **not**
> retry it (non-transient). Verified vs ragas `llms/base.py is_finished`.

> ✅ **Evaluation lifecycle callbacks shipped 2026-06-06 (PR #4)** — the faithful analog of ragas's
> `evaluate(..., callbacks=…)`. `EvaluationConfig` gained a `callbacks: CallbackManager`;
> `evaluate_with_config` emits `EvaluationStarted` → per-cell `MetricStarted` /
> `MetricSucceeded` / `MetricFailed` → `EvaluationFinished` `RuntimeEvent`s (added the
> `metric_failed` / `evaluation_finished` constructors). This wires the previously-orphaned
> `CallbackManager` / `RuntimeEvent` infra into the eval path; an empty manager (the default, incl.
> the CLI) is a no-op, so behavior is unchanged when unused.

> ✅ **FixOutputFormat repair shipped 2026-06-07 (PR #20)** — the second (repair) half of
> FixOutputFormat, completing the item whose truncation-detection half landed in PR #3. New
> `generate_and_parse(llm, request, context)` helper (faithful analog of Python ragas's
> `RagasOutputParser.parse_output_string`): generate → [`parse_json`]; on parse failure, feed the
> malformed output **and** the original prompt back to the model through a `FixOutputFormat`-style
> repair prompt (`fix_output_format`, returning Python's `StringIO {text}` wrapper), then re-parse.
> All **~26 `generate`+`parse_json` call sites** in `src/metric.rs` (21) and `src/agentic.rs` (5)
> now route through it, so every LLM metric self-heals malformed-but-complete output. **Documented
> divergences:** repair is bounded to a single attempt (Python's nested `retries_left` recursion is
> a non-goal, like RNG); and the repair also accepts a model that returns the corrected JSON
> *directly* (no `{text}` wrapper) — a robustness superset of Python. Offline tests (no-repair,
> repair-via-wrapper, direct-json fallback, repair-still-fails → context error) + a live gate
> (**FX**: a real model repairs a malformed `{"value": 42}` output). The testset synthesizers'
> `parse_json_block` (Value) path is a separate, optional follow-up (not in the "~24").
> **Review fix:** the first-parse helper `extract_json_block` was rewritten to mirror Python's
> `prompt/utils.py::extract_json` — ` ```json ` fence preference + first **balanced** `{...}`/`[...]`
> structure via bracket-matching (was first-`{` to last-`}`), so multi-object / array / prose-with-
> braces outputs parse like Python (the testset twin `parse_json_block` keeps the old logic — same
> follow-up). Also added an agentic repair test + `prompts().len()==2` assertions on the four
> unparseable-output gates to prove the repair path actually runs.

> ✅ **Phase 5 closed 2026-06-07 (PR #22).** The remaining `evaluate()`-option items are **N/A by
> Rust design**, not unfinished work — documenting them honestly rather than adding redundant/dead code:
> - **`token_usage_parser` / per-provider `TokenUsageParser`** — Python needs a caller-supplied
>   parser because its LLM abstraction returns a raw `LLMResult` whose usage shape varies. ragas-rs
>   parses usage at the **provider boundary** (`parse_chat_response` → `LlmResponse.usage`), so the
>   OpenAI-compatible shape (every provider this crate targets) is parsed by default and any other
>   provider parses its own usage in its `LlmProvider` impl — there is no raw-output layer for an
>   external parser to sit on. Cost is already on the report (`UsageSummary::estimated_cost`).
> - **`metric.init(run_config)`** — Python injects the run config / LLM into each metric just before
>   scoring; ragas-rs metrics are constructed **ready** with their providers (`FaithfulnessMetric::new(llm)`),
>   so there is no separate init step.
> - **`batch_size`** — Python chunks the dataset to bound concurrency; ragas-rs bounds it directly via
>   `RunConfig` concurrency in the `AsyncExecutor` (functional equivalent).
> - **PydanticPrompt renderer + `Loss`** — explicitly "build just-ahead-of the optimizer to avoid dead
>   code", and the real 4-LLM-stage optimizer is in the **deferred/infeasible** tail (needs a metric
>   prompt-set abstraction with no Rust equivalent). Building Loss now would be dead code → **deferred
>   with the optimizer**.
>
> Everything else in Phase 5 is done: retry/timeout, response caching incl. the persistent
> `DiskCacheBackend`, FixOutputFormat repair + `LLMDidNotFinish`, `column_map`, `raise_exceptions`,
> usage + cost on the `EvaluationReport`, lifecycle callbacks, and tiktoken token counts behind the
> `tokenizer` feature. **Phase 5 ✅.**

| Item | Effort | Value | Approach |
|---|---|---|---|
| **RunConfig retry/backoff + per-op timeout** | M | high | **CRITICAL:** retry & timeout are currently *dead config* (only concurrency is consumed). Add exponential-backoff wrapper + `tokio::time::timeout` per job. |
| **Provider response caching** ✅ | M | medium | ✅ DONE — `CachingLlmProvider`/`CachingEmbeddingProvider` decorators (PR #5) **plus** a pluggable `CacheBackend` trait with `InMemoryCacheBackend` (default) and **`DiskCacheBackend`** (PR #21): JSON-file-per-entry persistence; filename = FNV-1a of the key (a fully specified algorithm → stable across runs/machines/Rust releases, unlike std `DefaultHasher`), full key stored in-file + verified on read → collisions degrade to a miss, never a wrong value; **atomic writes** (temp file + rename) so concurrent readers never see a torn file; an unserializable request bypasses the cache (no empty-key collision); all I/O best-effort + mutex-poison-recovering (cache failure never breaks eval). Re-running an eval over the same inputs serves cached responses instead of re-calling the model — faithful analog of Python's `DiskCacheBackend` minus the `diskcache` dep. Deterministic (cross-instance persistence tests for both LLM and embedding paths + corrupt/key-mismatch/fall-through gates, no live gate). |
| **FixOutputFormat repair + `LLMDidNotFinishException`** ✅ | M | high | ✅ DONE — `LLMDidNotFinishException` (PR #3, truncation in `parse_chat_response`) **and** the second-stage LLM repair (PR #20): `generate_and_parse` + `fix_output_format` route all ~26 metric/agentic `generate`+`parse_json` sites through a `FixOutputFormat`-style repair-on-parse-failure pass. Live-verified (FX). |
| **`evaluate()` options: column_map, raise_exceptions, token_usage_parser, metric.init** | M | high | ✅ `column_map` + `raise_exceptions` shipped (`evaluate_with_config`). **N/A by Rust design:** `token_usage_parser` (usage is parsed at the provider boundary into `LlmResponse.usage`, not via a raw-output layer) and `metric.init` (metrics are constructed ready with their providers — no separate init step). |
| **Per-provider TokenUsageParser + cost on report** | M | medium | ✅ Cost on report (`UsageSummary::estimated_cost`). Per-provider parsing is **N/A by Rust design** — each `LlmProvider` impl parses its own usage into `LlmResponse.usage`; the OpenAI-compatible shape (every targeted provider) is parsed by default in `parse_chat_response`. |
| **`num_tokens_from_string` via tiktoken-rs + real Tokenizer trait** | M | low | `LazyTokenizer` is a whitespace stub; real BPE counts feed cost + testset bins. (HF-vocab path is model-dependent — mark it.) |
| **PydanticPrompt-equivalent renderer + Loss types (foundation)** ⏸️ | L | medium | **Deferred with the optimizer.** Loss is to be built *just-ahead-of* the optimizer to avoid dead code, and the real 4-LLM-stage `GeneticOptimizer` is in the deferred/infeasible tail (needs a metric prompt-set abstraction with no Rust equivalent). Building Loss now = dead code, so it moves with the optimizer. |

## Phase 6 — Testset generation stack (heaviest portable subsystem)

*High value (real synthetic data) but the largest dependency chain. All steps are LLM + RNG + pure
math (no downloaded models), so genuinely feasible — but XL aggregate. Substitute char/word
truncation for tiktoken and deterministic sampling for NumPy RNG.*

| Item | Effort | Value | Approach |
|---|---|---|---|
| **KnowledgeGraph widening + cluster/relationship queries** 🔶 | M | high | 🔶 PARTIAL 2026-06-07 — `find_n_indirect_clusters(graph, n, depth_limit, bidirectional, relationship_filter)` DONE: deterministic seeded-DFS port (path-clusters of 2..depth_limit nodes, per-start collection, round-robin dedup with superset-evicts-subset). RNG `random.shuffle` of start nodes dropped (non-goal) → sorted-id order; `bidirectional` is a caller flag (this crate's `GraphEdge` has no per-edge flag — similarity clusters pass `true`). Errors on depth_limit<2 / n<1 / no-match (Python's 3 ValueErrors). 7 hand-computed offline tests (superset-drops-subset, depth cap, directed-vs-bidirectional, relationship filter, n cap, branching distinct paths, arg validation). Earlier: `find_two_nodes_single_rel` analog inlined as `entity_overlap_clusters` (PR #14). Deliberately **skip** Leiden `find_indirect_clusters`. STILL TODO: wire into MultiHopAbstract. |
| **LLM-based extractors (Summary, Keyphrases, Title, Headlines, NER, Themes, TopicDescription)** ✅ | M | high | DONE 2026-06-06 — `LlmExtractor` + `LlmExtractorKind` (all 7 kinds) drive `LlmProvider::generate`, parse with the shared outermost-`{ .. }` JSON-repair path; single-value kinds use `chunks[0]`, list kinds `extend` per chunk (no post-dedup; `max_num` bounds the prompt only). Char-budget substitutes for the tiktoken `split_text_by_token_limit` (documented non-goal). `extract_bundle` (NER→Themes→Summary) feeds the existing `attach_extractions` substrate. Offline ScriptedLlm tests + live gate vs DeepSeek. |
| **Transforms engine (apply_transforms, Parallel groups, default bins)** ✅ | L | high | ✅ DONE 2026-06-07 — `GraphTransform` enum (Extract/Embed with optional node-type filter, Cosine, Overlap, Filter, Parallel) + `apply_transforms` runner threading the graph through each step; `Parallel` children run sequentially (faithful: Python's `apply_transforms` recurses into a `Parallel` as a sequence — its result is identical for independent transforms). **`default_transforms(graph, llm, embedding)` DONE**: char-budget bin branching (`select_default_transform_branch`: long→split path, medium→no-split, else too-short error — char thresholds substitute for tiktoken bins, documented) → extract summaries/themes/entities, embed summaries (`summary_embedding`), filter weak chunks, build `summary_similarity` (via new configurable `build_cosine_relationships_with`) + `entities_overlap` edges. Per-node token-count `filter_nodes` predicates approximated by node-type filtering (documented). Live-verified **raw-text→testset end-to-end** vs DeepSeek+SiliconFlow (gate DT, ~153s: default_transforms → TestsetGenerator with persona gen + all 3 synthesizers). |
| **Splitters + relationship builders + embedding/regex extractors + node filters** 🔶 | M | high | 🔶 PARTIAL 2026-06-06 — `EmbeddingExtractor` (embed node text → `GraphProperty::Vector`) + `build_cosine_relationships` (faithful `CosineSimilarityBuilder`: pairwise cosine over embedded nodes → `cosine_similarity` edges ≥ threshold; reuses `cosine_similarity`; live-verified vs SiliconFlow) + `build_overlap_relationships` (faithful `OverlapScoreBuilder`, the default-pipeline entity-overlap builder: per-pair fuzzy Jaro-Winkler entity matching with the top-5% "noisy item" exclusion → directed `entities_overlap` edges; reuses the Phase-2 `string_distance_similarity_with`; deterministic, hand-computed tests) + `CustomNodeFilter` (faithful port of the default-pipeline `node_filter`: LLM scores each chunk 1–5 against its parent doc's `summary` + rubric, drops `score <= min_score`; live-verified vs DeepSeek) + `split_by_headlines` (faithful `HeadlineSplitter`: cuts a document's `text` at its `headlines`, merges small / word-splits large pieces via `adjust_headline_chunks` — char-budget substitutes for tiktoken — into `chunk` nodes linked by `child`+`next` edges; deterministic `{doc}::h{i}` ids) + `extract_markdown_headings` (deterministic `markdown_headings_extractor` analog, no regex dep). **Deferred:** the general arbitrary-pattern / links / emails `RegexBasedExtractor` — needs a regex engine the default build deliberately excludes (Cargo.toml) and is unused by the generation pipeline (offer: feature-gate it if wanted). |
| **Persona generation from KG + scenario model** 🔶 | M | high | 🔶 PARTIAL 2026-06-07 — `generate_personas_from_kg` (faithful port of `persona.py`: filter nodes with `summary`+`summary_embedding`, greedy cosine>0.75 clustering, longest-summary representative, LLM-generate `Persona{name, role_description}` for the first `num_personas`; reuses the existing `Persona`/`cosine_similarity`; skips `np.random` padding — documented). Live-verified vs DeepSeek. STILL TODO: the scenario model (`generate_scenarios`/`generate_sample`) — lands with the synthesizers. |
| **Three named synthesizers + TestsetGenerator orchestrator** ✅ | L | high | ✅ DONE 2026-06-07 — **All three synthesizers AND the `TestsetGenerator` orchestrator complete end-to-end.** `TestsetGenerator` (port of `generate.py`): `new(llm, kg)` + `with_personas`/`with_llm_context`/`with_query_distribution` + `generate(testset_size, num_personas)` — derives personas (via `generate_personas_from_kg`) or accepts a list, picks `default_query_distribution` (includes only synthesizers whose structure is present — entity nodes / `entities_overlap` edges / similarity clusters — uniform weights, like Python's `get_node_clusters` pre-filter) or a custom one, splits via `calculate_split_values` (`ceil(n·w)`), runs each synthesizer over its split, and merges into one `EvaluationDataset` tagged with `synthesizer_name`. RNG `random.shuffle` dropped (deterministic); a synthesizer that yields no scenarios is non-fatal (contributes zero); `EmptyDataset` only if nothing is generated. Live-verified end-to-end vs DeepSeek (gate TG, ~72s). **ALL THREE named synthesizers COMPLETE end-to-end (3 of 3).** MultiHopAbstract: `prepare_multi_hop_abstract_scenarios` + `MultiHopAbstractSynthesizer` — clusters via `find_n_indirect_clusters` over similarity edges (matches our `cosine_similarity` edge type OR a `summary_similarity` property; bidirectional, depth 3), cluster nodes expanded via `child` edges (else the node itself; HeadlineSplitter not built so the else-branch runs), `generate_concept_combinations` (`ConceptCombinationPrompt` port — LLM pairs concepts across nodes), flattened→theme/persona match, valid nodes = those whose `themes` carry a combination concept; reuses the shared `multi_hop_samples`/`generate_multi_hop_query_answer`/`make_contexts`. Live-verified vs DeepSeek (gate MHA). MultiHopSpecific: `MultiHopScenario{node_ids, combination, persona, style, length}` + `prepare_multi_hop_specific_scenarios` (entity-overlap clusters via `find_two_nodes_single_rel` analog over `entities_overlap` edges, normalized smaller-id-first + deduped; per cluster, themes split from the edge's `overlapped_items` "x => y" strings, theme→persona match, `combination=[theme]`, valid nodes = cluster nodes whose `entities` carry the theme; deterministic style/length rotation, `ceil(n/clusters)` per-cluster cap) + `MultiHopSpecificSynthesizer::generate` (hop-tagged `<i-hop>` contexts via `make_contexts` analog → multi-hop `QueryAnswerGenerationPrompt` port → `SingleTurnSample` with both contexts). Same response/retrieved_contexts-mirroring divergence as single-hop; RNG Cartesian/shuffle skipped (deterministic). Live-verified vs DeepSeek (gate MHS). **SingleHopSpecific synthesizer COMPLETE end-to-end.** Scenario-prep layer: `QueryStyle`/`QueryLength` enums (with `as_str` prompt value + `name` Python-variant name), `SingleHopScenario`, `match_themes_to_personas` (LLM theme→persona matching) + `prepare_single_hop_scenarios` (majority entity-node selection, per-term first-matching persona, deterministic style/length rotation, `ceil(n/nodes)` per-node cap; skips Python's RNG style×length Cartesian + `random.shuffle` — documented). Generation layer: `SingleHopSpecificSynthesizer::generate(graph, personas, n) -> EvaluationDataset` orchestrates prep → per-scenario `QueryAnswerGenerationPrompt` port (`{query, answer}` grounded only in the node text, persona/term/style/length-conditioned, optional `llm_context`); records `synthesis_type`/`source_node_ids`/`term`/`persona_name`/`query_style`/`query_length` metadata. **Documented divergence:** Python leaves `response`/`retrieved_contexts` for the system-under-test (only sets `reference`/`reference_contexts`), but this crate's `EvaluationDataset` requires them non-empty, so (matching the existing `Synthesizer`) the answer/context are mirrored into them; nodes without text are skipped. Live-verified end-to-end vs DeepSeek. STILL TODO: MultiHopAbstract synthesizer (needs `find_n_indirect_clusters` seeded-DFS + `ConceptCombinationPrompt` + `child` edges + `summary_similarity`/`themes` properties) + `TestsetGenerator` orchestrator. |

---

## Deferred / infeasible (honest tail)

| Item | Status | Reason |
|---|---|---|
| **FaithfulnesswithHHEM** | infeasible | Needs Vectara's downloaded HHEM-2.1 cross-encoder; no ort/candle/tokenizer/weights story. Only honest deliverable is a contract that accepts caller-supplied entailment scores (a passthrough). Plain LLM Faithfulness already covers the need. |
| **MultiModalFaithfulness / MultiModalRelevance / ImageText prompts** | deferred (XL) | Need a vision-provider trait + image I/O (base64/SSRF-safe) parallel to `LlmProvider`. Math is trivial but nothing can run until that plumbing lands. Low value; defer until a vision provider is requested. |
| **DSPy / MIPROv2 optimizer** | infeasible | Hard-depends on the Python-only `dspy` package + MIPROv2 teleprompter. Research-grade to reimplement; low marginal value over a real GA. Keep as explicit not-supported markers. |
| **GeneticOptimizer (real ragas 4-LLM-stage algorithm)** | deferred (XL) | Needs a metric prompt-set abstraction (`get/set_prompts`) + annotated-dataset type that don't exist. Current Rust GA is structurally different. Defer behind those + Loss types. |
| **LangChain / LlamaIndex integrations** | infeasible (N/A) | Subclass Python `Chain`/`RunEvaluator` / drive live query engines. No Rust equivalent. The native `evaluate()` pipeline is the analog. |
| **Generic agent-trace converters (LangGraph/Swarm/Bedrock/AG-UI/R2R)** | deferred | SDK-driving halves are N/A; build **one** generic OpenAI-style-messages-JSON → `MultiTurnSample` adapter for the portable halves. Not on the core eval path. |
| **Observability exporters (Langfuse/Helicone/Opik/MLflow, LangSmith)** | deferred (low) | Vendor-SDK glue. If ever needed, start with one REST exporter (Langfuse via reqwest); Helicone is just an alternate base_url+headers. Not an eval capability. |
| **Google Drive real-API backend** | hard (low) | No first-party Rust SDK; needs raw REST + OAuth/service-account signing. Local CSV/JSONL/in-memory cover the real need. |
| **FewShotPydanticPrompt + nearest-example store** | deferred (low) | Largely redundant with the dict-based dynamic few-shot; only worth it once the embedding example-store + PydanticPrompt renderer exist. |
| **ragas.experimental cloud SDK / _analytics telemetry** | N/A | Out-of-process Python services / no-op in Rust. Listed for catalog completeness. |

## Suggested first slice

**Phase 1** — six items, all S–M effort, high value, each reuses scaffolding that already works and
adds no new subsystem. Best single near-term win is **ContextPrecisionWithoutReference** (reference-free
precision is the common production case). Pair it with the **Phase 5 RunConfig retry/timeout fix**
(a CRITICAL latent bug: retry/timeout config is currently ignored) for a high-value reliability
improvement that benefits every metric.
