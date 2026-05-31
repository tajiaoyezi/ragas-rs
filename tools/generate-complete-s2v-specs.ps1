$phases = @(
  @{N=5; Name='schema-and-datasets'; Goal='完整样本、消息、tool call、多轮数据集、serde schema 与 validation'; Scope='src/schema/ + src/dataset.rs'; Depends='1,4'},
  @{N=6; Name='runtime-executor'; Goal='executor、run config、retry、timeout、cancellation、callbacks、cost、cache'; Scope='src/runtime/ + src/eval.rs'; Depends='5'},
  @{N=7; Name='providers-and-adapters'; Goal='LLM/embedding provider matrix、adapter registry、mock/local/http providers'; Scope='src/providers/ + src/llm.rs'; Depends='5,6'},
  @{N=8; Name='prompts-and-parsers'; Goal='prompt templates、few-shot、typed output parser、judge JSON parser、多模态 prompt scaffold'; Scope='src/prompts/'; Depends='5,6'},
  @{N=9; Name='metric-framework-complete'; Goal='metric base、validators、result schema、metric collection registry、parity labels'; Scope='src/metrics/base.rs + src/metrics/result.rs + src/metrics/validators.rs'; Depends='5,6,8'},
  @{N=10; Name='rag-metrics'; Goal='faithfulness/context/answer/factual/noise/RAG 指标全批次迁移'; Scope='src/metrics/rag/'; Depends='9,7'},
  @{N=11; Name='deterministic-and-similarity-metrics'; Goal='BLEU/ROUGE/CHRF/string/semantic similarity/classic metrics'; Scope='src/metrics/traditional/'; Depends='9,7'},
  @{N=12; Name='advanced-metrics'; Goal='rubrics、agent、tool call、SQL、多模态、summarization metrics'; Scope='src/metrics/advanced/'; Depends='9,7,8'},
  @{N=13; Name='testset-generation'; Goal='graph、transforms、extractors、splitters、relationship builders、persona、single/multi-hop synthesizers'; Scope='src/testset/'; Depends='5,7,8'},
  @{N=14; Name='backends-integrations-cli'; Goal='JSONL/CSV/in-memory backend、optional integrations、CLI evaluate/testset/benchmark'; Scope='src/backends/ + src/integrations/ + src/cli/'; Depends='6,9,13'},
  @{N=15; Name='optimizers-experiments'; Goal='experiment model、prompt/model optimizer、benchmark llm/embedding flows'; Scope='src/experiments/ + src/optimizers/'; Depends='9,14'},
  @{N=16; Name='parity-docs-release'; Goal='upstream parity fixtures、docs/examples、feature flags、release packaging'; Scope='tests/parity/ + examples/ + docs/'; Depends='10,11,12,13,14'}
)

