# ragas-rs Complete Refactor Breakdown

**Source baseline**: `vibrantlabsai/ragas` commit `298b682`  
**Master PRD**: `docs/prds/ragas-rs-complete-refactor.prd.md`  
**Status**: Draft planning artifact for expanding current S2V scope

## Why This Exists

The first S2V pass intentionally produced a Rust core MVP:

- dataset
- metric
- llm/provider
- eval

That is not a complete ragas project refactor. The upstream Python project currently includes top-level runtime modules plus these major directories:

- `backends/`
- `embeddings/`
- `integrations/`
- `llms/`
- `metrics/`
- `optimizers/`
- `prompt/`
- `testset/`

This document is the bridge from the completed MVP to the full rewrite. It is intentionally more detailed than the existing adapter index so the next S2V update can generate phase/task specs without re-shrinking scope.

## Phase Set

| Phase | Name | Primary purpose | Existing status |
|---|---|---|---|
| 1 | foundation-dataset | Existing Rust foundation from the first pass | Done |
| 2 | metric-abstractions | Existing metric abstraction MVP | Done |
| 3 | providers | Existing OpenAI-compatible provider MVP | Done |
| 4 | evaluator-builtins | Existing evaluate + three metric MVP | Done |
| 5 | schema-and-datasets | Full sample/message/tool-call schemas and dataset IO | New |
| 6 | runtime-executor | Run config, executor, callbacks, cost, cache | New |
| 7 | providers-and-adapters | LLM/embedding providers and registry | New |
| 8 | prompts-and-parsers | Prompt templates and output parsing | New |
| 9 | metric-framework-complete | Full metric traits, result schema, validators, registry | New |
| 10 | rag-metrics | RAG/context/answer/factual/noise metrics | New |
| 11 | deterministic-and-similarity-metrics | Lexical and embedding similarity metrics | New |
| 12 | advanced-metrics | Rubrics, agent/tool/sql/multimodal/summarization metrics | New |
| 13 | testset-generation | Graph, transforms, personas, synthesizers | New |
| 14 | backends-integrations-cli | Backends, tracing integrations, CLI | New |
| 15 | optimizers-experiments | Experiments, optimizers, benchmark flows | New |
| 16 | parity-docs-release | Golden parity, examples, docs, release gates | New |

## Task Set

| Task | Name | S2V target file to generate |
|---|---|---|
| 5.1 | schema-core | `docs/specs/tasks/task-5.1-schema-core.md` |
| 5.2 | dataset-io | `docs/specs/tasks/task-5.2-dataset-io.md` |
| 5.3 | validation | `docs/specs/tasks/task-5.3-validation.md` |
| 6.1 | run-config | `docs/specs/tasks/task-6.1-run-config.md` |
| 6.2 | executor | `docs/specs/tasks/task-6.2-executor.md` |
| 6.3 | callbacks-cost-cache | `docs/specs/tasks/task-6.3-callbacks-cost-cache.md` |
| 7.1 | provider-core | `docs/specs/tasks/task-7.1-provider-core.md` |
| 7.2 | llm-adapters | `docs/specs/tasks/task-7.2-llm-adapters.md` |
| 7.3 | embedding-adapters | `docs/specs/tasks/task-7.3-embedding-adapters.md` |
| 8.1 | prompt-core | `docs/specs/tasks/task-8.1-prompt-core.md` |
| 8.2 | output-parser | `docs/specs/tasks/task-8.2-output-parser.md` |
| 8.3 | multimodal-prompt | `docs/specs/tasks/task-8.3-multimodal-prompt.md` |
| 9.1 | metric-base | `docs/specs/tasks/task-9.1-metric-base.md` |
| 9.2 | metric-result | `docs/specs/tasks/task-9.2-metric-result.md` |
| 9.3 | metric-registry | `docs/specs/tasks/task-9.3-metric-registry.md` |
| 10.1 | context-metrics | `docs/specs/tasks/task-10.1-context-metrics.md` |
| 10.2 | faithfulness-family | `docs/specs/tasks/task-10.2-faithfulness-family.md` |
| 10.3 | answer-quality | `docs/specs/tasks/task-10.3-answer-quality.md` |
| 11.1 | lexical | `docs/specs/tasks/task-11.1-lexical.md` |
| 11.2 | semantic | `docs/specs/tasks/task-11.2-semantic.md` |
| 11.3 | quoted-spans | `docs/specs/tasks/task-11.3-quoted-spans.md` |
| 12.1 | rubrics | `docs/specs/tasks/task-12.1-rubrics.md` |
| 12.2 | agents-tools | `docs/specs/tasks/task-12.2-agents-tools.md` |
| 12.3 | sql-multimodal-summary | `docs/specs/tasks/task-12.3-sql-multimodal-summary.md` |
| 13.1 | graph-core | `docs/specs/tasks/task-13.1-graph-core.md` |
| 13.2 | transforms | `docs/specs/tasks/task-13.2-transforms.md` |
| 13.3 | synthesizers | `docs/specs/tasks/task-13.3-synthesizers.md` |
| 14.1 | backends | `docs/specs/tasks/task-14.1-backends.md` |
| 14.2 | integrations | `docs/specs/tasks/task-14.2-integrations.md` |
| 14.3 | cli | `docs/specs/tasks/task-14.3-cli.md` |
| 15.1 | experiments | `docs/specs/tasks/task-15.1-experiments.md` |
| 15.2 | optimizers | `docs/specs/tasks/task-15.2-optimizers.md` |
| 15.3 | benchmarks | `docs/specs/tasks/task-15.3-benchmarks.md` |
| 16.1 | parity-suite | `docs/specs/tasks/task-16.1-parity-suite.md` |
| 16.2 | docs-examples | `docs/specs/tasks/task-16.2-docs-examples.md` |
| 16.3 | release | `docs/specs/tasks/task-16.3-release.md` |

## Implementation Rule

When these tasks are generated, each task must be implemented through the same S2V rhythm used by the existing MVP tasks:

1. Draft -> Ready self-review commit
2. In Progress commit
3. RED test commit
4. GREEN implementation commit
5. Optional refactor commit
6. §9 verification
7. §10 completion notes
8. adapter Task index update
