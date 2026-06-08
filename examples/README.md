# Examples 📚

Runnable examples for **ragas-rs** (crate name `ragas`). Each example is a standalone binary under `examples/` that you launch with:

```bash
cargo run --example <name>
```

## Offline vs. live

Every example in this directory **runs fully offline** — the ones that need an LLM or embedding model use in-process mock/scripted providers, so no network call is made and no API key is required. Real-LLM behaviour is exercised separately through the `#[ignore]` live gates documented in [../docs/live-verification/results.md](../docs/live-verification/results.md), not by these examples.

The one example that *touches* configuration, `show_config`, still makes no network call: it reads your environment (and `.env`) only to **report** whether live keys are present.

## The examples

| Example | What it shows | Offline/Live | Command |
|---|---|---|---|
| `quickstart_end_to_end` | Full stack on a local `ScriptedLlm` mock: a `Synthesizer` turns a document into a dataset (scripted calls), then `evaluate()` runs deterministic ROUGE-L plus the real `FaithfulnessMetric` pipeline and prints per-metric scores and means. The header notes that real-LLM behaviour is unverified here. | 🖥️ Offline (scripted mock) | `cargo run --example quickstart_end_to_end` |
| `evaluate` | Real `ResponseRelevancyMetric` + `LlmContextRecallMetric` + a custom `FnMetric` run side-by-side through `evaluate()` against `MockLlmProvider`/`MockEmbeddingProvider` — the metrics are genuine, only the provider responses are canned. | 🖥️ Offline (mock providers) | `cargo run --example evaluate` |
| `testset` | Deterministic testset path with no provider: chunk text → `KnowledgeGraph` + `build_chunk_relationships` → single-hop sample via `PersonaGenerator` + `synthesize_single_hop_sample`; prints the synthesized `user_input`. | 🖥️ Offline (no provider) | `cargo run --example testset` |
| `benchmark` | `run_provider_benchmark` over one `MockLlmProvider` (with stamped `TokenUsage`) and one prompt using `CostRates`; prints the JSON benchmark report. | 🖥️ Offline (mock provider) | `cargo run --example benchmark` |
| `experiment` | Experiment tracking with hand-built `EvaluationReport`s (no provider): builds baseline + candidate `ExperimentRecord`s, then runs `summarize_experiment` and `compare_runs` and prints the comparison JSON. | 🖥️ Offline (no provider) | `cargo run --example experiment` |
| `show_config` | Calls `ProviderConfig::from_env()` and prints the secret-redacted resolved provider config plus whether a chat API key is set (i.e. whether LLM metrics / testset / CLI would run live). Reads env and `.env`; makes no network call. | 🖥️ Offline to run; reports live readiness | `cargo run --example show_config` |

## Running live examples 🔌

The examples above stay offline by design, but the library itself runs live once provider keys are configured. To wire up a real OpenAI-compatible provider (DeepSeek, SiliconFlow, OpenAI, etc.):

1. Copy the worked example `.env.example` to `.env` and fill in your keys. Resolution order per value is: real process env var → `.env` in the current directory → built-in default. A blank value is treated as unset.
2. Confirm what got resolved (secrets are redacted in the output):

   ```bash
   cargo run --example show_config
   # or, via the CLI binary:
   cargo run -- config
   ```

   A chat key being set is what flips the CLI and the LLM-backed metrics/testset from their offline fallbacks to live calls.

## See also

- [../README.md](../README.md) — project overview and quick start.
- [../docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md) — module map, core traits, and the evaluation data flow.
- [../docs/parity-roadmap.md](../docs/parity-roadmap.md) — what is implemented vs the Python `ragas` catalog.
