# Task 21.1 - dspy-mipro-cache-contracts

**Status**: Done
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

- [x] **AC1**: Optimizer registry lists genetic, DSPy, and MIPROv2 families with implementation status.
- [x] **AC2**: DSPy cache contracts record deterministic key/value behavior and unsupported Python-runtime behavior.
- [x] **AC3**: Unsupported DSPy/MIPROv2 parity creates release-blocking claims.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-21.1.1 | TEST-21.1.1 | Done |
| AC2 | SCEN-21.1.2 | TEST-21.1.2 | Done |
| AC3 | SCEN-21.1.3 | TEST-21.1.3 | Done |

## 8. Risks

- Optimizer labels can imply algorithmic parity that has not been proven.
- Cache compatibility can leak prompt or credential fields if it reuses raw payloads.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：src/optimizers/mod.rs; src/lib.rs; docs/specs/tasks/task-21.1-dspy-mipro-cache-contracts.md
- **commit 列表**：
  - 3ccc984 docs(spec): task-21.1 Ready gate format
  - 47af7ca docs(spec): task-21.1 进入实施
  - ebce5df test(optimizers): 加 task-21.1 RED 测试
  - a0861a8 feat(optimizers): 实现 task-21.1 DSPy MIPRO cache contracts
- **RED 结果**：`cargo test test_21_1` failed as expected with 3 failing 21.1 tests because optimizer descriptors, DSPy cache contract metadata, and optimizer release blockers were empty or defaulted.
- **§9 Verification 结果**：
  - Install: `cargo build` passed
  - Typecheck: `cargo check` passed
  - Unit Test: `cargo test` passed, 166 passed / 0 failed
  - Build: `cargo build` passed
- **剩余风险 / 未做项**：无 ADR 触发；DSPy and MIPROv2 remain KnownGap release blockers because the Rust crate does not embed the Python DSPy runtime.
- **下游 task 影响**：task 21.2 can rely on explicit optimizer family descriptors and DSPy cache contract metadata when wiring experiment and CLI contracts.
