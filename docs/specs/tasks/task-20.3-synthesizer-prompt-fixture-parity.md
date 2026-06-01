# Task 20.3 - synthesizer-prompt-fixture-parity

**Status**: Done
**Phase**: 20
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Upstream testset synthesizers create single-hop, multi-hop, and pre-chunked samples through prompt-driven flows. Current Rust synthesizers are deterministic scaffolds without full prompt fixture parity.

## 2. Goal

Implement synthesizer descriptor, prompt snapshot, and fixture comparison contracts for deterministic testset generation parity.

## 3. Scope And Out-of-Scope

**In scope**:
- Single-hop, multi-hop, and pre-chunked synthesizer descriptors.
- Prompt snapshot fixtures.
- Deterministic sample output comparison.
- Release blockers for unsupported synthesizer strategies.

**Out of scope**:
- Default CI live LLM sample generation.

## 4. Actors

- Testset generation maintainer.
- Release owner.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-20.2-transform-engine-extractor-parity.md
- test/features/synthesizer-prompt-fixture-parity.feature

### 5.2 Imports

Use `src/testset/`, `src/prompts/`, and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [x] **AC1**: Synthesizer registry lists single-hop, multi-hop, and pre-chunked strategies.
- [x] **AC2**: Prompt snapshot fixtures preserve variables and rendered message order.
- [x] **AC3**: Unsupported or unfixture-backed synthesizer strategies block release.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-20.3.1 | TEST-20.3.1 | Done |
| AC2 | SCEN-20.3.2 | TEST-20.3.2 | Done |
| AC3 | SCEN-20.3.3 | TEST-20.3.3 | Done |

## 8. Risks

- Prompt fixture drift can be hidden if only final samples are compared.
- Multi-hop relationship selection can be nondeterministic without stable ordering.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Parity Test
- Build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：src/testset/mod.rs; src/lib.rs; docs/specs/tasks/task-20.3-synthesizer-prompt-fixture-parity.md
- **commit 列表**：
  - 069a960 docs(spec): task-20.3 Ready gate format
  - 5813cae docs(spec): task-20.3 进入实施
  - 42ade4a test(testset): 加 task-20.3 RED 测试
  - f8c1720 feat(testset): 实现 task-20.3 synthesizer prompt parity
- **RED 结果**：`cargo test test_20_3` failed as expected with 3 failing 20.3 tests because synthesizer descriptors, prompt snapshot rendering, and synthesizer release blockers were empty.
- **§9 Verification 结果**：
  - Install: `cargo build` passed
  - Typecheck: `cargo check` passed
  - Unit Test: `cargo test` passed, 163 passed / 0 failed
  - Parity Test: `cargo test parity::` passed, 12 passed / 0 failed
  - Build: `cargo build` passed
- **剩余风险 / 未做项**：无 ADR 触发；pre-chunked synthesizer remains a KnownGap release blocker until fixture-backed deterministic generation is implemented.
- **下游 task 影响**：Phase 20 can close after phase smoke; task 21.1 can proceed with testset graph, transform, and synthesizer parity descriptors available.
