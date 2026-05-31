# language: en
# Maps to:
#   - docs/specs/tasks/task-11.1-lexical.md

Feature: lexical
  In order to complete the Rust refactor of ragas
  As a ragas-rs maintainer
  I want exact match/string distance/BLEU/ROUGE/CHRF

  Scenario: SCEN-11.1.1 Exact/string metrics are deterministic and provider-free
    Given the complete refactor task 11.1
    When TEST-11.1.1 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-11.1.2 BLEU/ROUGE/CHRF expose documented tokenizer assumptions
    Given the complete refactor task 11.1
    When TEST-11.1.2 is executed
    Then the behavior matches the task acceptance criterion
  Scenario: SCEN-11.1.3 Traditional metrics handle empty strings explicitly
    Given the complete refactor task 11.1
    When TEST-11.1.3 is executed
    Then the behavior matches the task acceptance criterion
