# Task 17.2 - parity-fixture-policy

**Status**: Done
**Phase**: 17
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Current parity evidence is too small: one tracked fixture cannot support a full upstream rewrite claim.

## 2. Goal

Define and implement a fixture policy that prevents `ParityComplete` claims unless the relevant upstream feature has golden Python baseline data and Rust output evidence.

## 3. Scope And Out-of-Scope

**In scope**:
- `src/parity/` fixture policy types.
- Tests for missing fixture evidence, tolerance policy, and release-blocking labels.
- Fixture naming and storage conventions under `tests/parity/fixtures/`.

**Out of scope**:
- Writing every future metric/provider/testset fixture in this task.
- Live API calls in default CI.

## 4. Actors

- Maintainer adding parity fixtures.
- Release owner verifying claims.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/ragas-latest-gap-analysis.md
- test/features/parity-fixture-policy.feature

### 5.2 Imports

Extend the existing `parity` module.

### 5.3 Function Signatures

RED tests own concrete signatures.

## 6. Acceptance Criteria

- **AC1**: `ParityComplete` validation requires at least one fixture reference.
- **AC2**: Fixture metadata records upstream module path, upstream test source when available, deterministic/mock/live mode, and tolerance policy.
- **AC3**: Known gaps and partial items are allowed during development but release-blocking by default.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|---|
| AC1 | SCEN-17.2.1 | TEST-17.2.1 | Done |
| AC2 | SCEN-17.2.2 | TEST-17.2.2 | Done |
| AC3 | SCEN-17.2.3 | TEST-17.2.3 | Done |

## 8. Risks

- Fixture count can grow quickly; structure must remain searchable.
- Live provider behavior must not destabilize deterministic CI.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build
- extra: `cargo test parity::`

## 10. Completion Notes

- **完成日期**：2026-06-01
- **改动文件**：
  - `src/parity/mod.rs`（新增 `ParityFixtureMetadata`、`ParityFixtureMode`、`ParityClaim`、claim validation 和 release blockers）
  - `src/lib.rs`（导出 fixture policy public API）
- **commit 列表**：
  - `079a67a` docs(spec): task-17.2 进入实施
  - `5981f76` test(parity): 加 task-17.2 RED 测试
  - `935cd81` feat(parity): 实现 task-17.2 fixture policy
- **§9 Verification 结果**：
  - install: ✅ `cargo build`
  - typecheck: ✅ `cargo check`
  - unit-test: ✅ `cargo test` (127 passed / 0 failed)
  - build: ✅ `cargo build`
  - extra: ✅ `cargo test parity::` (9 passed / 0 failed)
- **剩余风险 / 未做项**：本 task 建立 fixture evidence 规则，但尚未为所有 upstream metrics/providers/testset features 生成 fixture；这些由后续 phase/task 执行。
- **下游 task 影响**：task 17.3 可把 `release_blocking_claims` 接入 quality gate；phase 19 的 metric parity tasks 必须为每个 `ParityComplete` claim 提供 fixture metadata。
