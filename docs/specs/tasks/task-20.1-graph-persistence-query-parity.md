# Task 20.1 - graph-persistence-query-parity

**Status**: Done
**Phase**: 20
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Upstream testset generation relies on knowledge graph storage, graph queries, properties, relationships, and persistence behavior. Current Rust graph support is deterministic but not fixture-backed for full upstream graph workflows.

## 2. Goal

Implement graph parity descriptors and fixture-backed save/load/query contracts for testset generation.

## 3. Scope And Out-of-Scope

**In scope**:
- Graph fixture round trips.
- Query descriptors for node type, property, and relationship filters.
- Release blockers for missing graph features such as clusters or advanced query forms.

**Out of scope**:
- External graph database integrations.

## 4. Actors

- Testset maintainer.
- Release owner validating graph parity.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/ragas-latest-gap-analysis.md
- test/features/graph-persistence-query-parity.feature

### 5.2 Imports

Use `src/testset/` and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Graph fixtures round-trip nodes, edges, and typed properties deterministically.
- **AC2**: Query contracts cover node type, property, and relationship filters.
- **AC3**: Missing upstream graph features create release-blocking claims.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-20.1.1 | TEST-20.1.1 | Done |
| AC2 | SCEN-20.1.2 | TEST-20.1.2 | Done |
| AC3 | SCEN-20.1.3 | TEST-20.1.3 | Done |

## 8. Risks

- Graph ordering differences can create noisy fixture drift.
- Missing graph metadata can change synthesized sample semantics.

## 9. Verification Plan

- install
- typecheck
- unit-test
- parity-test
- build

## 10. Completion Notes

- **完成日期**：2026-06-01
- **改动文件**：src/testset/mod.rs; src/lib.rs
- **commit 列表**：
  - e5ba884 docs(spec): task-20.1 进入实施
  - 773572c test(testset): 加 task-20.1 RED 测试
  - 6f4b0fd feat(testset): 实现 task-20.1 graph parity contracts
- **RED 结果**：`cargo test test_20_1` failed as expected with 3 failing 20.1 tests because graph fixture parsing, query descriptors, and graph release blockers were empty.
- **§9 Verification 结果**：
  - install: `cargo build` passed
  - typecheck: `cargo check` passed
  - unit-test: `cargo test` passed, 157 passed / 0 failed
  - parity-test: `cargo test parity::` passed, 12 passed / 0 failed
  - build: `cargo build` passed
- **剩余风险 / 未做项**：Node type, property, relationship, and neighbor traversal contracts are deterministic; upstream graph clusters and advanced query behavior remain KnownGap release blockers.
- **下游 task 影响**：task 20.2 can build transform and extractor parity on top of deterministic graph fixture parsing and `graph_parity_claims()`.
