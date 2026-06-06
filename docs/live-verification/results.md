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

## Gates (all PASS)

LLM metric discrimination gates (17):

| metric | gate |
|---|---|
| Faithfulness | `live_faithfulness_scores_faithful_above_unfaithful` |
| ResponseRelevancy * | `live_answer_relevancy_scores_relevant_above_irrelevant` |
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

`*` also exercises the embedding endpoint.

End-to-end gates (3):

- CLI `evaluate` with a live provider — `live_evaluate_command_runs_llm_metrics`
- LLM test-set synthesizer, single-hop — `test_31_2_7_live_synthesizer_generates_from_real_model`
- LLM test-set synthesizer, multi-hop — `test_31_2_8_live_multi_hop_synthesizer_generates_two_context_sample`

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
