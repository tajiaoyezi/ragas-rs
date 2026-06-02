Feature: Provider protocol parity closure
  Current-upstream provider families must have executable Rust protocol evidence before provider release blockers can be closed.

  Scenario: SCEN-26.1.1 provider protocol descriptors cover tracked upstream families
    Given the current upstream provider files under ragas llms and embeddings
    When Rust builds provider protocol descriptors
    Then OpenAI-compatible, Azure OpenAI, LiteLLM, Instructor, Haystack, HuggingFace, Google, and OCI GenAI have explicit auth, endpoint, kind, structured-output, and fixture metadata

  Scenario: SCEN-26.1.2 deterministic request plans preserve provider payload semantics
    Given a representative LLM, embedding, and structured-output request
    When Rust plans provider protocol calls
    Then provider plans preserve model, messages, inputs, schema metadata, and usage extraction paths without exposing authorization secrets

  Scenario: SCEN-26.1.3 provider parity claims no longer block release
    Given fixture-backed provider protocol descriptors
    When the consolidated release blocker ledger is built
    Then provider parity claims are Complete and the ledger has no Provider category entries
