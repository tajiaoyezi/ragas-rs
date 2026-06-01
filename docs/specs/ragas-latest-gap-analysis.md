# ragas-rs latest upstream gap analysis

**Date**: 2026-06-01  
**Source**: `git ls-remote https://github.com/vibrantlabsai/ragas.git`  
**Upstream main**: `298b68274234c060deacab3cf5fb52aa3a20e885`  
**Latest release tag**: `v0.4.3` / `4ecab384fda829ca50bec3f07cc49589d756e172`

## Summary

The current Rust repo has broad module scaffolding and 121 passing tests, but it does not yet satisfy full upstream parity. The strongest current evidence is local Rust correctness for implemented behavior, not upstream equivalence.

## Upstream Source Shape

Upstream `src/ragas` includes these major directories:

| Directory | File count |
|---|---:|
| `backends` | 10 |
| `embeddings` | 8 |
| `integrations` | 15 |
| `llms` | 9 |
| `metrics` | 114 |
| `optimizers` | 7 |
| `prompt` | 23 |
| `testset` | 33 |

Upstream top-level files include runtime and public API areas such as `_analytics.py`, `async_utils.py`, `cache.py`, `callbacks.py`, `cli.py`, `config.py`, `cost.py`, `dataset.py`, `dataset_schema.py`, `evaluation.py`, `executor.py`, `experiment.py`, `losses.py`, `messages.py`, `run_config.py`, `sdk.py`, `tokenizers.py`, `utils.py`, and `validation.py`.

## Current Rust Source Shape

The current Rust repo has module directories for `backends`, `benchmarks`, `cli`, `docs_examples`, `experiments`, `integrations`, `metrics`, `optimizers`, `parity`, `prompts`, `release`, and `testset`, plus core files for dataset, eval, llm, metric, providers, runtime, schema, and validation.

This is a good structural base, but several modules are one-file approximations while upstream has many specialized implementations.

## Release v0.4.3 Delta To Track

- DSPy optimizer with MIPROv2 behavior.
- DSPy caching behavior.
- System prompt support for InstructorLLM and LiteLLMStructuredLLM.
- Remaining quickstart templates.
- FactualCorrectness language adaptation.
- DiskCacheBackend pickling compatibility behavior.
- Lazy default tokenizer initialization.

## Release-Blocking Gap Classes

| Class | Current evidence | Required evidence |
|---|---|---|
| Metric parity | One tracked fixture and many unit tests | Fixture-backed parity for every upstream metric collection and legacy metric |
| Provider parity | OpenAI-compatible, Azure config, mock providers | OpenAI, Google, HuggingFace, LiteLLM, Haystack, OCI, Instructor/LiteLLM structured behavior, system prompt behavior |
| Backend parity | In-memory, local JSONL/CSV | Registry, local, gdrive, cache/disk behavior, schema roundtrip fixtures |
| Testset parity | Lightweight graph/transforms/synthesizers | Graph queries/save/clusters, transform engine, extractors, splitters, relationship builders, single/multi-hop LLM prompt behavior, pre-chunked generation |
| Optimizer parity | Genetic optimizer scaffold | DSPy optimizer, MIPROv2, DSPy caching, optimizer config behavior |
| Integration parity | Generic tracing/redaction | LangChain, LangGraph, LangSmith, LlamaIndex, AG-UI, Bedrock, Griptape, Helicone, Opik, R2R, Swarm, tracing integrations |
| Quality gates | `cargo test` green | Unit + integration + parity + property/fuzz + coverage + mutation/defect ledger evidence |