$tasks = @(
  @{Id='5.1'; Phase=5; Name='schema-core'; Module='schema'; Goal='MultiTurnSample、Message、ToolCall、rubric/reference/metadata schema'; AC=@('Message and ToolCall model supports user/assistant/system/tool roles and tool-call IDs','MultiTurnSample preserves ordered messages, reference, rubrics, and metadata','Schema types serialize and deserialize without losing optional fields')},
  @{Id='5.2'; Phase=5; Name='dataset-io'; Module='dataset'; Goal='JSONL/CSV serde roundtrip、dataset builders、validation diagnostics'; AC=@('Dataset can load and save JSONL for single-turn and multi-turn samples','CSV import maps required columns into SingleTurnSample with clear errors','Dataset builders preserve sample order and metadata')},
  @{Id='5.3'; Phase=5; Name='validation'; Module='validation'; Goal='sample/metric compatibility validator、required column checker'; AC=@('Validator detects missing fields required by a metric','Validator reports sample index and field path for invalid records','Validation can run before evaluate and fail without provider calls')},
  @{Id='6.1'; Phase=6; Name='run-config'; Module='runtime'; Goal='timeout/retry/concurrency/cancellation model'; AC=@('RunConfig stores timeout, retry, concurrency, and cancellation settings','Defaults are conservative and deterministic','Invalid config returns structured errors')},
  @{Id='6.2'; Phase=6; Name='executor'; Module='runtime'; Goal='ordered async executor、partial failure isolation、progress events'; AC=@('Executor preserves output order for concurrent tasks','Executor records partial failures without aborting unrelated work','Progress events are emitted for start, success, and failure')},
  @{Id='6.3'; Phase=6; Name='callbacks-cost-cache'; Module='runtime'; Goal='callbacks、token usage/cost model、cache key/value abstraction'; AC=@('Callback hooks receive evaluation lifecycle events','Token usage aggregates per provider and metric','Cache key derivation is stable and redacts secrets')},
  @{Id='7.1'; Phase=7; Name='provider-core'; Module='providers'; Goal='provider registry、mock providers、usage accounting'; AC=@('Provider registry resolves LLM and embedding providers by name','Mock providers support deterministic unit tests','Provider responses carry usage accounting when available')},
  @{Id='7.2'; Phase=7; Name='llm-adapters'; Module='providers'; Goal='OpenAI-compatible completion polish、Azure/local-compatible config'; AC=@('OpenAI-compatible chat client supports base URL, model, and headers','Azure-compatible config maps deployment name and API version','HTTP errors are sanitized and preserve status/body summary')},
  @{Id='7.3'; Phase=7; Name='embedding-adapters'; Module='providers'; Goal='OpenAI-compatible embeddings、batching、normalization'; AC=@('Embedding provider batches inputs without reordering outputs','Optional vector normalization is deterministic','Embedding errors include request batch position')},
  @{Id='8.1'; Phase=8; Name='prompt-core'; Module='prompts'; Goal='typed prompt template、few-shot examples、language adaptation hooks'; AC=@('Prompt template renders typed variables with missing-variable diagnostics','Few-shot examples can be attached and serialized','Language adaptation hook can rewrite prompt text deterministically')},
  @{Id='8.2'; Phase=8; Name='output-parser'; Module='prompts'; Goal='JSON/schema parser、repair strategy、malformed output diagnostics'; AC=@('Parser extracts typed JSON scores and reasons','Malformed judge output returns parse diagnostics with raw excerpt','Repair strategy is explicit and testable')},
  @{Id='8.3'; Phase=8; Name='multimodal-prompt'; Module='prompts'; Goal='image/text prompt scaffold and typed multimodal message model'; AC=@('Multimodal message supports text and image parts','Prompt rendering preserves part order','Unsupported media returns structured error')},
  @{Id='9.1'; Phase=9; Name='metric-base'; Module='metrics'; Goal='full metric traits: single/multi-turn, LLM/embedding requirements, batch hooks'; AC=@('Metric traits distinguish single-turn, multi-turn, LLM, and embedding requirements','Batch scoring hooks default to per-sample behavior','Metric metadata declares required sample fields')},
  @{Id='9.2'; Phase=9; Name='metric-result'; Module='metrics'; Goal='result schema, score normalization, reason/evidence, error taxonomy'; AC=@('Metric result stores score, value type, reason, evidence, and error','Score normalization clamps or rejects invalid numeric scores by policy','Error taxonomy distinguishes provider, parse, validation, and metric failures')},
  @{Id='9.3'; Phase=9; Name='metric-registry'; Module='metrics'; Goal='metric collection registry, feature flags, parity status labels'; AC=@('Metric registry resolves built-ins by stable names','Feature-gated metrics are hidden unless enabled','Parity status labels are exported for docs and tests')},
  @{Id='10.1'; Phase=10; Name='context-metrics'; Module='metrics-rag'; Goal='context precision/recall/entity recall/relevance variants'; AC=@('Context precision variants match declared formulas','Context recall and entity recall operate on references and contexts','Context relevance returns score with evidence')},
  @{Id='10.2'; Phase=10; Name='faithfulness-family'; Module='metrics-rag'; Goal='faithfulness, response groundedness, factual correctness'; AC=@('Faithfulness uses prompt/parser contract from phase 8','Response groundedness records supporting context evidence','Factual correctness handles TP/FP/FN style output')},
  @{Id='10.3'; Phase=10; Name='answer-quality'; Module='metrics-rag'; Goal='answer relevancy/correctness/similarity/noise sensitivity'; AC=@('Answer relevancy supports embedding and LLM judge paths','Answer correctness combines semantic and factual signals','Noise sensitivity returns interpretable numeric score')},
  @{Id='11.1'; Phase=11; Name='lexical'; Module='metrics-traditional'; Goal='exact match/string distance/BLEU/ROUGE/CHRF'; AC=@('Exact/string metrics are deterministic and provider-free','BLEU/ROUGE/CHRF expose documented tokenizer assumptions','Traditional metrics handle empty strings explicitly')},
  @{Id='11.2'; Phase=11; Name='semantic'; Module='metrics-traditional'; Goal='embedding similarity and thresholded semantic metrics'; AC=@('Semantic similarity uses embedding provider with batching','Threshold policy is configurable','Scores are stable for zero vectors')},
  @{Id='11.3'; Phase=11; Name='quoted-spans'; Module='metrics-traditional'; Goal='quoted spans and citation overlap metrics'; AC=@('Quoted span extraction preserves byte and char ranges','Overlap scoring handles partial matches','Missing citations produce explicit zero-score reason')},
  @{Id='12.1'; Phase=12; Name='rubrics'; Module='metrics-advanced'; Goal='aspect critic, simple criteria, domain/instance rubrics'; AC=@('Rubric metrics accept typed criteria','Aspect critic returns binary or graded result according to config','Domain and instance rubrics serialize for audit')},
  @{Id='12.2'; Phase=12; Name='agents-tools'; Module='metrics-advanced'; Goal='goal accuracy, tool call accuracy, tool call F1, topic adherence'; AC=@('Tool call metrics compare names, args, and order policy','Agent goal accuracy supports multi-turn traces','Topic adherence records per-topic evidence')},
  @{Id='12.3'; Phase=12; Name='sql-multimodal-summary'; Module='metrics-advanced'; Goal='SQL semantic equivalence, multimodal faithfulness/relevance, summarization'; AC=@('SQL semantic equivalence compares normalized SQL or judge output','Multimodal metrics route through multimodal prompt model','Summarization score parses coverage and conciseness signals')},
  @{Id='13.1'; Phase=13; Name='graph-core'; Module='testset'; Goal='knowledge graph node/edge model and graph queries'; AC=@('Graph stores nodes, relationships, and typed properties','Graph queries filter by type and relationship','Graph serialization roundtrips fixtures')},
  @{Id='13.2'; Phase=13; Name='transforms'; Module='testset'; Goal='splitters, extractors, filters, relationship builders'; AC=@('Splitters produce stable chunks with source metadata','Extractors attach entities/themes/summaries','Relationship builders create deterministic edges')},
  @{Id='13.3'; Phase=13; Name='synthesizers'; Module='testset'; Goal='persona, single-hop, multi-hop synthesizers'; AC=@('Persona generator stores name, role, and goals','Single-hop synthesizer creates samples from one chunk','Multi-hop synthesizer combines related graph nodes')},
  @{Id='14.1'; Phase=14; Name='backends'; Module='backends'; Goal='in-memory, JSONL, CSV backend registry'; AC=@('Backend trait supports save, load, list, and delete','In-memory backend is deterministic for tests','JSONL and CSV local backends preserve dataset schema')},
  @{Id='14.2'; Phase=14; Name='integrations'; Module='integrations'; Goal='tracing hooks and optional LangSmith/Langfuse/Opik-style adapters'; AC=@('Tracing integration receives callback events','External integrations are feature-gated','Payload redaction is applied before export')},
  @{Id='14.3'; Phase=14; Name='cli'; Module='cli'; Goal='ragas evaluate, ragas testset, ragas benchmark'; AC=@('CLI evaluate reads dataset and writes report','CLI testset invokes synthesizer flow','CLI benchmark prints machine-readable summary')},
  @{Id='15.1'; Phase=15; Name='experiments'; Module='experiments'; Goal='experiment record model, compare runs, report summaries'; AC=@('Experiment records inputs, metrics, provider config, and outputs','Compare runs computes metric deltas','Report summary serializes to JSON')},
  @{Id='15.2'; Phase=15; Name='optimizers'; Module='optimizers'; Goal='prompt/model optimization abstractions and genetic optimizer scaffold'; AC=@('Optimizer trait accepts objective metric and candidate generator','Genetic optimizer scaffold evolves candidates deterministically with seeded RNG','Optimizer history is inspectable')},
  @{Id='15.3'; Phase=15; Name='benchmarks'; Module='benchmarks'; Goal='LLM/embedding benchmark runner and cost summaries'; AC=@('Benchmark runner executes providers over fixed prompts','Cost summary aggregates usage and configured rates','Benchmark output is stable JSON')},
  @{Id='16.1'; Phase=16; Name='parity-suite'; Module='parity'; Goal='upstream golden fixtures, gap matrix, parity status reports'; AC=@('Parity fixture format stores Python baseline and Rust output','Gap matrix lists Complete, Partial, and Known Gap per feature','Parity tests fail on undeclared semantic drift')},
  @{Id='16.2'; Phase=16; Name='docs-examples'; Module='docs'; Goal='Rust examples mapped to upstream howtos/tutorials'; AC=@('Each public workflow has a runnable Rust example','Examples map to upstream docs section names','Docs state feature flags and known parity gaps')},
  @{Id='16.3'; Phase=16; Name='release'; Module='release'; Goal='feature flags, crate metadata, CI gates, release checklist'; AC=@('Cargo features match optional capability groups','CI runs build, check, test, and parity gates','Release checklist includes versioning and rollback steps')}
)

