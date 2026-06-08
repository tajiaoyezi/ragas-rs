<!--
  Pull request template · ragas-rs
  Keep the required sections; delete optional ones that don't apply.
  See CONTRIBUTING.md for the full workflow.
-->

## Summary

<!-- 1–3 sentences: what this PR does and why. -->

## Linked issue

<!-- `Closes #NN`, or "n/a (infra/docs)" with a reason. -->

- Closes #

## Type of change

- [ ] New metric / feature
- [ ] Bug fix
- [ ] Refactor (no behavior change)
- [ ] Docs only
- [ ] Build / CI / tooling

## Checks (run locally — these mirror CI)

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings` (default **and** `--features tokenizer`)
- [ ] `cargo test` passes (offline)

## Metric / LLM-path PRs only — the anti-gaming rule

<!-- Required if you add or change a metric. See CONTRIBUTING.md + docs/parity-roadmap.md. -->

- [ ] Ships as a real `impl Metric` / `impl MultiTurnMetric` (not a stub)
- [ ] Has an env-gated `#[ignore]` **live discrimination test** (a good sample scores
      strictly above an adversarial one) — gate name: `____`
- [ ] The live gate has actually passed against a real provider, and is recorded in
      `docs/live-verification/results.md` — **or** the change is deterministic and offline
      tests are the gate (explain below)
- [ ] Any deterministic lexical fallback is kept (not deleted)

## Documented divergences from Python ragas

<!-- List any deliberate divergence (RNG, rounding, tiktoken bins, an algorithm shortcut).
     "None" is a valid answer. -->

- None

## Breaking changes

- [ ] N/A
- [ ] Yes — described below (with migration notes)

## Notes for reviewers

<!-- Optional: anything that needs extra eyes. -->
