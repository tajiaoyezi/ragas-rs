# Changelog

All notable changes to `ragas-rs` are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims to follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**Scope note:** `ragas-rs` targets **functional parity** with Python
[`ragas`](https://github.com/explodinggradients/ragas) — each metric / feature *exists
and works* (and passes a live discrimination test against a real provider) — **not**
byte-exact numeric agreement. See [`docs/parity-roadmap.md`](docs/parity-roadmap.md).

## [Unreleased]

First development line toward `0.1.0`, delivered across six functional-parity phases. All
offline tests pass, and the LLM / embedding / testset paths are live-verified against an
OpenAI-compatible provider (DeepSeek chat + SiliconFlow embeddings) — see
[`docs/live-verification/results.md`](docs/live-verification/results.md).

### Added

- **Evaluation core** — async `evaluate()` / `evaluate_with()` /
  `evaluate_with_config()` with bounded concurrency and per-sample failure isolation;
  `EvaluationDataset`, `SingleTurnSample` / `MultiTurnSample`, JSONL / CSV loaders with
  `column_map`, and an `EvaluationReport` carrying a token-usage + cost summary.
- **~35 metrics** on the `Metric` / `MultiTurnMetric` traits:
  - *Deterministic (no provider):* `ExactMatch`, `StringPresence`, `NonLLMStringSimilarity`
    (Levenshtein / Hamming / Jaro / Jaro-Winkler), `BleuScore`, `ChrfScore`, `RougeScore`,
    NonLLM / ID-based context precision & recall, `DataCompyScore`,
    `ToolCallAccuracy` / `ToolCallF1` (Phases 2 & 4).
  - *LLM / embedding (live-verified):* `Faithfulness`, `ResponseRelevancy`,
    `SemanticSimilarity`, Context Precision / Recall / Utilization / Relevance /
    EntityRecall, `FactualCorrectness`, `AnswerCorrectness` / `AnswerAccuracy`,
    `AspectCritic`, `SimpleCriteria`, `SqlSemanticEquivalence`, `ResponseGroundedness`,
    `RubricsScore`, `SummarizationScore`, `NoiseSensitivity`, and the multi-turn
    `AgentGoalAccuracy` (with / without reference), `TopicAdherence`
    (precision / recall / F1), and `InstanceSpecificRubrics` (Phases 1, 3 & 4).
- **Test-set generation stack** (Phase 6) — 7 LLM extractors, a transforms engine
  (`apply_transforms` / `default_transforms`), embedding / cosine / overlap relationship
  builders, `CustomNodeFilter`, headline splitting, KG persona generation, the three
  named synthesizers (`SingleHopSpecific`, `MultiHopSpecific`, `MultiHopAbstract`), and
  the `TestsetGenerator` orchestrator — raw text → tagged test set, live-verified
  end-to-end.
- **Runtime hardening** (Phase 5) — retry / backoff + per-operation timeout
  (`ResilientLlmProvider`), response caching with a pluggable backend
  (`InMemoryCacheBackend`, persistent `DiskCacheBackend`), `FixOutputFormat` parse-repair
  + `LLMDidNotFinish` truncation detection, usage tracking, lifecycle callbacks, and
  optional offline BPE token counting behind the `tokenizer` feature.
- **Providers** — an OpenAI-compatible HTTP client (`generate` / `embed`) with API-key
  redaction, plus mock providers for tests.
- **CLI** — a `ragas` binary (`config` / `evaluate` / `testset` / `benchmark`).

### Notes

- Numeric parity with Python ragas is **not** a goal (NumPy RNG, Python rounding, and
  tiktoken bin boundaries are out of scope). Deliberate divergences are documented in code
  and commit messages.
- Out of scope by design: downloaded-model metrics (HHEM, cross-encoder NLI), multimodal
  metrics, DSPy / MIPROv2, live framework integrations, and cloud backends — see the
  deferred / infeasible tail in [`docs/parity-roadmap.md`](docs/parity-roadmap.md).

[Unreleased]: https://github.com/tajiaoyezi/ragas-rs/commits/main
