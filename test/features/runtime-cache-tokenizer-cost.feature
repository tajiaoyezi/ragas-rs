Feature: runtime cache tokenizer cost
  Scenario: deterministic cache key excludes callbacks
    Given nested cache input containing callbacks
    When the runtime cache key is generated
    Then callback fields do not affect the key

  Scenario: default tokenizer initializes lazily
    Given a lazy tokenizer descriptor
    When no tokenization operation has run
    Then it reports uninitialized

  Scenario: token usage cost mirrors upstream rules
    Given token usage for model families
    When usage is added and priced
    Then different concrete models are rejected and configured rates are applied

