# Task 20.1 - graph-persistence-query-parity

**Status**: Ready
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
| AC1 | SCEN-20.1.1 | TEST-20.1.1 | Not Started |
| AC2 | SCEN-20.1.2 | TEST-20.1.2 | Not Started |
| AC3 | SCEN-20.1.3 | TEST-20.1.3 | Not Started |

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

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：<TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
