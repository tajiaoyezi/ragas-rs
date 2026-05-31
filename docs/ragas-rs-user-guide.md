# ragas-rs User Guide

This guide maps the Rust examples to the upstream ragas howtos and tutorials.

## Examples

| Workflow | Example | Upstream docs section |
|---|---|---|
| Evaluate | `examples/evaluate.rs` | Evaluate a RAG application |
| Testset | `examples/testset.rs` | Generate a testset |
| Benchmark | `examples/benchmark.rs` | Compare and monitor evaluation cost |

## Feature flags

- `default`: core dataset, metric, provider, runtime, parity, benchmark, and docs example APIs.
- `tokio`: examples use the caller-provided Tokio runtime for async provider and evaluation flows.

## Known parity gaps

- no Python runtime bridge: v1.0 is Rust-first and does not expose pyo3 or Python interop.
- knowledge graph generation: graph and synthesizer scaffolds exist, but full upstream graph generation remains a known gap.
- provider latency percentiles: benchmark output currently records usage and cost summaries, not p50/p95 latency.
