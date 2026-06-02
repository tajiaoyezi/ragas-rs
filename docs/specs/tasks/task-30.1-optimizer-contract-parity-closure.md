# Task 30.1 - optimizer-contract-parity-closure

**Status**: Done
**Phase**: 30
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

After Testset closure, the release ledger still reports two optimizer blockers: `optimizers::dspy` and `optimizers::mipro_v2`. Phase 21 identified them as KnownGap because Rust does not embed the Python DSPy runtime.

## 2. Goal

Close optimizer release blockers by adding deterministic Rust DSPy/MIPROv2 contract planning, fixture-backed parity claims, and tests that preserve the default-CI no-Python-runtime boundary.

## 3. Scope And Out-of-Scope

**In scope**:
- DSPy and MIPROv2 optimizer contract descriptors.
- MIPROv2 deterministic trial schedule from seed and trial count.
- DSPy cache planning with redacted deterministic keys.
- Fixture metadata and JSON parity fixtures for optimizer claims.
- Release ledger tests proving Optimizer blockers drop to zero.

**Out of scope**:
- Embedding or invoking the Python DSPy runtime in default CI.
- Live provider optimization calls.
- Stochastic hyperparameter search beyond deterministic contract planning.

## 4. Actors

- Optimizer maintainer.
- Release owner validating optimizer parity blockers.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-21.1-dspy-mipro-cache-contracts.md
- src/optimizers/mod.rs
- test/features/optimizer-contract-parity-closure.feature

### 5.2 Imports

Use `src/optimizers/`, `src/runtime.rs`, `src/parity/`, and `tests/parity/fixtures/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: DSPy and MIPROv2 descriptors are `Complete`, fixture-backed, and keep Python-runtime limitation metadata explicit.
- **AC2**: DSPy cache planning and MIPROv2 trial scheduling are deterministic and redacted.
- **AC3**: Release blocker ledger contains no `Optimizer` category while preserving remaining Quality blockers.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-30.1.1 | TEST-30.1.1 | Done |
| AC2 | SCEN-30.1.2 | TEST-30.1.2 | Done |
| AC3 | SCEN-30.1.3 | TEST-30.1.3 | Done |

## 8. Risks

- Users may expect Python DSPy execution; docs and contract fields must state that default CI proves deterministic planning only.
- Trial schedule fixtures can drift if seed math changes without migration.
- Cache redaction must be preserved for optimizer payloads with secret fields.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build
- optimizers-test
- parity-test
- examples-build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：
  - `src/optimizers/mod.rs`（修改：DSPy/MIPROv2 contract descriptors、fixture-backed parity claims、MIPROv2 deterministic trial planner、RED/GREEN tests）
  - `src/lib.rs`（修改：导出 optimizer contract API）
  - `src/release/mod.rs`（修改：release ledger 测试期望 Optimizer 已关闭）
  - `src/metrics/registry.rs`（修改：metric closure ledger 测试期望 Optimizer 已关闭）
  - `tests/parity/fixtures/optimizer_dspy.json`（新增）
  - `tests/parity/fixtures/optimizer_mipro_v2.json`（新增）
  - `docs/specs/tasks/task-30.1-optimizer-contract-parity-closure.md`（本回填）
- **commit 列表**：
  - `82d9ef5` test(optimizers): 加 task-30.1 RED 测试
  - `bd2b3fc` feat(optimizers): 实现 task-30.1 optimizer contract parity closure
- **RED 结果**：`cargo test test_30_1` failed as expected with 3 failing tests because DSPy/MIPROv2 descriptors were still `KnownGap`, `plan_mipro_v2_trials(9, 3)` returned no schedule, and the release ledger still contained `ReleaseBlockerCategory::Optimizer`.
- **§9 Verification 结果**：
  - install: `cargo build` passed
  - typecheck: `cargo check` passed
  - unit-test: `cargo test` passed, 217 passed / 0 failed
  - build: `cargo build` passed
  - optimizers-test: `cargo test optimizers::` passed, 9 passed / 0 failed
  - parity-test: `cargo test parity::` passed, 12 passed / 0 failed
  - examples-build: `cargo build --examples` passed
  - ledger-smoke: `total=13 non_waived=13 release_ready=false`; remaining category is `Quality=13`
- **剩余风险 / 未做项**：Optimizer release blockers are closed; unrelated Quality release evidence blockers remain and keep the overall project below the perfect-refactor release bar.
- **下游 task 影响**：A follow-up Quality closure task must supply property, fuzz, coverage, panic, mutation, platform, and E2E evidence before the final release ledger can be zero.
