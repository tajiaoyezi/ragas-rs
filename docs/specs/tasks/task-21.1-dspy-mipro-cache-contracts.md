# Task 21.1 - dspy-mipro-cache-contracts

**Status**: Ready
**Phase**: 21
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Release v0.4.3 added DSPy optimizer, MIPROv2, and DSPy caching behavior. Current Rust optimizer support is a deterministic genetic optimizer scaffold and does not claim DSPy parity.

## 2. Goal

Implement optimizer descriptors and cache contracts that distinguish existing Rust optimizer behavior from DSPy/MIPROv2 gaps.

## 3. Scope And Out-of-Scope

**In scope**:
- Optimizer family descriptors.
- MIPROv2/DSPy cache compatibility metadata.
- Release-blocking parity claims for unsupported optimizer behavior.

**Out of scope**:
- Embedding the Python DSPy runtime.

## 4. Actors

- Optimizer maintainer.
- Release owner tracking v0.4.3 parity.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/ragas-latest-gap-analysis.md
- test/features/dspy-mipro-cache-contracts.feature

### 5.2 Imports

Use `src/optimizers/`, `src/runtime.rs`, and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [ ] **AC1**: Optimizer registry lists genetic, DSPy, and MIPROv2 families with implementation status.
- [ ] **AC2**: DSPy cache contracts record deterministic key/value behavior and unsupported Python-runtime behavior.
- [ ] **AC3**: Unsupported DSPy/MIPROv2 parity creates release-blocking claims.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-21.1.1 | TEST-21.1.1 | Not Started |
| AC2 | SCEN-21.1.2 | TEST-21.1.2 | Not Started |
| AC3 | SCEN-21.1.3 | TEST-21.1.3 | Not Started |

## 8. Risks

- Optimizer labels can imply algorithmic parity that has not been proven.
- Cache compatibility can leak prompt or credential fields if it reuses raw payloads.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：<TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
