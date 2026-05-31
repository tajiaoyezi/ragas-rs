# Project Development Adapter

> S2V Development project adapter for ragas-rs. This file declares project paths, verification commands, constraints, and generated indexes.

---

## Project

- **Name**: ragas-rs
- **Type**: Library
- **Primary users / actors**: Rust backend engineers, platform engineers, evaluation engineers
- **Critical workflows**: Build EvaluationDataset; configure OpenAI-compatible provider or mock provider; run asynchronous evaluate over dataset x metrics; inspect structured EvaluationReport

---

## Specification Locations

- **SDD home**: docs/specs/
- **Master spec**: docs/prds/ragas-rs-complete-refactor.prd.md
- **Phase spec pattern**: docs/specs/phases/phase-{N}-{name}.md
- **Task spec pattern**: docs/specs/tasks/task-{phase}.{seq}-{name}.md
- **BDD acceptance home**: test/features/*.feature
- **ADR home**: docs/decisions/adr-{N}-{title}.md

---

## Source And Test Areas

### Source areas

- src/

### Unit test areas

- src/

### Integration test areas

- N/A: v1.0 keeps tests embedded in Rust modules under src/

### E2E test areas

- N/A: v1.0 is a library without a service or UI runtime

### Test File Naming

Rust unit tests live in `#[cfg(test)] mod tests` blocks inside the source module under `src/`. TEST-ID strings appear in each test body or test name so traceability can be grepped.

---

## Commands

- **Install**: cargo build
- **Lint**: N/A: no lint gate configured for v1.0
- **Typecheck**: cargo check
- **Unit Test**: cargo test
- **Integration tests**: N/A: no integration test suite in v1.0
- **E2E tests**: N/A: no e2e target in v1.0
- **Build**: cargo build
- **Coverage**: N/A: no coverage gate configured for v1.0
- **Runtime smoke**: N/A: library crate has no standalone runtime

---

## Constraints

- **Runtime target**: Rust 1.95+ with caller-provided tokio runtime for async execution
- **Supported platforms**: Linux x64, macOS arm64, Windows x64
- **Security requirements**: no API key storage; no sample or provider response persistence; Authorization header must not appear in errors
- **Performance requirements**: async batch evaluation with configurable concurrency; local metric overhead must remain small; benchmark target is greater than Python ragas 5x throughput with mock provider
- **Compatibility requirements**: no Python API compatibility in v1.0; public Rust API follows semver; serde DTOs use forward-compatible optional fields where practical
- **Release constraints**: Cargo crate library; rollback by semver patch/yank or dependency lock rollback

---

## Workflow

- **Collaboration Tier**: solo
  Overrides:
    - gate-mode: autonomous per user goal
    - branch-model: direct master commits
    - task-spec-archive: disabled

---

## Phase 状态索引

| # | Phase | Phase Spec | Status | Tasks | Worktree（仅 team）|
|---|---|---|---|---|---|
| 1 | foundation-dataset | docs/specs/phases/phase-1-foundation-dataset.md | Done | 1 | - |
| 2 | metric-abstractions | docs/specs/phases/phase-2-metric-abstractions.md | Done | 1 | - |
| 3 | providers | docs/specs/phases/phase-3-providers.md | Done | 1 | - |
| 4 | evaluator-builtins | docs/specs/phases/phase-4-evaluator-builtins.md | Done | 1 | - |
| 5 | schema-and-datasets | docs/specs/phases/phase-5-schema-and-datasets.md | Done | 3 | - |
| 6 | runtime-executor | docs/specs/phases/phase-6-runtime-executor.md | Done | 3 | - |
| 7 | providers-and-adapters | docs/specs/phases/phase-7-providers-and-adapters.md | Done | 3 | - |
| 8 | prompts-and-parsers | docs/specs/phases/phase-8-prompts-and-parsers.md | Done | 3 | - |
| 9 | metric-framework-complete | docs/specs/phases/phase-9-metric-framework-complete.md | Done | 3 | - |
| 10 | rag-metrics | docs/specs/phases/phase-10-rag-metrics.md | Done | 3 | - |
| 11 | deterministic-and-similarity-metrics | docs/specs/phases/phase-11-deterministic-and-similarity-metrics.md | Draft | 3 | - |
| 12 | advanced-metrics | docs/specs/phases/phase-12-advanced-metrics.md | Draft | 3 | - |
| 13 | testset-generation | docs/specs/phases/phase-13-testset-generation.md | Draft | 3 | - |
| 14 | backends-integrations-cli | docs/specs/phases/phase-14-backends-integrations-cli.md | Draft | 3 | - |
| 15 | optimizers-experiments | docs/specs/phases/phase-15-optimizers-experiments.md | Draft | 3 | - |
| 16 | parity-docs-release | docs/specs/phases/phase-16-parity-docs-release.md | Draft | 3 | - |

## Task 总索引

| Task | 模块 | Spec 文件 | Status | 依赖 / Phase 内顺序 | Worktree（仅 team）|
|---|---|---|---|---|---|
| 1.1 | dataset | docs/specs/tasks/task-1.1-foundation-dataset.md | Done | phase 1, first | - |
| 2.1 | metric | docs/specs/tasks/task-2.1-metric-abstractions.md | Done | after task 1.1 | - |
| 3.1 | llm | docs/specs/tasks/task-3.1-providers.md | Done | after task 2.1 | - |
| 4.1 | eval | docs/specs/tasks/task-4.1-evaluator-builtins.md | Done | after task 3.1 | - |
| 5.1 | schema | docs/specs/tasks/task-5.1-schema-core.md | Done | after task 4.1 | - |
| 5.2 | dataset | docs/specs/tasks/task-5.2-dataset-io.md | Done | after task 5.1 | - |
| 5.3 | validation | docs/specs/tasks/task-5.3-validation.md | Done | after task 5.2 | - |
| 6.1 | runtime | docs/specs/tasks/task-6.1-run-config.md | Done | after task 5.3 | - |
| 6.2 | runtime | docs/specs/tasks/task-6.2-executor.md | Done | after task 6.1 | - |
| 6.3 | runtime | docs/specs/tasks/task-6.3-callbacks-cost-cache.md | Done | after task 6.2 | - |
| 7.1 | providers | docs/specs/tasks/task-7.1-provider-core.md | Done | after task 6.3 | - |
| 7.2 | providers | docs/specs/tasks/task-7.2-llm-adapters.md | Done | after task 7.1 | - |
| 7.3 | providers | docs/specs/tasks/task-7.3-embedding-adapters.md | Done | after task 7.2 | - |
| 8.1 | prompts | docs/specs/tasks/task-8.1-prompt-core.md | Done | after task 6.3 | - |
| 8.2 | prompts | docs/specs/tasks/task-8.2-output-parser.md | Done | after task 8.1 | - |
| 8.3 | prompts | docs/specs/tasks/task-8.3-multimodal-prompt.md | Done | after task 8.2 | - |
| 9.1 | metrics | docs/specs/tasks/task-9.1-metric-base.md | Done | after task 8.3 | - |
| 9.2 | metrics | docs/specs/tasks/task-9.2-metric-result.md | Done | after task 9.1 | - |
| 9.3 | metrics | docs/specs/tasks/task-9.3-metric-registry.md | Done | after task 9.2 | - |
| 10.1 | metrics-rag | docs/specs/tasks/task-10.1-context-metrics.md | Done | after task 9.3 | - |
| 10.2 | metrics-rag | docs/specs/tasks/task-10.2-faithfulness-family.md | Done | after task 10.1 | - |
| 10.3 | metrics-rag | docs/specs/tasks/task-10.3-answer-quality.md | Done | after task 10.2 | - |
| 11.1 | metrics-traditional | docs/specs/tasks/task-11.1-lexical.md | Draft | after task 9.3 | - |
| 11.2 | metrics-traditional | docs/specs/tasks/task-11.2-semantic.md | Draft | after task 11.1 | - |
| 11.3 | metrics-traditional | docs/specs/tasks/task-11.3-quoted-spans.md | Draft | after task 11.2 | - |
| 12.1 | metrics-advanced | docs/specs/tasks/task-12.1-rubrics.md | Draft | after task 9.3 | - |
| 12.2 | metrics-advanced | docs/specs/tasks/task-12.2-agents-tools.md | Draft | after task 12.1 | - |
| 12.3 | metrics-advanced | docs/specs/tasks/task-12.3-sql-multimodal-summary.md | Draft | after task 12.2 | - |
| 13.1 | testset | docs/specs/tasks/task-13.1-graph-core.md | Draft | after task 8.3 | - |
| 13.2 | testset | docs/specs/tasks/task-13.2-transforms.md | Draft | after task 13.1 | - |
| 13.3 | testset | docs/specs/tasks/task-13.3-synthesizers.md | Draft | after task 13.2 | - |
| 14.1 | backends | docs/specs/tasks/task-14.1-backends.md | Draft | after task 13.3 | - |
| 14.2 | integrations | docs/specs/tasks/task-14.2-integrations.md | Draft | after task 14.1 | - |
| 14.3 | cli | docs/specs/tasks/task-14.3-cli.md | Draft | after task 14.2 | - |
| 15.1 | experiments | docs/specs/tasks/task-15.1-experiments.md | Draft | after task 14.3 | - |
| 15.2 | optimizers | docs/specs/tasks/task-15.2-optimizers.md | Draft | after task 15.1 | - |
| 15.3 | benchmarks | docs/specs/tasks/task-15.3-benchmarks.md | Draft | after task 15.2 | - |
| 16.1 | parity | docs/specs/tasks/task-16.1-parity-suite.md | Draft | after tasks 10-15 | - |
| 16.2 | docs | docs/specs/tasks/task-16.2-docs-examples.md | Draft | after task 16.1 | - |
| 16.3 | release | docs/specs/tasks/task-16.3-release.md | Draft | after task 16.2 | - |

## ADR 索引

| # | Title | Status | File |
|---|---|---|---|
| 001 | trait-layering | Accepted | docs/decisions/adr-001-trait-layering.md |
| 002 | rust-async-http-dependencies | Accepted | docs/decisions/adr-002-rust-async-http-dependencies.md |
| 003 | cargo-native-test-toolchain | Accepted | docs/decisions/adr-003-cargo-native-test-toolchain.md |
| 004 | openai-compatible-provider-protocol | Accepted | docs/decisions/adr-004-openai-compatible-provider-protocol.md |
| 005 | cargo-library-release-model | Accepted | docs/decisions/adr-005-cargo-library-release-model.md |

## BDD Feature 索引

| Task(s) | Feature 文件 |
|---|---|
| 1.1 | test/features/dataset.feature |
| 2.1 | test/features/metric.feature |
| 3.1 | test/features/llm.feature |
| 4.1 | test/features/eval.feature |
| 5.1 | test/features/schema-core.feature |
| 5.2 | test/features/dataset-io.feature |
| 5.3 | test/features/validation.feature |
| 6.1 | test/features/run-config.feature |
| 6.2 | test/features/executor.feature |
| 6.3 | test/features/callbacks-cost-cache.feature |
| 7.1 | test/features/provider-core.feature |
| 7.2 | test/features/llm-adapters.feature |
| 7.3 | test/features/embedding-adapters.feature |
| 8.1 | test/features/prompt-core.feature |
| 8.2 | test/features/output-parser.feature |
| 8.3 | test/features/multimodal-prompt.feature |
| 9.1 | test/features/metric-base.feature |
| 9.2 | test/features/metric-result.feature |
| 9.3 | test/features/metric-registry.feature |
| 10.1 | test/features/context-metrics.feature |
| 10.2 | test/features/faithfulness-family.feature |
| 10.3 | test/features/answer-quality.feature |
| 11.1 | test/features/lexical.feature |
| 11.2 | test/features/semantic.feature |
| 11.3 | test/features/quoted-spans.feature |
| 12.1 | test/features/rubrics.feature |
| 12.2 | test/features/agents-tools.feature |
| 12.3 | test/features/sql-multimodal-summary.feature |
| 13.1 | test/features/graph-core.feature |
| 13.2 | test/features/transforms.feature |
| 13.3 | test/features/synthesizers.feature |
| 14.1 | test/features/backends.feature |
| 14.2 | test/features/integrations.feature |
| 14.3 | test/features/cli.feature |
| 15.1 | test/features/experiments.feature |
| 15.2 | test/features/optimizers.feature |
| 15.3 | test/features/benchmarks.feature |
| 16.1 | test/features/parity-suite.feature |
| 16.2 | test/features/docs-examples.feature |
| 16.3 | test/features/release.feature |
