# Task 30.1 - optimizer-contract-parity-closure

**Status**: Ready
**Phase**: 30
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

After Testset closure, the release ledger still reports two optimizer blockers: `optimizers::dspy` and `optimizers::mipro_v2`. Phase 21 identified them as KnownGap because Rust does not embed the Python DSPy runtime.

## 2. Goal

Close optimizer release blockers by adding deterministic Rust DSPy/MIPROv2 contract planning, fixture-backed parity claims, and tests that preserve the default-CI no-Python-runtime boundary.

## 3. Scope And Out-of-Scope

**In scope**:
- DSPy and MIPROv2 optimizer contract descriptors.
- MIPROv2 deterministic trial schedule from seed and trial count.
- DSPy cache planning with redacted deterministic keys.
- Fixture metadata and JSON parity fixtures for optimizer claims.
- Release ledger tests proving Optimizer blockers drop to zero.

**Out of scope**:
- Embedding or invoking the Python DSPy runtime in default CI.
- Live provider optimization calls.
- Stochastic hyperparameter search beyond deterministic contract planning.

## 4. Actors

- Optimizer maintainer.
- Release owner validating optimizer parity blockers.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-21.1-dspy-mipro-cache-contracts.md
- src/optimizers/mod.rs
- test/features/optimizer-contract-parity-closure.feature

### 5.2 Imports

Use `src/optimizers/`, `src/runtime.rs`, `src/parity/`, and `tests/parity/fixtures/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- **AC1**: DSPy and MIPROv2 descriptors are `Complete`, fixture-backed, and keep Python-runtime limitation metadata explicit.
- **AC2**: DSPy cache planning and MIPROv2 trial scheduling are deterministic and redacted.
- **AC3**: Release blocker ledger contains no `Optimizer` category while preserving remaining Quality blockers.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-30.1.1 | TEST-30.1.1 | Spec Ready |
| AC2 | SCEN-30.1.2 | TEST-30.1.2 | Spec Ready |
| AC3 | SCEN-30.1.3 | TEST-30.1.3 | Spec Ready |

## 8. Risks

- Users may expect Python DSPy execution; docs and contract fields must state that default CI proves deterministic planning only.
- Trial schedule fixtures can drift if seed math changes without migration.
- Cache redaction must be preserved for optimizer payloads with secret fields.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build
- optimizers-test
- parity-test
- examples-build

## 10. Completion Notes

- **完成日期**：待实施后回填
- **改动文件**：待实施后回填
- **commit 列表**：待实施后回填
- **RED 结果**：待实施后回填
- **§9 Verification 结果**：待实施后回填
- **剩余风险 / 未做项**：待实施后回填
- **下游 task 影响**：待实施后回填
