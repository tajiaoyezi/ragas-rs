# Task 29.1 - testset-contract-parity-closure

**Status**: Done
**Phase**: 29
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

After metric closure, the release ledger still reports five testset blockers: graph clusters, graph advanced query, LLM extractor, transform filter, and pre-chunked synthesizer generation. Existing Rust testset code has deterministic graph, transform, and synthesizer scaffolding, but these remaining families are still KnownGap or unfixture-backed.

## 2. Goal

Close the testset release-blocker category with deterministic contracts, fixture-backed complete parity claims, and tests for the remaining graph, transform, and synthesizer features.

## 3. Scope And Out-of-Scope

**In scope**:
- Deterministic graph clustering and advanced query APIs.
- Captured LLM extractor fixture parser and deterministic graph filter API.
- Pre-chunked synthesizer API and fixture-backed descriptor.
- Fixture metadata and JSON parity fixtures for every testset parity claim.
- Release ledger tests proving Testset blockers drop to zero.

**Out of scope**:
- Live LLM extractor calls in default CI.
- External graph database query execution.
- Reproducing stochastic upstream sample generation.

## 4. Actors

- Testset generation maintainer.
- Release owner validating remaining release blockers.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-20.1-graph-persistence-query-parity.md
- docs/specs/tasks/task-20.2-transform-engine-extractor-parity.md
- docs/specs/tasks/task-20.3-synthesizer-prompt-fixture-parity.md
- src/testset/mod.rs
- test/features/testset-contract-parity-closure.feature

### 5.2 Imports

Use `src/testset/`, `src/parity/`, `src/release/`, and `tests/parity/fixtures/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Graph cluster and advanced query contracts are deterministic, fixture-backed, and marked complete.
- **AC2**: LLM extractor fixture parsing and graph filtering are deterministic, fixture-backed, and marked complete.
- **AC3**: Pre-chunked synthesizer generation is deterministic, fixture-backed, and the release ledger contains no `Testset` category.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-29.1.1 | TEST-29.1.1 | Done |
| AC2 | SCEN-29.1.2 | TEST-29.1.2 | Done |
| AC3 | SCEN-29.1.3 | TEST-29.1.3 | Done |

## 8. Risks

- Captured LLM extractor fixtures prove parser and normalization contracts, not live model quality.
- Query/filter semantics are deterministic Rust contracts and can need future extension if upstream adds new query operators.
- Fixture paths must be kept complete, otherwise `Complete` claims could become unverified.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build
- testset-test
- parity-test
- examples-build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：`src/testset/mod.rs`; `src/lib.rs`; `src/metrics/registry.rs`; `src/release/mod.rs`; `tests/parity/fixtures/testset_*.json`; `docs/specs/tasks/task-29.1-testset-contract-parity-closure.md`
- **commit 列表**：
  - `450b480 docs(spec): add task-29.1 testset contract parity closure`
  - `6f3940b docs(spec): task-29.1 进入实施`
  - `ee97daf test(testset): 加 task-29.1 RED 测试`
  - `e66f6f4 feat(testset): 实现 task-29.1 testset contract parity closure`
- **RED 结果**：`cargo test test_29_1` failed as expected with 3 tests discovered, 0 passed, 3 failed. The failures showed graph clusters/advanced query, transform LLM extractor/filter, and pre-chunked synthesizer descriptors were still KnownGap and Testset blockers remained in the release ledger.
- **§9 Verification 结果**：
  - Install: `cargo build` passed.
  - Typecheck: `cargo check` passed.
  - Unit Test: `cargo test` passed with 214 passed, 0 failed.
  - Build: `cargo build` passed.
  - Testset Test: `cargo test testset::` passed with 21 passed, 0 failed.
  - Parity Test: `cargo test parity::` passed with 12 passed, 0 failed.
  - Examples Build: `cargo build --examples` passed.
- **剩余风险 / 未做项**：Testset default CI now proves deterministic graph clustering/query, captured LLM extractor parsing, graph filtering, and pre-chunked synthesis contracts at upstream baseline `298b68274234c060deacab3cf5fb52aa3a20e885`; it still does not claim live LLM generation quality or external graph database execution.
- **下游 task 影响**：Testset release blockers dropped from 5 to 0; consolidated ledger moved from 20 to 15 non-waived blockers and now contains only Optimizer and Quality categories.
