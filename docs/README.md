# ragas-rs documentation

Documentation index for `ragas-rs` (crate `ragas`) — a Rust port of the ragas LLM-evaluation toolkit aiming for functional, not numeric, parity.

## Getting started

- [Project README](../README.md) — install, quickstart, and the full feature/metric overview.
- [Architecture](ARCHITECTURE.md) — module map, evaluation data flow, the provider/resilience layer, and the test-set pipeline.
- [Examples](../examples/README.md) — six runnable offline examples (`cargo run --example <name>`), all driven by mock/scripted providers.

## Architecture decision records (ADRs)

- [ADR-001: trait-layering](decisions/adr-001-trait-layering.md) — trait-layered module boundaries so callers inject custom metrics/mock providers without global registries.
- [ADR-002: rust-async-http-dependencies](decisions/adr-002-rust-async-http-dependencies.md) — standardize on tokio/reqwest/serde/async-trait/thiserror for async HTTP and JSON.
- [ADR-003: cargo-native-test-toolchain](decisions/adr-003-cargo-native-test-toolchain.md) — plain `cargo build`/`check`/`test` as the baseline green suite; coverage and lint N/A for v1.0.
- [ADR-004: openai-compatible-provider-protocol](decisions/adr-004-openai-compatible-provider-protocol.md) — OpenAI-compatible chat-completions + embeddings DTOs to cover most providers without vendor lock-in.
- [ADR-005: cargo-library-release-model](decisions/adr-005-cargo-library-release-model.md) — ship as an embeddable Cargo library crate; no server, Docker image, or hosted panel in v1.0.

## Status & verification

- [Parity roadmap](parity-roadmap.md) — Phases 1–6 (all done), the ~35 real metrics vs. Python's ~39, and the deferred/infeasible tail.
- [Live verification results](live-verification/results.md) — 37 live gates, 0 failures, against DeepSeek (chat) + SiliconFlow (embeddings).

## Contributing & policy

- [Contributing](../CONTRIBUTING.md) — how to build, test, and submit changes.
- [Code of conduct](../CODE_OF_CONDUCT.md) — expected behavior for participants.
- [Security policy](../SECURITY.md) — how to report vulnerabilities.
- [Changelog](../CHANGELOG.md) — notable changes per release.
