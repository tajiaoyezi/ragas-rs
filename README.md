# ragas-rs

A Rust core for evaluating Retrieval-Augmented Generation (RAG) and LLM applications,
inspired by the Python [`ragas`](https://github.com/explodinggradients/ragas) library.

It implements a focused, **real** subset of ragas — faithful multi-step metric pipelines,
LLM-driven test-set generation, an async evaluation runtime, and a small CLI — designed to
be embedded in Rust services. It is **not** a full port of Python ragas (see
[Scope](#scope) below).

## Status & scope

This is an early (`0.1`) library. What it does, it does for real:

- **Metric pipelines** are genuine multi-step implementations, not single-prompt stubs.
  Faithfulness decomposes the answer into atomic statements and verifies each against the
  context (NLI); AnswerRelevancy generates questions from the answer and compares embeddings;
  etc.
- **Tested** with mock LLMs covering discrimination (good vs. bad), JSON-repair, and
  malformed-output paths, plus deterministic-metric unit tests.
- **Verified live** against an OpenAI-compatible provider: Faithfulness, AnswerRelevancy,
  LLMContextPrecision, LLMContextRecall and LLM test-set generation were each run against a
  real model and asserted to discriminate correctly.

Honest caveats:

- Live verification was done against one provider (DeepSeek chat + SiliconFlow embeddings),
  not exhaustively, and **numeric outputs are not validated to match Python ragas
  bit-for-bit**. Treat scores as this library's own, not as drop-in ragas numbers.
- It covers a handful of metrics, not ragas' full catalog. See the table below.

## Features

- **Metrics (real, verified):** `Faithfulness`, `AnswerRelevancy` (ResponseRelevancy),
  `LLMContextPrecision` (with reference), `LLMContextRecall`, and a deterministic `RougeScore`
  (rouge1/2/L, precision/recall/F). Additional lexical helpers (BLEU-1, CHRF, string distance)
  exist but are simplified.
- **Test-set generation:** an LLM-driven `Synthesizer` (single- and multi-hop) over a
  knowledge graph, with a deterministic fallback.
- **Runtime:** an async `evaluate()` with bounded concurrency and per-sample failure isolation.
- **Providers:** an OpenAI-compatible HTTP client (`generate` / `embed`) with key redaction,
  plus mock providers for tests.
- **Optimizer:** a seeded genetic prompt optimizer.
- **CLI:** a `ragas` binary (config / evaluate / testset / benchmark).
- **Config:** centralized provider configuration from environment variables or a `.env` file.

## Install

```toml
# Cargo.toml
[dependencies]
ragas = { git = "https://github.com/<your-org>/ragas-rs" }
```

Requires a recent stable Rust toolchain (edition 2024).

## Quick start — CLI

```bash
cargo run -- config                                  # show resolved provider config (redacted)
cargo run -- evaluate --dataset data.jsonl --report out.json
cargo run -- testset  --doc doc.txt --source-id d1 --multi-hop --out testset.jsonl
cargo run -- help

# install it as a global `ragas` command:
cargo install --path .
ragas config
```

`evaluate` always runs the offline ROUGE-L metric; when an API key is configured it also runs
the LLM-based metrics. A dataset is JSONL, one sample per line:

```json
{"sample_type":"single_turn","user_input":"What is Ragas?","response":"It evaluates LLM apps.","retrieved_contexts":["Ragas evaluates LLM applications."],"reference":"Ragas evaluates LLM applications.","metadata":{}}
```

## Quick start — library

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

See `examples/` (`cargo run --example quickstart_end_to_end` runs the full stack offline with
mock providers).

## Provider configuration

Configuration is resolved by `ProviderConfig` in this order: **real environment variable >
`.env` file > built-in default**. Copy the template and fill in your keys:

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
| `OPENAI_EMBEDDING_BASE_URL` / `OPENAI_EMBEDDING_API_KEY` / `OPENAI_EMBEDDING_MODEL` | embeddings (only `AnswerRelevancy` needs them; may be a different provider) | falls back to the chat key/base; model `text-embedding-3-small` |

The endpoint must be OpenAI-compatible (`{base}/chat/completions`, `{base}/embeddings`).
`.env` is git-ignored — never commit real keys, and never put them in `.env.example`.

## Scope

This library implements a real subset of ragas. The following are **out of scope / not
implemented**, by design:

- Most of ragas' ~45-metric catalog (only the metrics listed above are full implementations).
- Framework integrations (LangChain, LlamaIndex, etc.) — no Rust equivalent.
- DSPy / MIPROv2 prompt optimizers.
- Multimodal and model-download metrics (HHEM, cross-encoder).
- Full numeric parity with Python ragas, and live cloud backends (e.g. Google Drive).

## Development & testing

```bash
cargo test              # unit + binary tests (offline, deterministic)
cargo clippy
cargo test -- --ignored # live tests — require provider keys (env or .env)
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
