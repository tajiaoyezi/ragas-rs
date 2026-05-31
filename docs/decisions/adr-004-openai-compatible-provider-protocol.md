# ADR 004: openai-compatible-provider-protocol

**Status**: Accepted
**Date**: 2026-05-31
**Category**: 协议接口

## Context

ragas-rs needs an initial LLM/Embedding provider that covers common production deployments without binding to one vendor SDK.

## Decision

Implement OpenAI-compatible HTTP chat completions and embeddings DTOs first.

## Rationale

OpenAI-compatible APIs are widely supported by hosted and self-hosted model providers. HTTP DTOs avoid SDK lock-in and keep tests network-free via parser and trait mocks.

## Alternatives

- Python ragas provider API: rejected because Python interop is out of scope.
- LangChain provider adapters: rejected because they bring a non-Rust abstraction stack.
- Multiple vendor SDKs: rejected because v1.0 needs a small surface.

## Consequences

Some provider-specific fields are ignored in v1.0.

## Rollback Or Migration Plan

Add provider-specific adapters as optional modules without changing the base traits.

## Follow-ups

Record known compatible endpoints during real integration testing.
