Feature: Integration contract parity closure
  Current-upstream integration families must have executable Rust contract evidence before integration release blockers can be closed.

  Scenario: SCEN-27.1.1 integration contract descriptors cover tracked upstream families
    Given the current upstream integration files
    When Rust builds integration contract descriptors
    Then LangChain, LangGraph, LangSmith, LlamaIndex, AG-UI, Bedrock, Griptape, Helicone, Langfuse, Opik, R2R, and Swarm have explicit upstream module, boundary mode, target operation, auth/redaction, lifecycle field, and fixture metadata

  Scenario: SCEN-27.1.2 deterministic export plans preserve lifecycle payload semantics
    Given representative runtime events and sensitive payload fields
    When Rust plans integration exports
    Then event kind, run id, metric name, sample index, target operation, and redacted credentials are preserved in the export plan

  Scenario: SCEN-27.1.3 integration parity claims no longer block release
    Given fixture-backed integration contract descriptors
    When the consolidated release blocker ledger is built
    Then integration parity claims are Complete and the ledger has no Integration category entries
