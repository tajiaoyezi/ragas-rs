# Security Policy

`ragas-rs` is an embeddable Rust library for evaluating RAG/LLM applications. This
document describes how to report a vulnerability and what is in (and out of) scope.

## Supported versions

The project is **pre-release** (`0.1.x`). Security fixes are provided only against the
latest mainline.

| Version | Status | Security fixes |
|---|---|---|
| `main` branch | in development | ✅ yes |
| `0.1.x` | pre-release | 🟡 best-effort (upgrade to latest `main`) |
| `< 0.1.0` | n/a | — |

A standard N / N-1 policy will replace this table after the first stable release.

## Reporting a vulnerability

**Please do not report security issues in public GitHub Issues.**

Report privately via a
**[GitHub Security Advisory](https://github.com/tajiaoyezi/ragas-rs/security/advisories/new)**.
The maintainer is notified and will collaborate with you on a private fix before any
public disclosure. Please include:

- the affected version (a commit hash on `main` is best);
- the vulnerability class (e.g. SSRF, secret/key leak, deserialization, panic / DoS);
- reproduction steps — a minimal Rust snippet or CLI invocation plus the input that
  triggers it;
- the impact (what an attacker gains), and an optional proof of concept.

## Response timeline

| Stage | Target |
|---|---|
| Acknowledge receipt | 72 hours |
| Triage + severity rating | 7 days |
| Fix (critical / high) | 30 days |
| Fix (medium / low) | 90 days |
| Coordinated disclosure | on fix release, or 90 days maximum |

This is a small pre-release project maintained in spare time; we will tell you the
realistic timeline in the first response.

## Threat model — what to look at

`ragas-rs` is a library, not a network service. The relevant surfaces are:

- **Provider secrets.** API keys come from environment variables or a git-ignored `.env`
  (see `.env.example`). The provider layer is designed to keep keys out of its outbound
  request path: the request plan's `safe_debug` masks them and auth headers are redacted.
  Any path that leaks a key into logs, errors, or a `Debug` output is a vulnerability —
  please report it.
- **Outbound HTTP / SSRF.** The OpenAI-compatible client calls a **caller-configured**
  `base_url` (`{base}/chat/completions`, `{base}/embeddings`). The base URL is trusted
  configuration; a way to make the client reach an *unintended* endpoint from *untrusted
  input* (dataset or document text) would be in scope.
- **Untrusted input handling.** Datasets (JSONL / CSV) and source documents are parsed
  and fed into prompts. A crafted input that causes a panic, unbounded memory use, or a
  denial of service in parsing / scoring is in scope.

## Out of scope

- **Prompt injection in evaluated content.** Steering an LLM judge via adversarial
  document / answer text is an inherent property of LLM evaluation, not a library
  vulnerability. Treat scores over adversarial inputs accordingly.
- Vulnerabilities in third-party dependencies that already have a published CVE — please
  open a PR bumping the dependency (or let Dependabot do it).
- Issues that require a malicious `base_url` or API key that the operator configured
  themselves (trusted configuration).
- Numeric divergence from Python ragas (an explicit non-goal — not a security issue).

## Coordinated disclosure

We fix privately, release, then publish a
[GitHub Security Advisory](https://github.com/tajiaoyezi/ragas-rs/security/advisories)
crediting the reporter (unless you prefer to stay anonymous) and note the fix in the
`Security` section of [CHANGELOG.md](CHANGELOG.md) without a detailed PoC. This is an
Apache-2.0 hobby project with no bug bounty — but every responsible disclosure is
genuinely appreciated.
