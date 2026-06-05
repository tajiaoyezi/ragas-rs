# ragas-rs → Python `ragas` functional-parity roadmap

A prioritized plan to maximize **functional replication** of the Python
[`ragas`](https://github.com/explodinggradients/ragas) library in Rust.

> Generated 2026-06-04 from a research pass (current ragas catalog from docs + GitHub,
> ragas-rs surface read from the actual code) followed by an adversarial completeness/correctness
> review. Baseline numbers were verified directly against the source.

**Status:** Phase 1 ✅; Phase 2 ✅ (deterministic + NonLLM context pair; only the low-value
ID-based variant deferred); Phase 3 ✅ (7/7) — all LLM metrics live-verified vs DeepSeek. Plus
`context_utilization` wired into the `ragas evaluate` CLI default. Wired metric count **5 → 25** of
~39 (2026-06-05); all on `main`.

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
> **Only `IDBasedContextRecall` remains** (low value; needs a separate context-ID schema field).

| Metric | Effort | Value | Approach |
|---|---|---|---|
| **StringPresence** | S | medium | `response.contains(reference)` + `Metric` wrapper. |
| **NonLLMContextPrecisionWithReference + NonLLMContextRecall** | S | medium | Per context, max `string_distance_similarity` over the other list → threshold → route through existing `context_precision_from_relevance()` / symmetric recall. Implement as a pair. |
| **NonLLMStringSimilarity (DistanceMeasure enum)** | M | medium | Levenshtein done; add Hamming/Jaro/Jaro-Winkler (or `strsim`). Jaro-Winkler is reused by the Phase-6 OverlapScoreBuilder. |
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

## Phase 4 — Agentic + multi-turn metrics

*Counting functions exist; the LLM inference steps are missing and some need `MultiTurnSample` schema
additions (`reference_topics`) + transcript renderers. Share a transcript renderer + infer-outcome prompt.*

| Metric | Effort | Value | Approach |
|---|---|---|---|
| **ToolCallAccuracy** | M | high | Extractor that flattens assistant tool_calls in order; per-key arg averaging × name_match ÷ reference length. No LLM for the default. |
| **ToolCallF1** | S | high | Set-based P/R/F1 already correct; needs the shared extractor + `(name, frozenset(args))` equality + registry wiring. |
| **AgentGoalAccuracyWithReference** | M | high | 2-call: render transcript → infer outcome → compare to reference → binary verdict. |
| **AgentGoalAccuracyWithoutReference** | M | medium | Adds a goal-inference call in place of the reference; shares the renderer/outcome prompt. |
| **InstanceRubrics** | M | medium | Reuses the RubricsScore engine but adds an optional per-sample `ScoreRubric` to `SingleTurnSample` (schema change touching dataset/CSV/JSONL/tests). |
| **TopicAdherenceScore** | L | medium | 3-stage pipeline (topic extraction, refusal detection, in-scope classification) + P/R/F1, plus `reference_topics` on `MultiTurnSample`. |
| **DataCompyScore** | M | medium | Parse reference/response as CSV strings, on-index row equality + per-column unequal count, P/R/F1 modes. Needs a `csv` crate (already a dep). |

## Phase 5 — Runtime/backend hardening + prompt-system fidelity

*Makes the existing pipeline robust and faithful rather than adding metrics. Several are **CRITICAL
latent bugs** — high value because they affect every metric.*

| Item | Effort | Value | Approach |
|---|---|---|---|
| **RunConfig retry/backoff + per-op timeout** | M | high | **CRITICAL:** retry & timeout are currently *dead config* (only concurrency is consumed). Add exponential-backoff wrapper + `tokio::time::timeout` per job. |
| **Provider response caching** | M | medium | Cache key + store exist but **nothing wraps** `generate`/`embed`. Add a `Cache` trait + caching provider decorator keyed on `generate_runtime_cache_key`. |
| **FixOutputFormat repair + `LLMDidNotFinishException`** | M | high | JSON extraction exists; the second-stage LLM repair on parse failure does not. Add it + the missing typed "model truncated output" error (distinct from malformed JSON). |
| **`evaluate()` options: column_map, raise_exceptions, token_usage_parser, metric.init** | M | high | Core `evaluate()` lacks column remapping, raise-vs-swallow toggle, per-provider token parsers, and an init lifecycle hook. |
| **Per-provider TokenUsageParser + cost on report** | M | medium | Only OpenAI's response shape is parsed; add a trait + per-provider impls, attach total_tokens/cost to `EvaluationReport`. |
| **`num_tokens_from_string` via tiktoken-rs + real Tokenizer trait** | M | low | `LazyTokenizer` is a whitespace stub; real BPE counts feed cost + testset bins. (HF-vocab path is model-dependent — mark it.) |
| **PydanticPrompt-equivalent renderer + Loss types (foundation)** | L | medium | Deterministic `to_string()` renderer + `Loss` trait (MSE/Binary). Build Loss *just-ahead-of* the optimizer to avoid dead code. |

## Phase 6 — Testset generation stack (heaviest portable subsystem)

*High value (real synthetic data) but the largest dependency chain. All steps are LLM + RNG + pure
math (no downloaded models), so genuinely feasible — but XL aggregate. Substitute char/word
truncation for tiktoken and deterministic sampling for NumPy RNG.*

| Item | Effort | Value | Approach |
|---|---|---|---|
| **KnowledgeGraph widening + cluster/relationship queries** | M | high | Foundation. `find_n_indirect_clusters` = portable seeded-DFS (the actually-used multi-hop path); deliberately **skip** Leiden `find_indirect_clusters`. |
| **LLM-based extractors (Summary, Keyphrases, Title, Headlines, NER, Themes, TopicDescription)** | M | high | Same parse-JSON-write-property pattern as existing live metrics; 7 prompts. |
| **Transforms engine (apply_transforms, Parallel groups, default bins)** | L | high | `Transform` trait + sequential/`tokio::join!` runner + token-length bin branching (bins diverge without tiktoken — document it). |
| **Splitters + relationship builders + embedding/regex extractors + node filters** | M | high | Cosine builders reuse `cosine_similarity`; Jaccard/Overlap use Jaro-Winkler from Phase 2; `CustomNodeFilter` is a 1–5 LLM score. |
| **Persona generation from KG + scenario model** | M | high | KG-derived personas (cosine-grouped summaries + prompt) replacing manual seed strings; two-phase `generate_scenarios`/`generate_sample`. |
| **Three named synthesizers + TestsetGenerator orchestrator** | L | high | SingleHopSpecific / MultiHopAbstract / MultiHopSpecific over transform outputs; Rust-native `generate_with_documents/chunks` entrypoints (no LangChain/LlamaIndex loaders). |

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
