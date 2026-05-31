# ADR 005: cargo-library-release-model

**Status**: Accepted
**Date**: 2026-05-31
**Category**: 部署发布

## Context

The product goal is a Rust core that can be embedded in production inference paths as a single binary dependency.

## Decision

Ship ragas-rs as a Cargo library crate. Do not provide a server process, Docker image, or hosted panel in v1.0.

## Rationale

A library crate lets downstream Rust services compile evaluation logic into their existing binary, avoiding extra runtime dependencies.

## Alternatives

- Docker service: rejected because it adds a deployable service and network hop.
- CLI-only: rejected because production embedding is the primary use case.
- Hosted API: rejected because managed storage and panel are out of scope.

## Consequences

The crate API must be stable and documented enough for embedding callers.

## Rollback Or Migration Plan

Add optional CLI or service wrapper in a later PRD without changing the library core.

## Follow-ups

Decide crates.io package name availability before release.