New-Item -ItemType Directory -Force docs/specs/phases, docs/specs/tasks, test/features | Out-Null

foreach ($phase in $phases) {
  $phaseTasks = $tasks | Where-Object { $_.Phase -eq $phase.N }
  $taskRows = ($phaseTasks | ForEach-Object { "| $($_.Id) | docs/specs/tasks/task-$($_.Id)-$($_.Name).md | Draft |" }) -join "`n"
  $content = @"
# Phase $($phase.N) - $($phase.Name)

**Status**: Draft
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md
**Depends On**: $($phase.Depends)

## 1. Goal

$($phase.Goal)

## 2. Scope

$($phase.Scope)

## 3. Dependencies

$($phase.Depends)

## 4. Risks

- Scope is derived from upstream ragas commit 298b682 and may need explicit parity gap registration.
- Optional dependencies must stay feature-gated so the default crate remains embeddable.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
$taskRows

## 6. Phase Acceptance And Smoke

- All tasks in this phase are Done.
- `cargo build` passes from repository root.
- `cargo test` passes from repository root.
- Any task that claims Python ragas parity includes a parity fixture or declares Known Gap.
"@
  Set-Content -Path "docs/specs/phases/phase-$($phase.N)-$($phase.Name).md" -Value $content -Encoding utf8
}

