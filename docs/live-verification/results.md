# Live verification — LLM metric discrimination gates

This records a real-provider run of the project's `#[ignore]` live discrimination gates, which
are the stated "done" gate for the LLM/embedding metrics: each one calls a real provider and
asserts that a faithful / correct / relevant sample scores **strictly higher** than an
adversarial one. Until such a run exists, the honest status of an LLM metric is
"math verified; real-LLM unverified" — this file is the evidence that converts that to verified.

## Run

- **Date:** 2026-06-06
- **Provider:** DeepSeek (chat) + SiliconFlow (embeddings), via OpenAI-compatible endpoints,
  configured in `.env` (gitignored — keys are never committed).
- **Command:** `cargo test --lib -- --ignored --test-threads=4`
- **Result:** **20 passed, 0 failed** (~137s).

### Follow-up run — Phase 4 LLM agentic metrics

- **Date:** 2026-06-06
- **Provider:** DeepSeek (chat), same configuration.
- **Command:** `cargo test --lib -- --ignored --test-threads=3 live_agent_goal_accuracy
  live_topic_adherence live_instance_specific_rubrics`
- **Result:** **3 passed, 0 failed** (~26s) — the three gates added in the rows marked **(P4)** below.

### Follow-up run — SemanticSimilarity (answer_similarity)

- **Date:** 2026-06-06
- **Provider:** SiliconFlow (embeddings), same configuration.
- **Command:** `cargo test --lib live_semantic_similarity_scores_similar_above_dissimilar -- --ignored`
- **Result:** **1 passed, 0 failed** (~1s) — the embedding-only gate marked **(SS)** below.

### Follow-up run — LLM testset extractor (Phase 6)

- **Date:** 2026-06-06
- **Provider:** DeepSeek (chat), same configuration.
- **Command:** `cargo test --lib live_llm_extractor_pulls_named_entities_and_summary -- --ignored`
- **Result:** **1 passed, 0 failed** (~8s) — the `LlmExtractor` gate marked **(EX)** below: the real
  model extracts the obvious named entities (Tesla + Musk/SpaceX) from an entity-rich passage and
  returns a non-empty summary shorter than the source.

### Follow-up run — embedding cosine relationship builder (Phase 6)

- **Date:** 2026-06-06
- **Provider:** SiliconFlow (embeddings), same configuration.
- **Command:** `cargo test --lib live_cosine_relationships_link_semantically_similar_nodes -- --ignored`
- **Result:** **1 passed, 0 failed** (~7s) — the `EmbeddingExtractor` + `build_cosine_relationships`
  gate marked **(CB)** below: real embeddings make two semantically similar sentences (two about
  cats) score strictly higher than an unrelated sentence (stock market), so the cosine edge is
  ordered correctly.

### Follow-up run — CustomNodeFilter (Phase 6)

- **Date:** 2026-06-07
- **Provider:** DeepSeek (chat), same configuration.
- **Command:** `cargo test --lib live_custom_node_filter_drops_irrelevant_chunk -- --ignored`
- **Result:** **1 passed, 0 failed** (~5s) — the `CustomNodeFilter` gate marked **(NF)** below: the
  real model scores an on-topic chunk high (kept) and an irrelevant advertisement chunk low
  (removed) against the document summary.

### Follow-up run — transforms engine (Phase 6)

- **Date:** 2026-06-07
- **Provider:** SiliconFlow (embeddings), same configuration.
- **Command:** `cargo test --lib live_apply_transforms_embed_then_cosine_pipeline -- --ignored`
- **Result:** **1 passed, 0 failed** (~1s) — the `apply_transforms` gate marked **(TE)** below: a
  real `[Embed(chunks) -> Cosine]` pipeline through the engine embeds the chunks (node-type filter
  skips the doc) and links the two similar chunks with a cosine edge, end-to-end.

### Follow-up run — KG persona generation (Phase 6)

- **Date:** 2026-06-07
- **Provider:** DeepSeek (chat), same configuration.
- **Command:** `cargo test --lib live_generate_personas_from_kg_builds_a_persona -- --ignored`
- **Result:** **1 passed, 0 failed** (~7s) — the `generate_personas_from_kg` gate marked **(PG)**
  below: the real model turns a clustered KG node summary into a usable persona (non-empty name +
  role description).

