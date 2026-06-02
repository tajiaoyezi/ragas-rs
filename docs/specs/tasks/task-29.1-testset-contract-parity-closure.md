# Task 29.1 - testset-contract-parity-closure

**Status**: In Progress
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
| AC1 | SCEN-29.1.1 | TEST-29.1.1 | Spec Ready |
| AC2 | SCEN-29.1.2 | TEST-29.1.2 | Spec Ready |
| AC3 | SCEN-29.1.3 | TEST-29.1.3 | Spec Ready |

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

- **完成日期**：待实施后回填
- **改动文件**：待实施后回填
- **commit 列表**：待实施后回填
- **RED 结果**：待实施后回填
- **§9 Verification 结果**：待实施后回填
- **剩余风险 / 未做项**：待实施后回填
- **下游 task 影响**：待实施后回填
