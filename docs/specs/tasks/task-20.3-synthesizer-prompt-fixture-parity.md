# Task 20.3 - synthesizer-prompt-fixture-parity

**Status**: In Progress
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

- [ ] **AC1**: Synthesizer registry lists single-hop, multi-hop, and pre-chunked strategies.
- [ ] **AC2**: Prompt snapshot fixtures preserve variables and rendered message order.
- [ ] **AC3**: Unsupported or unfixture-backed synthesizer strategies block release.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-20.3.1 | TEST-20.3.1 | Not Started |
| AC2 | SCEN-20.3.2 | TEST-20.3.2 | Not Started |
| AC3 | SCEN-20.3.3 | TEST-20.3.3 | Not Started |

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

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：<TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
