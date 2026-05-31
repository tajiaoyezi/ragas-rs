pub mod dataset;
pub mod error;
pub mod eval;
pub mod llm;
pub mod metric;
pub mod metrics;
pub mod prompts;
pub mod providers;
pub mod runtime;
pub mod schema;
pub mod validation;

pub use dataset::{
    EvaluationDataset, EvaluationDatasetBuilder, EvaluationSample, SingleTurnSample,
};
pub use error::RagasError;
pub use eval::{EvaluationOptions, EvaluationReport, SampleEvaluation, evaluate};
pub use llm::{
    AzureOpenAiConfig, ChatMessage, EmbeddingAdapter, EmbeddingProvider, EmbeddingRequest,
    EmbeddingResponse, LlmProvider, LlmRequest, LlmResponse, OpenAiCompatibleClient,
    OpenAiCompatibleConfig, TokenUsage, normalize_embedding_vector, parse_chat_response,
    parse_embedding_response,
};
pub use metric::{
    ContextPrecisionMetric, FaithfulnessMetric, FnMetric, Metric, MetricResult, MetricValue,
    RankingItem, ResponseRelevancyMetric, cosine_similarity,
};
pub use metrics::{
    AgentGoalOutcome, AnswerCorrectnessWeights, AspectCriticConfig, AspectCriticMode,
    ContextPrecisionVariant, DetailedMetricResult, DomainRubric, FactualCorrectnessCounts,
    FaithfulnessJudgeContract, InstanceRubric, MetricError, MetricErrorKind, MetricEvidence,
    MetricMetadata, MetricProviderRequirement, MetricRegistry, MetricRegistryEntry,
    MetricSampleKind, MetricValueType, MultiTurnMetric, ParityStatus, QuotedSpan, RubricCriterion,
    RubricMetric, ScoreNormalizationPolicy, SemanticThresholdPolicy, SingleTurnMetric,
    ToolCallOrderPolicy, TopicAdherence, agent_goal_accuracy, answer_correctness,
    answer_relevancy_from_embedding_similarity, answer_relevancy_from_judge_output, bleu_unigram,
    chrf_score, context_entity_recall, context_precision_from_relevance, context_recall,
    context_relevance, exact_match, extract_quoted_spans, factual_correctness,
    id_based_context_precision, lexical_tokenizer_assumptions, noise_sensitivity, normalize_score,
    quoted_citation_coverage, quoted_span_overlap, response_groundedness, rouge_l_recall,
    score_aspect_critic, semantic_similarity_batch, semantic_similarity_from_vectors,
    string_distance_similarity, threshold_semantic_similarity, tool_call_accuracy, tool_call_f1,
    topic_adherence,
};
pub use prompts::{
    FewShotExample, JudgeOutputParser, LanguageAdapterRule, MultimodalPromptMessage,
    MultimodalPromptPart, OutputParseDiagnostic, ParsedJudgeOutput, PromptTemplate, PromptValue,
    PromptValueKind, PromptVariables, RenderedPrompt, RepairStrategy,
};
pub use providers::{
    MockEmbeddingProvider, MockLlmProvider, ProviderRegistry, record_provider_usage,
};
pub use runtime::{
    AsyncExecutor, CacheKey, CallbackManager, CancellationConfig, ExecutorJobResult,
    ExecutorOutcome, ExecutorReport, ProgressEvent, ProgressEventKind, RetryConfig, RunConfig,
    RunConfigBuilder, RunConfigError, RuntimeEvent, RuntimeEventKind, TimeoutConfig, UsageSummary,
    UsageTotals, UsageTracker,
};
pub use schema::{Message, MessageRole, MultiTurnSample, Rubric, ToolCall};
pub use validation::{
    MetricRequirements, SampleField, ValidationIssue, ValidationReport, validate_before_evaluate,
    validate_dataset_requirements, validate_single_turn_samples,
};