### Follow-up run — single-hop scenario prep (Phase 6)

- **Date:** 2026-06-07
- **Provider:** DeepSeek (chat), same configuration.
- **Command:** `cargo test --lib live_prepare_single_hop_scenarios_matches_relevant_persona -- --ignored`
- **Result:** **1 passed, 0 failed** (~4s) — the `prepare_single_hop_scenarios` gate marked **(SH)**
  below: the real model matches a node's astronomy entities (galaxy/telescope) to the astronomer
  persona, not the chef, so every produced scenario is anchored on the relevant persona.

### Follow-up run — single-hop specific synthesizer (Phase 6)

- **Date:** 2026-06-07
- **Provider:** DeepSeek (chat), same configuration.
- **Command:** `cargo test --lib live_single_hop_specific_synthesizer_generates_grounded_testset -- --ignored`
- **Result:** **1 passed, 0 failed** (~25s) — the `SingleHopSpecificSynthesizer::generate` gate
  marked **(SHS)** below: the real model turns an astronomy chunk + an astronomer/chef persona pair
  into a grounded single-hop test set end-to-end — every sample has a non-empty query, a grounded
  answer drawn from the passage (mirrored into `reference`), the source passage as its
  `reference_contexts`, and is anchored on the astronomer persona (not the chef).

### Follow-up run — multi-hop specific synthesizer (Phase 6)

- **Date:** 2026-06-07
- **Provider:** DeepSeek (chat), same configuration.
- **Command:** `cargo test --lib live_multi_hop_specific_synthesizer_generates_grounded_testset -- --ignored`
- **Result:** **1 passed, 0 failed** (~17s) — the `MultiHopSpecificSynthesizer::generate` gate
  marked **(MHS)** below: the real model turns an entity-overlap cluster (two chunks sharing the
  "Einstein" entity, joined by an `entities_overlap` edge) + a historian/chef persona pair into a
  grounded multi-hop test set end-to-end — every sample spans both hop-tagged contexts
  (`<1-hop>`/`<2-hop>`), carries a grounded answer, and is anchored on the historian (not the chef).

### Follow-up run — multi-hop abstract synthesizer (Phase 6)

- **Date:** 2026-06-07
- **Provider:** DeepSeek (chat), same configuration.
- **Command:** `cargo test --lib live_multi_hop_abstract_synthesizer_generates_grounded_testset -- --ignored`
- **Result:** **1 passed, 0 failed** (~28s) — the `MultiHopAbstractSynthesizer::generate` gate
  marked **(MHA)** below: the real model turns a similarity cluster (two themed docs joined by a
  `cosine_similarity` edge) + an analyst/chef persona pair into a grounded abstract multi-hop test
  set end-to-end — the LLM forms a cross-document concept combination, and every sample carries a
  grounded answer over hop-tagged contexts anchored on the analyst (not the chef).

### Follow-up run — TestsetGenerator orchestrator (Phase 6)

- **Date:** 2026-06-07
- **Provider:** DeepSeek (chat), same configuration.
- **Command:** `cargo test --lib live_testset_generator_generates_mixed_testset -- --ignored`
- **Result:** **1 passed, 0 failed** (~72s) — the `TestsetGenerator::generate` gate marked **(TG)**
  below: over a two-chunk graph carrying entity nodes + an `entities_overlap` edge + a
  `cosine_similarity` edge, the generator picks the default distribution (all three synthesizers),
  splits the requested size across them, runs each, and merges a non-empty test set where every
  sample is tagged with its `synthesizer_name` and carries a non-empty grounded answer + reference
  contexts — the full single/multi-hop testset stack end-to-end against a real model.

## Gates (all PASS)

LLM metric discrimination gates (20):

