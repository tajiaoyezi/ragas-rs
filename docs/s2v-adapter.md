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
- **Master spec**: docs/prds/ragas-rs.prd.md
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

## Task 总索引

| Task | 模块 | Spec 文件 | Status | 依赖 / Phase 内顺序 | Worktree（仅 team）|
|---|---|---|---|---|---|
| 1.1 | dataset | docs/specs/tasks/task-1.1-foundation-dataset.md | Done | phase 1, first | - |
| 2.1 | metric | docs/specs/tasks/task-2.1-metric-abstractions.md | Done | after task 1.1 | - |
| 3.1 | llm | docs/specs/tasks/task-3.1-providers.md | Done | after task 2.1 | - |
| 4.1 | eval | docs/specs/tasks/task-4.1-evaluator-builtins.md | Done | after task 3.1 | - |

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
