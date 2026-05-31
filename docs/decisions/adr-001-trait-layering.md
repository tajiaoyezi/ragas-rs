# ADR 001: trait-layering

**Status**: Accepted
**Date**: 2026-05-31
**Category**: 架构

## Context

ragas-rs must be embeddable in Rust services and keep metric logic, provider IO, dataset modeling, and evaluation orchestration independently testable.

## Decision

Use trait-layered module boundaries: `dataset`, `metric`, `llm`, and `eval`.

## Rationale

Trait boundaries let callers inject custom metrics and mock providers without global registries or runtime plugins. This matches the PRD goal of a small, type-safe, embeddable core.

## Alternatives

- Global registry: rejected because it adds mutable global state and makes tests order-dependent.
- Monolithic evaluator object: rejected because provider and metric contracts would be harder to reuse.

## Consequences

Public traits become the semver surface. Changes must remain additive where possible.

## Rollback Or Migration Plan

If trait ergonomics prove too heavy, add helper adapters without removing the trait API.

## Follow-ups

Document v1.0 built-in metric semantics separately from Python ragas parity.