| metric | gate |
|---|---|
| Faithfulness | `live_faithfulness_scores_faithful_above_unfaithful` |
| ResponseRelevancy * | `live_answer_relevancy_scores_relevant_above_irrelevant` |
| SemanticSimilarity * (SS) | `live_semantic_similarity_scores_similar_above_dissimilar` |
| ContextPrecision | `live_context_precision_ranks_useful_context_above_useless` |
| LlmContextRecall | `live_context_recall_scores_supported_above_unsupported` |
| ContextUtilization | `live_context_utilization_ranks_useful_context_above_useless` |
| AspectCritic | `live_aspect_critic_discriminates_on_topic_answers` |
| SimpleCriteriaScore | `live_simple_criteria_scores_better_answer_higher` |
| SqlSemanticEquivalence | `live_sql_equivalence_scores_equivalent_above_different` |
| AnswerCorrectness * | `live_answer_correctness_scores_correct_above_incorrect` |
| AnswerAccuracy | `live_answer_accuracy_scores_correct_above_incorrect` |
| FactualCorrectness | `live_factual_correctness_scores_correct_above_incorrect` |
| ContextEntityRecall | `live_context_entity_recall_scores_present_above_absent` |
| ContextRelevance | `live_context_relevance_scores_relevant_above_irrelevant` |
| ResponseGroundedness | `live_response_groundedness_scores_grounded_above_ungrounded` |
| RubricsScore | `live_rubrics_score_scores_better_answer_higher` |
| SummarizationScore | `live_summarization_score_scores_faithful_summary_above_off_topic` |
| NoiseSensitivity | `live_noise_sensitivity_flags_noisy_response_above_clean` |
| AgentGoalAccuracy (multi-turn) **(P4)** | `live_agent_goal_accuracy_discriminates_achieved_from_failed` |
| TopicAdherence (multi-turn) **(P4)** | `live_topic_adherence_scores_adherent_above_non_adherent` |
| InstanceSpecificRubrics **(P4)** | `live_instance_specific_rubrics_scores_better_answer_higher` |

`*` also exercises the embedding endpoint. **(P4)** = Phase-4 agentic metrics, run in the
follow-up above.

End-to-end / testset gates (13):

- CLI `evaluate` with a live provider — `live_evaluate_command_runs_llm_metrics`
- LLM test-set synthesizer, single-hop — `test_31_2_7_live_synthesizer_generates_from_real_model`
- LLM test-set synthesizer, multi-hop — `test_31_2_8_live_multi_hop_synthesizer_generates_two_context_sample`
- LLM testset extractor **(EX)** — `live_llm_extractor_pulls_named_entities_and_summary`
- Embedding cosine relationship builder **(CB)** — `live_cosine_relationships_link_semantically_similar_nodes`
- CustomNodeFilter **(NF)** — `live_custom_node_filter_drops_irrelevant_chunk`
- Transforms engine **(TE)** — `live_apply_transforms_embed_then_cosine_pipeline`
- KG persona generation **(PG)** — `live_generate_personas_from_kg_builds_a_persona`
- Single-hop scenario prep **(SH)** — `live_prepare_single_hop_scenarios_matches_relevant_persona`
- Single-hop specific synthesizer **(SHS)** — `live_single_hop_specific_synthesizer_generates_grounded_testset`
- Multi-hop specific synthesizer **(MHS)** — `live_multi_hop_specific_synthesizer_generates_grounded_testset`
- Multi-hop abstract synthesizer **(MHA)** — `live_multi_hop_abstract_synthesizer_generates_grounded_testset`
- TestsetGenerator orchestrator **(TG)** — `live_testset_generator_generates_mixed_testset`

## What this proves — and what it does NOT

- **Proves:** each metric, driven by a real LLM, discriminates a good sample from a bad one
  (correct ordering) on a representative example, and the full provider → metric → score path
  works against a real OpenAI-compatible API, embeddings included.
- **Does NOT prove:** numeric parity with Python ragas (an explicit non-goal — NumPy RNG,
  rounding, and tiktoken bins are out of scope); robustness across providers/models (one provider
  tested); or correctness on adversarial/edge inputs beyond the one example pair per gate. Treat
  scores as this library's own, not as drop-in ragas numbers.

To reproduce: set the provider env vars (see `.env.example`) and run the command above. The gates
stay `#[ignore]` so normal `cargo test` (and CI without a key) skips them.