foreach ($task in $tasks) {
  $featurePath = "test/features/$($task.Name).feature"
  $acRows = for ($i = 0; $i -lt $task.AC.Count; $i++) {
    $n = $i + 1
    "| AC$n | SCEN-$($task.Id).$n | TEST-$($task.Id).$n | Not Started |"
  }
  $acList = for ($i = 0; $i -lt $task.AC.Count; $i++) {
    $n = $i + 1
    "- **AC$n**: $($task.AC[$i])"
  }
  $content = @"
# Task $($task.Id) - $($task.Name)

**Status**: Draft
**Phase**: $($task.Phase)
**PRD**: docs/prds/ragas-rs-complete-refactor.prd.md

## 1. Background

This task is part of the complete Rust refactor of upstream ragas commit 298b682. It expands the previously completed MVP core toward full project coverage.

## 2. Goal

$($task.Goal)

## 3. Scope And Out-of-Scope

**In scope**:
- Rust module area: `$($task.Module)`.
- Behavior listed in §6 acceptance criteria.
- Unit tests and, where applicable, parity fixtures for upstream ragas semantics.

**Out of scope**:
- Unrelated phases from the complete refactor matrix.
- Hidden Python runtime dependency or pyo3 bridge.
- Marking parity complete without explicit fixture evidence.

## 4. Actors

- Rust caller using ragas-rs.
- Evaluation framework maintainer tracking Python ragas parity.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-complete-refactor.prd.md
- docs/specs/ragas-complete-refactor-breakdown.md
- $featurePath

### 5.2 Imports

Use existing public crate exports unless this task explicitly creates a new module boundary.

### 5.3 Function Signatures

Function signatures are owned by this task's RED tests and must be added before GREEN implementation.

## 6. Acceptance Criteria

$($acList -join "`n")

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
$($acRows -join "`n")

## 8. Risks

- Upstream Python semantics may not map one-to-one to Rust types.
- Optional external integrations must not leak into the default dependency set.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：待实施
- **改动文件**：待实施
- **commit 列表**：待实施
- **§9 Verification 结果**：待实施
- **剩余风险 / 未做项**：待实施
- **下游 task 影响**：待实施
"@
  Set-Content -Path "docs/specs/tasks/task-$($task.Id)-$($task.Name).md" -Value $content -Encoding utf8

  $scenarioLines = for ($i = 0; $i -lt $task.AC.Count; $i++) {
    $n = $i + 1
    @"

  Scenario: SCEN-$($task.Id).$n $($task.AC[$i])
    Given the complete refactor task $($task.Id)
    When TEST-$($task.Id).$n is executed
    Then the behavior matches the task acceptance criterion
"@
  }
  $feature = @"
# language: en
# Maps to:
#   - docs/specs/tasks/task-$($task.Id)-$($task.Name).md

Feature: $($task.Name)
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want $($task.Goal)
$($scenarioLines -join "")
"@
  Set-Content -Path $featurePath -Value $feature -Encoding utf8
}
