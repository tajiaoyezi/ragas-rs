# Task 24.3 - sdk-empty-module-closure

**Status**: Ready
**Phase**: 24
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

The release blocker ledger still treats `workflow::sdk_facing` as a `KnownGap`. In the current upstream baseline, `src/ragas/sdk.py` exists but is empty, so there is no hosted SDK client behavior to port. Treating this empty module as an unresolved blocker overstates the upstream surface and prevents the final audit from distinguishing real missing functionality from a no-op upstream module.

## 2. Goal

Close the `workflow::sdk_facing` release blocker by adding an explicit Rust SDK module contract that records the empty upstream module and converts the workflow parity claim to fixture-backed `Complete`.

## 3. Scope And Out-of-Scope

**In scope**:
- SDK module contract metadata for current upstream `src/ragas/sdk.py`.
- Workflow descriptor and parity claim update for `workflow::sdk_facing`.
- Fixture evidence that the current upstream SDK module is empty and no remote SDK behavior is required for parity.

**Out of scope**:
- Hosted SDK services, remote dashboards, or API clients not present in upstream `sdk.py`.
- Python API binary compatibility.
- Adding new CLI commands unrelated to the empty SDK module.

## 4. Actors

- Rust adopter checking SDK-facing API parity.
- Release owner inspecting workflow release blockers.
- QA engineer validating upstream baseline evidence.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/tasks/task-21.2-experiment-sdk-cli-contracts.md
- test/features/sdk-empty-module-closure.feature
- Upstream baseline file: `src/ragas/sdk.py`

### 5.2 Imports

Use `src/cli/`, `src/parity/`, `src/release/`, and `tests/parity/fixtures/`.

### 5.3 Function Signatures

RED tests own final signatures.

## 6. Acceptance Criteria

- [x] **AC1**: Rust exposes an SDK module contract that records upstream `src/ragas/sdk.py` as zero-byte and non-release-blocking.
- [x] **AC2**: `WorkflowFamily::SdkFacing` descriptor is `Complete`, SDK-surfaced, and fixture-backed.
- [x] **AC3**: `workflow::sdk_facing` is absent from workflow release blockers; synthetic missing workflow claims still block release.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-24.3.1 | TEST-24.3.1 | Spec Ready |
| AC2 | SCEN-24.3.2 | TEST-24.3.2 | Spec Ready |
| AC3 | SCEN-24.3.3 | TEST-24.3.3 | Spec Ready |

## 8. Risks

- If upstream later adds SDK behavior, this closure becomes stale and must be reopened.
- Treating an empty module as complete must be fixture-backed so release owners can audit the baseline.
- This task does not close CLI experiment command gaps or provider/integration blockers.

## 9. Verification Plan

- Install
- Typecheck
- Unit Test
- Build

## 10. Completion Notes

- **完成日期**：待实施后回填
- **改动文件**：待实施后回填
- **commit 列表**：待实施后回填
- **§9 Verification 结果**：待实施后回填
- **剩余风险 / 未做项**：待实施后回填
- **下游 task 影响**：待实施后回填
