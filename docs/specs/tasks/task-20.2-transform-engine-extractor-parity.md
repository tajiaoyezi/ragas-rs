# Task 20.2 - transform-engine-extractor-parity

**Status**: Done
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

- [x] **AC1**: Transform registry lists splitters, extractors, and relationship builders with deterministic/live mode.
- [x] **AC2**: Extracted entities, themes, summaries, and relationships normalize into stable graph properties.
- [x] **AC3**: Unsupported upstream transform stages create release-blocking claims.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-20.2.1 | TEST-20.2.1 | Done |
| AC2 | SCEN-20.2.2 | TEST-20.2.2 | Done |
| AC3 | SCEN-20.2.3 | TEST-20.2.3 | Done |

## 8. Risks

- LLM extractor prompts can change upstream semantics without type changes.
- Relationship ordering can affect downstream synthesizer behavior.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：src/testset/mod.rs; src/lib.rs; docs/specs/tasks/task-20.2-transform-engine-extractor-parity.md
- **commit 列表**：
  - 680be04 docs(spec): task-20.2 Ready gate format
  - 1d91993 docs(spec): task-20.2 进入实施
  - 3a34e15 test(testset): 加 task-20.2 RED 测试
  - e3e5027 feat(testset): 实现 task-20.2 transform extractor parity
  - 7e03de7 docs(spec): task-20.2 verification field format
- **RED 结果**：`cargo test test_20_2` failed as expected with 3 failing 20.2 tests because transform descriptors, extraction normalization, and transform release blockers were empty.
- **§9 Verification 结果**：
  - Install: `cargo build` passed
  - Typecheck: `cargo check` passed
  - Unit Test: `cargo test` passed, 160 passed / 0 failed
  - Build: `cargo build` passed
- **剩余风险 / 未做项**：无 ADR 触发；default CI does not execute live LLM extractors, so `testset::transform::llm_extractor` and `testset::transform::filter` remain explicit release blockers.
- **下游 task 影响**：task 20.3 can use deterministic transform descriptors, normalized extraction graph properties, and transform release blockers when building synthesizer prompt fixtures.
