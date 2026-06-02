# ragas-rs perfect refactor PRD

**Slug**: ragas-rs-perfect-refactor  
**Version**: v2.0  
**Date**: 2026-06-01  
**Status**: Active

> This PRD supersedes `docs/prds/ragas-rs-complete-refactor.prd.md` for the active goal. The prior PRD completed a broad Rust module map for upstream commit `298b682`; this PRD raises the bar to current-upstream full functional parity plus a stronger test and release evidence model.

## 1. Current Baseline

- Upstream repository: `vibrantlabsai/ragas`.
- Upstream default branch `main`: `298b68274234c060deacab3cf5fb52aa3a20e885`.
- Latest upstream release tag observed on 2026-06-01: `v0.4.3` at `4ecab384fda829ca50bec3f07cc49589d756e172`.
- Current Rust repository state before this PRD: 16 phases, 40 tasks, 121 Rust tests passing, but only one tracked parity fixture and several documented `KnownGap` / `Partial` statuses.

## 2. Product Goal

Rebuild ragas as a Rust-first evaluation framework whose behavior is demonstrably aligned with the current upstream Python project, not merely structurally similar. Completion requires full module-level coverage, metric-level golden parity, provider and backend behavior coverage, integration-facing contract coverage, and an evidence-based "no known bugs" release gate.

"No potential bugs" is operationalized as: no known unresolved correctness, safety, data-loss, panic, security, or parity defects in the verified scope; every unsupported upstream behavior is either implemented or blocks release. The project must not claim mathematical absence of bugs without evidence.

## 3. Users And Workflows

- Rust platform teams embedding evaluation into production services.
- Evaluation framework maintainers tracking upstream ragas semantic parity.
- Application teams running CLI, batch evaluation, experiments, optimizers, and testset generation.
- QA and release owners requiring hard evidence before using the Rust crate as a Python ragas replacement.

Critical workflows:

1. Freeze upstream latest baseline and generate a complete feature inventory.
2. Run Rust equivalents for upstream dataset, provider, metric, testset, optimizer, backend, integration, CLI, and prompt behaviors.
3. Compare Rust outputs against Python upstream golden fixtures.
4. Run unit, integration, parity, property/fuzz, coverage, and release audit gates.
5. Block release if any feature is unimplemented, untested, or only a semantic approximation without explicit approval.

## 4. Scope

In scope:

- Top-level upstream runtime and utility modules: `evaluation`, `executor`, `run_config`, `cache`, `callbacks`, `cost`, `dataset`, `dataset_schema`, `messages`, `tokenizers`, `validation`, `experiment`, `sdk`, `cli`, and utilities.
- Upstream directories: `backends`, `embeddings`, `integrations`, `llms`, `metrics`, `optimizers`, `prompt`, and `testset`.
- Release `v0.4.3` deltas: DSPy optimizer / MIPROv2, DSPy caching, system prompt support for Instructor/LiteLLM structured LLMs, remaining quickstart templates, FactualCorrectness language adaptation, DiskCacheBackend compatibility behavior, and lazy tokenizer initialization semantics.
- Full metric catalog parity across collection metrics and legacy metrics.
- Testset generation parity: graph, transforms, extractors, splitters, relationship builders, personas, single-hop, multi-hop, and pre-chunked generation.
- Backend and provider behavior parity, with real-network tests behind explicit feature gates and deterministic mocks for default CI.
- Documentation, examples, quickstarts, and CLI workflows mapped to upstream.
- Strong verification: unit, integration, E2E, golden parity, property/fuzz, panic safety, coverage, mutation where practical, and cross-platform CI.

Out of scope:

- Python binary/API compatibility via pyo3, embedding Python, or shipping a Python runtime inside the Rust crate.
- Reproducing Python implementation internals when a Rust-native design proves the same public behavior through golden parity.
- Hosted dashboards or managed SaaS.

## 5. Non-Negotiable Success Criteria

- Every upstream `src/ragas` module and test category has a Rust owner task and an implementation status.
- Every metric exposed by upstream has at least one golden fixture; `ParityComplete` requires fixture evidence, not a label.
- No `KnownGap`, `Partial`, `NotStarted`, or unclassified upstream feature remains at release.
- `cargo build`, `cargo check`, `cargo test`, `cargo test parity::`, `cargo build --examples`, and every added quality gate pass from the repository root.
- Default crate still builds without Python, Node, JVM, or external service runtime.
- Live provider/integration tests are opt-in and never required for default deterministic CI.
- Release checklist includes upstream baseline hashes, test evidence, coverage evidence, bug ledger status, and rollback plan.

## 6. Phase Plan

| Phase | Name | Goal | Initial status |
|---|---|---|---|
| 17 | latest-baseline-and-quality-gates | Freeze current upstream baseline, replace informal gaps with a machine-readable parity inventory, and define release-blocking quality gates. | Done |
| 18 | provider-backend-runtime-parity | Complete provider, backend, cache, tokenizer, cost, callback, and integration-facing parity. | Done |
| 19 | metric-catalog-golden-parity | Drive every upstream metric to fixture-backed parity or block release. | Done |
| 20 | testset-generation-full-parity | Complete graph, transforms, synthesizers, personas, and pre-chunked generation parity. | Done |
| 21 | optimizer-experiment-cli-docs-parity | Complete DSPy/MIPROv2, experiment, SDK, CLI, quickstart, and docs parity. | Done |
| 22 | exhaustive-test-engineering | Add coverage, fuzz/property, mutation, E2E, and cross-platform evidence gates. | Done |
| 23 | release-candidate-bug-zero-audit | Resolve every open gap/bug and produce final release evidence. | Done |
| 24 | release-blocker-closure | Resolve concrete release blockers produced by the final audit ledger, starting with quickstart/example parity gaps that can be closed with deterministic Rust evidence. | Done |
| 25 | backend-gdrive-parity-closure | Close the remaining backend release blocker by implementing Google Drive / Sheets backend contracts through a deterministic Rust transport abstraction and fixture-backed parity evidence. | Done |
| 26 | provider-protocol-parity-closure | Close provider release blockers with deterministic provider protocol contracts and fixture-backed parity claims for every tracked upstream family. | Done |

