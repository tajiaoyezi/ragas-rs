# Contributing to ragas-rs

Thanks for your interest in `ragas-rs` — a Rust core for evaluating RAG and LLM
applications, inspired by the Python [`ragas`](https://github.com/explodinggradients/ragas)
library. Code, docs, bug reports, and feature ideas are all welcome.

**Apache-2.0 · no CLA.** By contributing you agree your work is licensed under the
project's [Apache License 2.0](LICENSE). There is no separate contributor agreement.

## Ways to contribute

| I want to… | Go to | Notes |
|---|---|---|
| Report a bug | [Issues → 🐛 Bug Report](https://github.com/tajiaoyezi/ragas-rs/issues/new/choose) | `.github/ISSUE_TEMPLATE/bug_report.yml` |
| Request a feature / metric | [Issues → ✨ Feature Request](https://github.com/tajiaoyezi/ragas-rs/issues/new/choose) | check the parity roadmap first |
| Ask a question / discuss | [Discussions](https://github.com/tajiaoyezi/ragas-rs/discussions) | not an issue |
| Report a security vulnerability | [Security Advisory](https://github.com/tajiaoyezi/ragas-rs/security/advisories/new) (private) | ⚠️ **do not open an issue** — see [SECURITY.md](SECURITY.md) |
| Change code or docs | Fork → feature branch → PR | see [Pull requests](#commits--pull-requests) |

## Development setup

Requires a recent stable Rust toolchain (**edition 2024**, MSRV **1.88**).

```bash
git clone https://github.com/tajiaoyezi/ragas-rs.git
cd ragas-rs

cargo build                                   # build (offline, default features)
cargo test                                    # unit + binary tests (offline, deterministic)
cargo run --example quickstart_end_to_end     # full stack offline with mock providers
```

LLM/embedding features need an OpenAI-compatible provider. Copy the template and fill
in your keys (the file is git-ignored — never commit real keys):

```bash
cp .env.example .env          # PowerShell: Copy-Item .env.example .env
cargo run -- config           # verify the resolved provider config (redacted)
```

## Before you open a PR — run the same checks as CI

CI (`.github/workflows/ci.yml`) is the gate. Run it locally first:

```bash
cargo fmt --all -- --check
cargo build --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
# the optional tiktoken token-counting feature is covered separately:
cargo clippy --all-targets --features tokenizer -- -D warnings
cargo test --features tokenizer
```

A PR must be `fmt`-clean, `clippy -D warnings`-clean (default **and** `tokenizer`), and
all offline tests green.

## The one hard rule: no fake completeness

This project exists to *replicate ragas for real*, not to advertise features that don't
work. The execution rules in [`docs/parity-roadmap.md`](docs/parity-roadmap.md) are
mandatory and exist to prevent "self-certifying" metrics:

1. **Every new metric ships as a real `impl Metric` (or `impl MultiTurnMetric`) _plus_
   an env-gated `#[ignore]` live discrimination test** — a faithful / correct / relevant
   sample must score **strictly higher** than an adversarial one against a real provider.
   No `complete` / `parity` / `bug-zero` label until that gate has actually passed
   against a real LLM (record it in
   [`docs/live-verification/results.md`](docs/live-verification/results.md)).
2. **Keep the deterministic fallback.** Where Python has both an LLM and a non-LLM
   variant, don't delete the lexical function when you add the LLM version — keep it as a
   no-provider fallback.
3. **Don't chase byte-exact Python parity.** NumPy RNG, Python rounding, and tiktoken
   bin boundaries are explicit non-goals. If you diverge deliberately, **document the
   divergence** in code and in the commit message.

If you're adding or fixing a metric, your PR description should point at the live gate
(or explain why the change is deterministic and offline tests are the gate).

## Live (provider-backed) tests

Discrimination gates are marked `#[ignore]` so a normal `cargo test` (and CI without a
key) skips them. To run them you need provider keys configured (see `.env.example`):

```bash
cargo test -- --ignored                 # all live gates (cost tokens, need a network)
cargo test <gate_name> -- --ignored     # one gate
```

Tests that don't call a provider must stay **fully offline** — never resolve a live
provider from `.env` inside a non-`#[ignore]` test.

## Commits & pull requests

### Pull requests

- **Branch first — never push directly to `main`.** Open a feature branch
  (`<type>/<short-slug>`, e.g. `feat/answer-similarity` or `fix/cosine-length-guard`)
  and merge via PR.
- Keep PRs focused; every changed line should trace to the stated goal (no drive-by
  refactors of unrelated code).
- Fill in the [pull request template](.github/PULL_REQUEST_TEMPLATE.md).

### Commit messages

We use [Conventional Commits](https://www.conventionalcommits.org/) with a scope:

```
<type>(<scope>): <imperative summary>

<body: what changed and why; document any deliberate divergence from Python ragas;
note the test count and that clippy/fmt are clean>
```

Examples from the history: `feat(testset): …`, `fix(metric): …`,
`refactor(testset): …`, `docs(roadmap): …`. If an AI agent co-authored the change, add a
`Co-authored-by:` trailer.

## Architecture decisions

Significant decisions live in [`docs/decisions/`](docs/decisions/) as ADRs
(`adr-NNN-<slug>.md`). If your change makes a non-obvious architectural call (a new
dependency, a protocol choice, a deliberate divergence from ragas' design), add an ADR in
the same style as the existing ones and reference it from your PR.

## Code of conduct

Participation is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). Be kind.
