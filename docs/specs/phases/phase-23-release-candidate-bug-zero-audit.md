# Phase 23 - release-candidate-bug-zero-audit

**Status**: Ready
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md
**Depends On**: 22

## 1. Goal

Resolve or explicitly block every remaining parity, quality, safety, and documentation gap before a release-candidate claim.

## 2. Scope

Release blocker ledger, waiver policy, bug-zero audit, final verification evidence, and release checklist updates.

## 3. Dependencies

All previous phases and their generated parity/quality claims.

## 4. Risks

- A release can look green if blockers from separate registries are not aggregated.
- Waivers can hide real gaps unless they require scope, owner, expiry, and rollback impact.
- "No known bugs" can be overstated without clear evidence boundaries.

## 5. Phase Tasks

| Task | Spec | Status |
|---|---|---|
| 23.1 | docs/specs/tasks/task-23.1-release-blocker-ledger.md | Done |
| 23.2 | docs/specs/tasks/task-23.2-gap-resolution-and-waiver-policy.md | Ready |
| 23.3 | docs/specs/tasks/task-23.3-final-bug-zero-release-audit.md | Ready |

## 6. Phase Acceptance And Smoke

- All release-blocking claims are aggregated into one ledger.
- Any waiver is explicit, scoped, justified, and release-visible.
- Final audit refuses release when any critical gap remains.
- `cargo build`, `cargo check`, `cargo test`, `cargo test parity::`, and `cargo build --examples` pass from the repository root.
