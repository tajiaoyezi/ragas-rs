<div align="center">

# ragas-rs

**A Rust core for evaluating Retrieval-Augmented Generation (RAG) and LLM applications.**

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![CI](https://github.com/tajiaoyezi/ragas-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/tajiaoyezi/ragas-rs/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-edition%202024%20%7C%20MSRV%201.88-orange.svg)](Cargo.toml)
[![Status](https://img.shields.io/badge/status-0.1%20early-yellow.svg)](#-status)

*A focused, real subset of Python [`ragas`](https://github.com/explodinggradients/ragas) — genuine multi-step metric pipelines, knowledge-graph test-set generation, and an async evaluation runtime, built to embed in Rust services.*

[Docs](docs/README.md) · [Architecture](docs/ARCHITECTURE.md) · [Roadmap](docs/parity-roadmap.md) · [Live verification](docs/live-verification/results.md) · [Examples](examples/README.md)

</div>

---

## 📖 What is ragas-rs

`ragas-rs` (crate name `ragas`) is a Rust implementation of a focused subset of the Python [`ragas`](https://github.com/explodinggradients/ragas) evaluation library. The metrics are genuine multi-step pipelines — Faithfulness decomposes an answer into atomic statements and verifies each against the retrieved context; ResponseRelevancy generates questions from the answer and compares embeddings — not single-prompt stubs. It is designed to be embedded directly in Rust services and is **not** a full port of Python ragas (see [Scope](#-scope)).

## 🚦 Status

This is an early (`0.1`) library. What it does, it does for real — and it is honest about its limits.

**Live verification** (`docs/live-verification/results.md`): the **22 LLM/embedding metrics** plus the full test-set generation stack were each driven by a real OpenAI-compatible model and asserted to discriminate correctly (a good sample scores strictly above an adversarial one). That is **37 gates, 0 failures** — 22 LLM/embedding discrimination gates + 14 end-to-end / testset gates + 1 runtime-hardening gate (parse-repair). Provider: **DeepSeek** (chat) + **SiliconFlow** (embeddings) over OpenAI-compatible endpoints, runs spanning **2026-06-06 through 2026-06-07**.

**Offline test suite:** `cargo test` runs **407 tests, 0 failures** (397 lib + 9 bin + 1 doctest) fully offline; the 37 live gates are `#[ignore]` and need provider keys.

Honest caveats:

- **One provider tested.** Live verification used a single OpenAI-compatible provider (DeepSeek + SiliconFlow), not a cross-provider matrix.
- **Not numeric parity with Python ragas.** NumPy RNG, Python rounding, and tiktoken bin boundaries are explicitly out of scope. Treat scores as this library's own, not as drop-in ragas numbers.
- **A subset, not the full catalog.** It implements ~35 of ragas' ~39 metric classes; see [Scope](#-scope).
- Each live gate proves discrimination on one representative example pair — not correctness across all inputs.

## ✨ Features

### Metrics — ~35 real, split 13 deterministic / 22 LLM-or-embedding

All metrics are real implementations on the `Metric` (single-turn) and `MultiTurnMetric` (multi-turn) traits: 30 single-turn + 5 multi-turn.

- **13 deterministic (no provider, unit-tested offline):** `ExactMatch`, `StringPresence`, `NonLLMStringSimilarity` (Levenshtein / Hamming / Jaro / Jaro-Winkler), `BleuScore`, `ChrfScore`, `RougeScore` (rouge1/2/L × precision/recall/F), `NonLLMContextPrecision`, `NonLLMContextRecall`, `IDBasedContextPrecision`, `IDBasedContextRecall`, `DataCompyScore`, plus the deterministic multi-turn `ToolCallAccuracy` and `ToolCallF1`.
- **22 LLM/embedding (live-verified):** `Faithfulness`, `ResponseRelevancy`, `SemanticSimilarity` (answer_similarity — embedding cosine of response vs reference), `LLMContextPrecision`, `LLMContextRecall`, `ContextUtilization`, `ContextRelevance`, `ContextEntityRecall`, `FactualCorrectness`, `AnswerCorrectness`, `AnswerAccuracy`, `AspectCritic`, `SimpleCriteriaScore`, `SqlSemanticEquivalence`, `ResponseGroundedness`, `RubricsScore`, `InstanceSpecificRubrics`, `SummarizationScore`, `NoiseSensitivity`, and the agentic multi-turn `AgentGoalAccuracyWithReference`, `AgentGoalAccuracyWithoutReference`, and `TopicAdherence` (precision / recall / F1).

Some lexical metrics fall back to a simplified whitespace tokenizer unless the optional `tokenizer` feature (real tiktoken BPE, ~3–5 MB) is enabled — a documented divergence from Python.

### The rest of the stack

- **Test-set generation:** the Phase-6 `TestsetGenerator` stack — an LLM-driven knowledge graph (extractors → transforms → cosine relationship building → personas) feeding three synthesizers (`SingleHopSpecific`, `MultiHopSpecific`, `MultiHopAbstract`), with a deterministic single-hop fallback.
- **Runtime:** an async `evaluate()` with bounded concurrency (semaphore-gated per `(sample, metric)` cell), per-sample failure isolation, callback/runtime events, and token-usage tracking.
- **Providers:** an OpenAI-compatible HTTP client (`generate` / `embed`) with two-layer key redaction, plus mock providers and resilience decorators (retry / timeout / caching / usage-recording).
- **Optimizer:** a seeded genetic prompt optimizer (a generic GA; the ragas 4-LLM-stage optimizer and DSPy / MIPROv2 are out of scope — see [Scope](#-scope)).
- **CLI:** a thin `ragas` binary — `config` / `evaluate` / `testset` / `benchmark`.
- **Config:** centralized provider configuration resolved from environment variables or a `.env` file.

## 📦 Install

Not published to crates.io — consume it as a git dependency:

```toml
# Cargo.toml
[dependencies]
ragas = { git = "https://github.com/tajiaoyezi/ragas-rs" }
```

Requires Rust **edition 2024** with **MSRV 1.88**. The default feature set is `runtime-tokio` (a no-op gate); the optional `tokenizer` feature wires real tiktoken BPE token counting.

## 🚀 Quick start

### CLI

```bash
cargo run -- config                                  # show resolved provider config (redacted)
cargo run -- evaluate --dataset data.jsonl --report out.json
cargo run -- testset  --doc doc.txt --source-id d1 --multi-hop --out testset.jsonl
cargo run -- benchmark --runs 3
cargo run -- help

# install it as a global `ragas` command:
cargo install --path .
ragas config
```

`evaluate` always runs the offline ROUGE-L metric; when a chat API key is configured it additionally runs `faithfulness` + `context_utilization` (and `context_recall` when any sample has a reference). A dataset is JSONL, one sample per line:

```json
{"sample_type":"single_turn","user_input":"What is Ragas?","response":"It evaluates LLM apps.","retrieved_contexts":["Ragas evaluates LLM applications."],"reference":"Ragas evaluates LLM applications.","metadata":{}}
```

### Library

Offline, deterministic — no provider required:

```rust
use std::sync::Arc;
use ragas::{
    evaluate, EvaluationDataset, EvaluationOptions, ExactMatchMetric, Metric, SingleTurnSample,
};

#[tokio::main]
async fn main() {
    let sample = SingleTurnSample::new("q", "paris", vec!["France".into()])
        .with_reference("paris");
    let dataset = EvaluationDataset::new(vec![sample]).unwrap();
    let metrics: Vec<Arc<dyn Metric>> = vec![Arc::new(ExactMatchMetric::new())];
    let report = evaluate(&dataset, &metrics, EvaluationOptions::default()).await;
    println!("{report:#?}");
}
```

Live, with a real LLM metric:

```rust
use std::sync::Arc;
use ragas::{
    evaluate, EvaluationDataset, EvaluationOptions, FaithfulnessMetric, Metric, ProviderConfig,
    SingleTurnSample,
};

#[tokio::main]
async fn main() {
    let provider = ProviderConfig::from_env()
        .chat_provider()
        .expect("set OPENAI_API_KEY (env or .env)");

    let dataset = EvaluationDataset::new(vec![SingleTurnSample::new(
        "What is Ragas?",
        "Ragas evaluates LLM applications.",
        vec!["Ragas is a framework to evaluate LLM applications.".to_string()],
    )])
    .unwrap();

    let metrics: Vec<Arc<dyn Metric>> = vec![Arc::new(FaithfulnessMetric::new(provider))];
    let report = evaluate(&dataset, &metrics, EvaluationOptions::default()).await;
    println!("{report:#?}");
}
```

See the [`examples/`](examples/) directory — `cargo run --example quickstart_end_to_end` runs the full stack offline with mock providers.

## ⚙️ Provider configuration

Each value is resolved in this order: **real environment variable > `.env` file (cwd) > built-in default**. A `.env` value never overrides an already-set real environment variable; blank values are treated as unset. Copy the template and fill in your keys:

```bash
cp .env.example .env       # PowerShell: Copy-Item .env.example .env
# then edit .env, and verify with:
cargo run -- config
```

| Variable | Purpose | Default |
|---|---|---|
| `OPENAI_API_KEY` | chat auth (Bearer) | — (required for LLM features) |
| `OPENAI_BASE_URL` | chat endpoint root | `https://api.openai.com/v1` |
| `OPENAI_MODEL` | chat model | `gpt-4o-mini` |
| `OPENAI_EMBEDDING_API_KEY` | embeddings auth (may differ from chat) | falls back to the chat key |
| `OPENAI_EMBEDDING_BASE_URL` | embeddings endpoint root | falls back to the chat base URL |
| `OPENAI_EMBEDDING_MODEL` | embeddings model | `text-embedding-3-small` |

The endpoint must be OpenAI-compatible (`{base}/chat/completions`, `{base}/embeddings`). `.env` is git-ignored — never commit real keys, and never put them in `.env.example`.

## 🏗️ Architecture

`ragas-rs` is built on trait-layered module boundaries — `dataset`, `metric` / `metrics`, `llm`, `eval`, `runtime`, `testset`, and `providers` / `resilience` — so callers inject custom metrics and mock or real providers without any global registry, and the public traits form the semver surface. The async `evaluate()` runtime fans out one task per `(sample, metric)` cell under a concurrency semaphore with per-cell failure isolation. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full module map, data-flow diagrams, and the test-set pipeline; the accepted design decisions are recorded as five ADRs in [`docs/decisions/`](docs/decisions/).

## 🔭 Scope

The goal is **functional parity** (each metric exists, works, and passes a live discrimination test), explicitly **not numeric parity** with Python ragas. The following are **out of scope / not implemented**, by design:

- The few remaining ragas metric classes beyond the ~35 above (the catalog is ~39 — chiefly the HHEM and multimodal entries below).
- Byte-exact numeric agreement with Python (NumPy RNG, Python rounding, tiktoken bin boundaries).
- `FaithfulnessWithHHEM` and multimodal metrics (Vectara cross-encoder weights / a vision-provider trait).
- DSPy / MIPROv2 and the real 4-LLM-stage `GeneticOptimizer` (Python-only ecosystem / deferred).
- Framework integrations (LangChain, LlamaIndex) and live cloud backends (e.g. the real Google Drive API).

See [docs/parity-roadmap.md](docs/parity-roadmap.md) for the full phase-by-phase status and the deferred tail.

## 🧪 Development & testing

```bash
cargo test              # 407 offline tests (lib + bin + doctest), deterministic
cargo clippy            # lint
cargo test -- --ignored # the 37 live gates — require provider keys (env or .env)
```

## 🤝 Contributing

Contributions are welcome. Please read:

- [CONTRIBUTING.md](CONTRIBUTING.md) — how to build, test, and submit changes.
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) — community expectations.
- [SECURITY.md](SECURITY.md) — how to report vulnerabilities.
- [CHANGELOG.md](CHANGELOG.md) — notable changes per release.

## 📄 License

Licensed under the [Apache License, Version 2.0](LICENSE).
