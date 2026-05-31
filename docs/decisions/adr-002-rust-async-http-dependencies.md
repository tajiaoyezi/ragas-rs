# ADR 002: rust-async-http-dependencies

**Status**: Accepted
**Date**: 2026-05-31
**Category**: 依赖

## Context

The provider layer needs async HTTP and JSON support while keeping runtime dependencies within the Rust binary.

## Decision

Use `tokio`, `reqwest`, `serde`, `async-trait`, and `thiserror`.

## Rationale

These are stable, common Rust ecosystem defaults. They let the project deliver OpenAI-compatible HTTP quickly without writing low-level HTTP/TLS code.

## Alternatives

- Direct `hyper`: rejected because it expands implementation scope.
- Synchronous `ureq`: rejected because the PRD requires async batch evaluation.
- Custom JSON parsing: rejected because serde gives safer DTO evolution.

## Consequences

Callers need a tokio runtime for async provider/evaluate paths.

## Rollback Or Migration Plan

If dependency weight becomes a release blocker, isolate `OpenAiCompatibleClient` behind a feature flag in a future task.

## Follow-ups

Reassess dependency feature flags before publishing to crates.io.
