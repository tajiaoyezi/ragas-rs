# Task 24.1 - experiment-quickstart-closure

**Status**: Done
**Phase**: 24
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The final audit ledger still refuses release when docs parity claims contain `KnownGap` entries. `docs::quickstart::experiments` is currently a known gap because there is no runnable Rust experiment quickstart example, even though the Rust crate already has experiment recording, summarization, and run comparison APIs.

## 2. Goal

Close the `docs::quickstart::experiments` release blocker by adding a runnable Rust experiment quickstart and changing the docs parity claim to fixture-backed `Complete` evidence.

## 3. Scope And Out-of-Scope

**In scope**:
- A deterministic `examples/experiment.rs` quickstart that builds with `cargo build --examples`.
- Docs example metadata for the experiment quickstart.
- Docs parity claim evidence that no longer emits `docs::quickstart::experiments` as a release blocker.

**Out of scope**:
- Hosted documentation publishing.
- Live provider calls.
- Resolving non-docs release blockers from providers, integrations, metrics, testset, optimizers, or quality gates.

## 4. Actors

- Rust adopter following the upstream experiments quickstart workflow.
- Release owner checking the consolidated release blocker ledger.
- QA engineer validating example parity evidence.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-21.3-quickstart-docs-parity.md
- docs/specs/tasks/task-23.1-release-blocker-ledger.md
- test/features/experiment-quickstart-closure.feature

### 5.2 Imports

Use `src/docs_examples/`, `src/experiments/`, `src/release/`, `examples/`, and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [x] **AC1**: `Run experiments` quickstart maps to a real Rust example path and appears in runnable docs example metadata.
- [x] **AC2**: The experiment example uses deterministic experiment summary and run comparison APIs without live providers.
- [x] **AC3**: `docs_parity_claims()` and the consolidated release blocker ledger no longer report `docs::quickstart::experiments` as a docs release blocker.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-24.1.1 | TEST-24.1.1 | Done |
| AC2 | SCEN-24.1.2 | TEST-24.1.2 | Done |
| AC3 | SCEN-24.1.3 | TEST-24.1.3 | Done |

## 8. Risks

- A metadata-only change would hide a docs blocker without proving that the quickstart compiles.
- The example could depend on nondeterministic provider behavior and make default CI flaky.
- Closing one docs blocker does not imply the whole final audit is release-ready.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Manual: cargo test parity::
- Build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：
  - `src/docs_examples/mod.rs`（新增 `Run experiments` runnable metadata、fixture-backed docs parity claim 与 TEST-24.1.1~24.1.3）
  - `src/release/mod.rs`（修正 ledger 聚合测试，允许已关闭 docs blockers 后 Docs 类别为空）
  - `examples/experiment.rs`（新增确定性 experiment quickstart）
- **commit 列表**：
  - `8f42d4f` docs(spec): task-24.1 进入实施
  - `73d9701` test(docs): 加 task-24.1 RED 测试
  - `45f190d` feat(docs): 实现 task-24.1 experiment quickstart closure
- **§9 Verification 结果**：
  - Install: passed (`cargo build`)
  - Typecheck: passed (`cargo check`)
  - Unit Test: passed, 193 passed / 0 failed (`cargo test`)
  - Manual parity: passed, 12 passed / 0 failed (`cargo test parity::`; helper `manual` step requires `/dev/tty`, so parity was verified by direct command execution)
  - Build: passed (`cargo build`)
  - Additional smoke: passed (`cargo build --examples`)
- **剩余风险 / 未做项**：This closes only the `docs::quickstart::experiments` docs release blocker; provider, backend, integration, metric, testset, optimizer, and quality evidence blockers remain outside this task.
- **下游 task 影响**：Phase 24 can continue with the next release-blocker closure task; final audit must still refuse release until all remaining non-waived blockers and required evidence are resolved.
