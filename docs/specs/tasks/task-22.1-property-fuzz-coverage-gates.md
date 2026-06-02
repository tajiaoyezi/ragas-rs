# Task 22.1 - property-fuzz-coverage-gates

**Status**: Done
**Phase**: 22
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The PRD requires extensive tests beyond unit coverage. Current release gates describe quality evidence, but property, fuzz, and coverage gates are not complete executable release inputs.

## 2. Goal

Implement quality gate descriptors and deterministic checks for property, fuzz, and coverage evidence.

## 3. Scope And Out-of-Scope

**In scope**:
- Gate descriptors for property, fuzz, and coverage evidence.
- Required/optional command classification.
- Release blockers for missing required evidence.

**Out of scope**:
- Requiring long-running fuzz campaigns in default CI.

## 4. Actors

- QA engineer.
- Release owner.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-17.3-quality-gates.md
- test/features/property-fuzz-coverage-gates.feature

### 5.2 Imports

Use `src/release/` and deterministic test helpers.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [x] **AC1**: Property, fuzz, and coverage gates declare command, scope, and required/optional mode.
- [x] **AC2**: Missing required quality evidence creates release-blocking findings.
- [x] **AC3**: Optional long-running gates are represented without blocking deterministic default CI.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-22.1.1 | TEST-22.1.1 | Done |
| AC2 | SCEN-22.1.2 | TEST-22.1.2 | Done |
| AC3 | SCEN-22.1.3 | TEST-22.1.3 | Done |

## 8. Risks

- Coverage tooling availability varies by platform.
- Fuzz evidence can be stale if duration and corpus are not tracked.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：2026-06-02
- **改动文件**：
  - `src/release/mod.rs`（新增 property/fuzz/coverage gate descriptor、required/optional mode、quality command evidence 与 blocker 计算；新增 TEST-22.1.1~22.1.3）
  - `src/lib.rs`（RED 阶段导出新增 release quality gate API）
- **commit 列表**：
  - `acee66e` docs(spec): task-22.1 Ready gate format
  - `a6e1934` docs(spec): task-22.1 进入实施
  - `4fb3153` test(release): 加 task-22.1 RED 测试
  - `78bd04c` feat(release): 实现 task-22.1 quality evidence gates
- **§9 Verification 结果**：
  - Install: passed (`cargo build`)
  - Typecheck: passed (`cargo check`)
  - Unit Test: passed, 175 passed / 0 failed (`cargo test`)
  - Build: passed (`cargo build`)
- **剩余风险 / 未做项**：Long-running fuzz campaign remains optional and non-blocking for deterministic default CI; task 22.2/22.3 must add panic, mutation, platform, and E2E evidence gates before the release audit can claim no known unresolved blockers.
- **下游 task 影响**：task 22.2 can reuse `QualityGateDescriptor`, `QualityGateMode`, and `required_quality_evidence_blockers`; task 23.x can aggregate missing required quality evidence into the final release blocker ledger.
