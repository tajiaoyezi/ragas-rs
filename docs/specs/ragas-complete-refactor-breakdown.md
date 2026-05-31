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
| 0 | core-mvp-completed | Existing Rust foundation from the first pass | Done |
| 1 | schema-and-datasets | Full sample/message/tool-call schemas and dataset IO | New |
| 2 | runtime-executor | Run config, executor, callbacks, cost, cache | New |
| 3 | providers-and-adapters | LLM/embedding providers and registry | New |
| 4 | prompts-and-parsers | Prompt templates and output parsing | New |
| 5 | metric-framework-complete | Full metric traits, result schema, validators, registry | New |
| 6 | rag-metrics | RAG/context/answer/factual/noise metrics | New |
| 7 | deterministic-and-similarity-metrics | Lexical and embedding similarity metrics | New |
| 8 | advanced-metrics | Rubrics, agent/tool/sql/multimodal/summarization metrics | New |
| 9 | testset-generation | Graph, transforms, personas, synthesizers | New |
| 10 | backends-integrations-cli | Backends, tracing integrations, CLI | New |
| 11 | optimizers-experiments | Experiments, optimizers, benchmark flows | New |
| 12 | parity-docs-release | Golden parity, examples, docs, release gates | New |

## Task Set

| Task | Name | S2V target file to generate |
|---|---|---|
| 1.1 | schema-core | `docs/specs/tasks/task-1.1-schema-core.md` |
| 1.2 | dataset-io | `docs/specs/tasks/task-1.2-dataset-io.md` |
| 1.3 | validation | `docs/specs/tasks/task-1.3-validation.md` |
| 2.1 | run-config | `docs/specs/tasks/task-2.1-run-config.md` |
| 2.2 | executor | `docs/specs/tasks/task-2.2-executor.md` |
| 2.3 | callbacks-cost-cache | `docs/specs/tasks/task-2.3-callbacks-cost-cache.md` |
| 3.1 | provider-core | `docs/specs/tasks/task-3.1-provider-core.md` |
| 3.2 | llm-adapters | `docs/specs/tasks/task-3.2-llm-adapters.md` |
| 3.3 | embedding-adapters | `docs/specs/tasks/task-3.3-embedding-adapters.md` |
| 4.1 | prompt-core | `docs/specs/tasks/task-4.1-prompt-core.md` |
| 4.2 | output-parser | `docs/specs/tasks/task-4.2-output-parser.md` |
| 4.3 | multimodal-prompt | `docs/specs/tasks/task-4.3-multimodal-prompt.md` |
| 5.1 | metric-base | `docs/specs/tasks/task-5.1-metric-base.md` |
| 5.2 | metric-result | `docs/specs/tasks/task-5.2-metric-result.md` |
| 5.3 | metric-registry | `docs/specs/tasks/task-5.3-metric-registry.md` |
| 6.1 | context-metrics | `docs/specs/tasks/task-6.1-context-metrics.md` |
| 6.2 | faithfulness-family | `docs/specs/tasks/task-6.2-faithfulness-family.md` |
| 6.3 | answer-quality | `docs/specs/tasks/task-6.3-answer-quality.md` |
| 7.1 | lexical | `docs/specs/tasks/task-7.1-lexical.md` |
| 7.2 | semantic | `docs/specs/tasks/task-7.2-semantic.md` |
| 7.3 | quoted-spans | `docs/specs/tasks/task-7.3-quoted-spans.md` |
| 8.1 | rubrics | `docs/specs/tasks/task-8.1-rubrics.md` |
| 8.2 | agents-tools | `docs/specs/tasks/task-8.2-agents-tools.md` |
| 8.3 | sql-multimodal-summary | `docs/specs/tasks/task-8.3-sql-multimodal-summary.md` |
| 9.1 | graph-core | `docs/specs/tasks/task-9.1-graph-core.md` |
| 9.2 | transforms | `docs/specs/tasks/task-9.2-transforms.md` |
| 9.3 | synthesizers | `docs/specs/tasks/task-9.3-synthesizers.md` |
| 10.1 | backends | `docs/specs/tasks/task-10.1-backends.md` |
| 10.2 | integrations | `docs/specs/tasks/task-10.2-integrations.md` |
| 10.3 | cli | `docs/specs/tasks/task-10.3-cli.md` |
| 11.1 | experiments | `docs/specs/tasks/task-11.1-experiments.md` |
| 11.2 | optimizers | `docs/specs/tasks/task-11.2-optimizers.md` |
| 11.3 | benchmarks | `docs/specs/tasks/task-11.3-benchmarks.md` |
| 12.1 | parity-suite | `docs/specs/tasks/task-12.1-parity-suite.md` |
| 12.2 | docs-examples | `docs/specs/tasks/task-12.2-docs-examples.md` |
| 12.3 | release | `docs/specs/tasks/task-12.3-release.md` |

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

