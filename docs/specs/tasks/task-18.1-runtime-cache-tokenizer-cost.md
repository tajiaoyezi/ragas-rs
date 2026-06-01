# Task 18.1 - runtime-cache-tokenizer-cost

**Status**: In Progress
**Phase**: 18
**PRD**: docs/prds/ragas-rs-perfect-refactor.prd.md

## 1. Background

Upstream `cache.py`, `tokenizers.py`, `cost.py`, and `run_config.py` define cache-key exclusion behavior, disk/cache compatibility, lazy default tokenizer initialization, token usage addition, and per-model cost accounting. Current Rust code has run config, usage tracking, and cache-key fragments but does not yet expose an explicit upstream parity contract for these behaviors.

## 2. Goal

Implement deterministic Rust contracts for upstream-compatible cache key generation, lazy tokenizer metadata, and token usage/cost aggregation.

## 3. Scope And Out-of-Scope

**In scope**:
- Runtime cache key inputs that exclude callback-like fields and sort structured data deterministically.
- Lazy tokenizer descriptor that does not instantiate tokenizer state until encode/count usage.
- Token usage addition and per-model cost accounting compatible with upstream `cost.py` semantics.
- Unit tests and parity labels for these contracts.

**Out of scope**:
- Pulling in Python, tiktoken, HuggingFace, or diskcache runtime dependencies.
- Live provider token accounting.
- Full persistent disk cache backend; task 18.3 owns backend registry/disk-cache model.

## 4. Actors

- Rust caller caching provider calls.
- Release owner checking upstream runtime parity.
- Maintainer extending provider cost accounting.

## 5. Behavior Contract

### 5.1 Required Reading

- docs/prds/ragas-rs-perfect-refactor.prd.md
- docs/specs/ragas-latest-gap-analysis.md
- test/features/runtime-cache-tokenizer-cost.feature

### 5.2 Imports

Use `src/runtime.rs` and existing public crate exports unless tests justify a smaller helper module.

### 5.3 Function Signatures

RED tests own final signatures; public exports must be added through `src/lib.rs`.

## 6. Acceptance Criteria

- **AC1**: Cache key generation is deterministic for nested JSON-like inputs and excludes `callbacks` by default.
- **AC2**: Lazy tokenizer reports deferred initialization until encode/count is first used and then remains initialized.
- **AC3**: Token usage addition rejects different concrete models and cost accounting supports one-model and per-model rates.

## 7. Traceability

| AC | Scenario | Test ID | Status |
|---|---|---|
| AC1 | SCEN-18.1.1 | TEST-18.1.1 | Not Started |
| AC2 | SCEN-18.1.2 | TEST-18.1.2 | Not Started |
| AC3 | SCEN-18.1.3 | TEST-18.1.3 | Not Started |

## 8. Risks

- A simple tokenizer approximation can be mistaken for full tiktoken parity if not labelled clearly.
- Cache keys must avoid callback/credential fields to prevent unstable keys and sensitive data persistence.

## 9. Verification Plan

- install
- typecheck
- unit-test
- build
- extra: `cargo test runtime::tests::test_18_1`

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：<TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
