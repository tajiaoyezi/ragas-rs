# Phase 21 - optimizer-experiment-cli-docs-parity

**Status**: Ready
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 20

## 1. Goal

Complete optimizer, experiment, SDK-facing, CLI, quickstart, and documentation parity contracts that remain outside the core evaluation loop.

## 2. Scope

`src/optimizers/`, `src/experiments/`, `src/cli/`, `src/docs_examples/`, examples, and release documentation.

## 3. Dependencies

Phase 20 testset fixtures and Phase 19 metric catalog status.

## 4. Risks

- DSPy/MIPROv2 behavior is complex and can be overclaimed by a simple optimizer scaffold.
- CLI examples can pass locally while failing to match upstream workflow semantics.
- Documentation parity can drift unless tracked with executable examples.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 21.1 | docs/specs/tasks/task-21.1-dspy-mipro-cache-contracts.md | Done |
| 21.2 | docs/specs/tasks/task-21.2-experiment-sdk-cli-contracts.md | Ready |
| 21.3 | docs/specs/tasks/task-21.3-quickstart-docs-parity.md | Ready |

## 6. Phase Acceptance And Smoke

- DSPy/MIPROv2 and optimizer cache gaps are explicitly implemented or release-blocking.
- Experiment, SDK-facing, and CLI workflows have deterministic contract tests.
- Quickstart and docs examples are indexed and runnable.
- `cargo build`, `cargo check`, `cargo test`, and `cargo build --examples` pass from the repository root.
