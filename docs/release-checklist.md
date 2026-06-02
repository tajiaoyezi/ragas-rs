# Release Checklist

## Versioning

- Update `Cargo.toml` using semver before tagging.
- Run `cargo build`, `cargo check`, `cargo test`, and `cargo test parity::`.
- Run `cargo build --examples` before publishing documentation examples.
- Run `cargo publish --dry-run` before creating a release tag.

## Packaging

- Confirm feature flags: `default`, `runtime-tokio`, `providers-openai`, `integrations`, `benchmarks`, `parity`, and `docs-examples`.
- Confirm `docs/ragas-rs-user-guide.md` lists known parity gaps.
- Confirm no Python runtime bridge or API key storage is introduced.

## No-known-bug audit

- Confirm the bug ledger reports zero unresolved release-blocking bugs.
- Treat unresolved critical/high correctness, safety, data-loss, panic, security, and parity defects as release blockers.
- Confirm every fixed release-blocking bug has a regression test reference before release readiness is reported.

## Final audit evidence

- Confirm build evidence from `cargo build`.
- Confirm check evidence from `cargo check`.
- Confirm unit evidence from `cargo test`.
- Confirm parity evidence from `cargo test parity::`.
- Confirm examples evidence from `cargo build --examples`.
- Confirm quality gate evidence for coverage, fuzz/property, panic, mutation, platform, and E2E gates.
- Confirm blocker ledger status from the consolidated release blocker ledger.
- Confirm bug ledger status from the no-known-bug audit.
- Do not describe the crate as bug-free; report only no known unresolved release-blocking bugs within the verified scope.

## Rollback

- If a crate release is bad, run `cargo yank --vers <version>`.
- Communicate a dependency lock rollback for downstream users.
- Ship a patch release after the rollback cause is fixed and verified.