## 7. First Delta Task Matrix

| Task | Phase | Module | Goal |
|---|---|---|---|
| 17.1 | 17 | parity | Create upstream latest inventory and classify every module/category. |
| 17.2 | 17 | parity/tests | Expand fixture policy so parity claims require golden data. |
| 17.3 | 17 | quality | Define stronger test gates and wire them into release evidence. |
| 17.4 | 17 | release | Establish no-known-bug ledger and release-blocking audit rules. |
| 18.1 | 18 | runtime | Implement upstream-compatible cache key, lazy tokenizer, and token cost accounting contracts. |
| 18.2 | 18 | providers | Implement provider adapter capability descriptors, system prompt support, and structured LLM contract metadata. |
| 18.3 | 18 | backends | Implement backend registry and disk-cache compatibility model for local deterministic CI. |
| 18.4 | 18 | integrations | Implement integration adapter registry, redaction policy, and callback payload contract coverage. |
| 19.1 | 19 | metrics | Implement upstream metric catalog inventory and owner descriptors. |
| 19.2 | 19 | parity/tests | Implement metric golden fixture runner and drift classification. |
| 19.3 | 19 | release | Aggregate metric catalog and fixture gaps into release blockers. |
| 20.1 | 20 | testset | Implement graph persistence and query parity contracts. |
| 20.2 | 20 | testset | Implement transform engine and extractor parity contracts. |
| 20.3 | 20 | testset | Implement synthesizer prompt snapshot and fixture parity contracts. |
| 21.1 | 21 | optimizers | Implement DSPy/MIPROv2/cache descriptors and release blockers. |
| 21.2 | 21 | cli | Implement experiment, SDK-facing, and CLI workflow contracts. |
| 21.3 | 21 | docs | Implement quickstart and docs parity descriptors. |
| 22.1 | 22 | quality | Implement property, fuzz, and coverage gate descriptors. |
| 22.2 | 22 | quality | Implement panic-safety and mutation gate descriptors. |
| 22.3 | 22 | quality | Implement cross-platform and E2E evidence matrix. |
| 23.1 | 23 | release | Aggregate all release blockers into one ledger. |
| 23.2 | 23 | release | Implement gap resolution and waiver policy. |
| 23.3 | 23 | release | Implement final bug-zero release audit evidence checks. |
| 24.1 | 24 | docs/examples | Close the `docs::quickstart::experiments` release blocker with a runnable Rust experiment example and fixture-backed parity claim. |
| 24.2 | 24 | backends | Close the `backend::disk-cache` release blocker with a deterministic persistent Rust disk cache and fixture-backed parity claim. |
| 24.3 | 24 | cli/sdk | Close the `workflow::sdk_facing` release blocker by recording that current upstream `src/ragas/sdk.py` is an empty module and mapping the Rust SDK surface to fixture-backed complete parity. |
| 25.1 | 25 | backends | Close the `backend::gdrive` release blocker with a deterministic Google Sheets-compatible backend transport, row roundtrip tests, and fixture-backed parity claim. |
| 26.1 | 26 | providers | Close the provider release-blocker category with deterministic provider protocol contracts, request-plan tests, and fixture-backed complete parity claims. |

## 8. Decisions

| # | Category | Decision | Rationale |
|---|---|---|---|
| D1 | Baseline | Treat `main` commit `298b682` as current source baseline and `v0.4.3` tag `4ecab38` as latest release baseline. | The active goal asks for current latest; both branch and release must be tracked to avoid silent drift. |
| D2 | Compatibility | Require semantic and behavioral parity, not Python API binary compatibility. | Rust-native API design should not embed Python runtime, but behavior must be proven by fixtures. |
| D3 | Quality | `ParityComplete` is illegal without upstream golden fixture evidence. | Labels are too weak; completion needs executable proof. |
| D4 | Testing | Default CI remains deterministic; live provider checks are opt-in feature gates. | Reliable CI must not depend on external services, keys, or network variability. |
| D5 | Release | Any unclassified, partial, known-gap, or failing parity feature blocks release. | This directly addresses the "complete functionality" and "no known bugs" requirement. |

## 9. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Upstream changes after baseline freeze | High | High | Record hashes and rerun inventory before release. |
| LLM judge output is inherently nondeterministic | High | High | Use deterministic mock fixtures plus tolerance policies and prompt snapshots. |
| Full provider parity requires external SDK behavior | Medium | High | Use protocol adapters, captured HTTP fixtures, and opt-in live tests. |
| Testset generation is large and LLM-driven | High | High | Split graph/transforms/synthesizer parity and require fixtures per stage. |
| "No potential bugs" is not mathematically provable | High | High | Enforce no known unresolved defects plus broad automated evidence; do not make unsupported claims. |

## 10. Open Questions

- Should release parity target upstream `main` only, latest release `v0.4.3` only, or both? Assumption for this PRD: both are tracked; release cannot ignore either.
- Which live providers can be used in opt-in CI? Assumption: none by default; all live checks are feature-gated and documented.
