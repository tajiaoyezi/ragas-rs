# Task 20.2 - transform-engine-extractor-parity

**Status**: Ready
**Phase**: 20
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Upstream transform stages include splitters, extractors, relationship builders, and graph enrichment. Current Rust transform support is deterministic but lacks a full transform engine contract.

## 2. Goal

Implement transform and extractor registry contracts with deterministic fixture evidence and release blockers for unsupported stages.

## 3. Scope And Out-of-Scope

**In scope**:
- Transform stage descriptors.
- Extractor output normalization.
- Relationship builder fixture coverage.
- Release-blocking claims for unsupported extractors.

**Out of scope**:
- Live LLM extractor calls in default CI.

## 4. Actors

- Testset maintainer.
- CI/release owner.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-20.1-graph-persistence-query-parity.md
- test/features/transform-engine-extractor-parity.feature

### 5.2 Imports

Use `src/testset/`, `src/prompts/`, and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: Transform registry lists splitters, extractors, and relationship builders with deterministic/live mode.
- **AC2**: Extracted entities, themes, summaries, and relationships normalize into stable graph properties.
- **AC3**: Unsupported upstream transform stages create release-blocking claims.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-20.2.1 | TEST-20.2.1 | Not Started |
| AC2 | SCEN-20.2.2 | TEST-20.2.2 | Not Started |
| AC3 | SCEN-20.2.3 | TEST-20.2.3 | Not Started |

## 8. Risks

- LLM extractor prompts can change upstream semantics without type changes.
- Relationship ordering can affect downstream synthesizer behavior.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：<TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
