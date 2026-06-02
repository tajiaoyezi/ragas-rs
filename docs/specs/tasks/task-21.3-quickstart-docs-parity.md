# Task 21.3 - quickstart-docs-parity

**Status**: Done
**Phase**: 21
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The active PRD includes upstream quickstarts, documentation workflows, and examples. Current Rust docs examples are broad but not exhaustively mapped to latest upstream quickstart templates.

## 2. Goal

Implement quickstart and documentation parity descriptors with runnable example coverage and release blockers for missing upstream docs workflows.

## 3. Scope And Out-of-Scope

**In scope**:
- Quickstart descriptor registry.
- Runnable docs example metadata.
- Missing-template release blockers.

**Out of scope**:
- Hosted documentation publishing.

## 4. Actors

- New user following quickstarts.
- Release owner validating docs parity.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-16.2-docs-examples.md
- test/features/quickstart-docs-parity.feature

### 5.2 Imports

Use `src/docs_examples/`, `examples/`, and `src/parity/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [x] **AC1**: Quickstart registry maps upstream quickstart names to Rust examples or known gaps.
- [x] **AC2**: Runnable example metadata includes command, expected output type, and feature flags.
- [x] **AC3**: Missing or non-runnable docs examples create release-blocking claims.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-21.3.1 | TEST-21.3.1 | Done |
| AC2 | SCEN-21.3.2 | TEST-21.3.2 | Done |
| AC3 | SCEN-21.3.3 | TEST-21.3.3 | Done |

## 8. Risks

- Docs parity can drift without executable examples.
- Feature-gated examples can look runnable in docs but fail in default builds.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：
  - `src/docs_examples/mod.rs`（新增 quickstart descriptor registry、runnable example metadata、docs parity release blockers 与 TEST-21.3.1~21.3.3）
  - `src/lib.rs`（导出 docs parity public API）
- **commit 列表**：
  - `44d01fd` docs(spec): task-21.3 Ready gate format
  - `a54e5f5` docs(spec): task-21.3 进入实施
  - `fbd9893` test(docs): 加 task-21.3 RED 测试
  - `050a94e` feat(docs): 实现 task-21.3 quickstart docs parity
- **§9 Verification 结果**：
  - Install: passed (`cargo build`)
  - Typecheck: passed (`cargo check`)
  - Unit Test: passed, 172 passed / 0 failed (`cargo test`)
  - Build: passed (`cargo build`)
- **剩余风险 / 未做项**：Docs parity registry blocks missing upstream quickstarts at release, but hosted documentation publishing remains out of scope.
- **下游 task 影响**：Phase 22 quality tasks can include quickstart example metadata and docs release blockers in coverage, panic-safety, E2E, and release audit evidence.
