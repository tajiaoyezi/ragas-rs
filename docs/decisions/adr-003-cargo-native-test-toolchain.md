# ADR 003: cargo-native-test-toolchain

**Status**: Accepted
**Date**: 2026-05-31
**Category**: 测试工具链

## Context

The goal requires auditable S2V verification while keeping the greenfield Rust project simple.

## Decision

Use Cargo native commands as the baseline green suite: `cargo build`, `cargo check`, and `cargo test`.

## Rationale

Cargo is present with the Rust toolchain and does not add another runner or runtime. It satisfies S2V baseline verification and can be broadened later.

## Alternatives

- cargo-nextest: rejected for v1.0 to avoid an extra tool dependency.
- tarpaulin coverage: rejected because coverage is not required for v1.0 and can be added later.
- cargo-deny: rejected because dependency policy is not yet defined.

## Consequences

Coverage and lint are explicitly N/A in the adapter for v1.0.

## Rollback Or Migration Plan

Add lint/coverage commands and ADR update when CI policy exists.

## Follow-ups

Consider clippy and cargo-deny before public release hardening.
