pub mod backends;
pub mod benchmarks;
pub mod cli;
pub mod dataset;
pub mod docs_examples;
pub mod error;
pub mod eval;
pub mod experiments;
pub mod integrations;
pub mod llm;
pub mod metric;
pub mod metrics;
pub mod optimizers;
pub mod parity;
pub mod prompts;
pub mod providers;
pub mod release;
pub mod runtime;
pub mod schema;
pub mod testset;
pub mod validation;

pub use backends::{
    CsvDatasetBackend, DatasetBackend, InMemoryDatasetBackend, JsonlDatasetBackend,
};
pub use benchmarks::{
    BenchmarkMeasurement, BenchmarkPrompt, BenchmarkProvider, BenchmarkReport, CostRates,
    CostSummary, run_provider_benchmark,
};
pub use cli::{CliCommand, CliOutput, CliRuntime, run_cli_command};
pub use dataset::{
    EvaluationDataset, EvaluationDatasetBuilder, EvaluationSample, SingleTurnSample,
};
pub use docs_examples::{DocExample, public_workflow_examples};
pub use error::RagasError;
pub use eval::{EvaluationOptions, EvaluationReport, SampleEvaluation, evaluate};
pub use experiments::{
    ExperimentRecord, ExperimentSummary, RunComparison, compare_runs, summarize_experiment,
};
pub use integrations::{
    IntegrationDestination, IntegrationEvent, IntegrationFeatureRegistry, IntegrationPayload,
    TracingIntegration, redact_payload,
};
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
    MetricSampleKind, MetricValueType, MultiTurnMetric, MultimodalMetricKind, ParityStatus,
    QuotedSpan, RubricCriterion, RubricMetric, ScoreNormalizationPolicy, SemanticThresholdPolicy,
    SingleTurnMetric, SqlJudgeVerdict, SummarizationSignals, ToolCallOrderPolicy, TopicAdherence,
    agent_goal_accuracy, answer_correctness, answer_relevancy_from_embedding_similarity,
    answer_relevancy_from_judge_output, bleu_unigram, chrf_score, context_entity_recall,
    context_precision_from_relevance, context_recall, context_relevance, exact_match,
    extract_quoted_spans, factual_correctness, id_based_context_precision,
    lexical_tokenizer_assumptions, multimodal_metric_from_prompt, noise_sensitivity,
    normalize_score, quoted_citation_coverage, quoted_span_overlap, response_groundedness,
    rouge_l_recall, score_aspect_critic, semantic_similarity_batch,
    semantic_similarity_from_vectors, sql_semantic_equivalence, string_distance_similarity,
    summarization_score_from_judge_output, threshold_semantic_similarity, tool_call_accuracy,
    tool_call_f1, topic_adherence,
};
pub use optimizers::{
    CandidateGenerator, GeneticOptimizer, GeneticOptimizerConfig, ObjectiveMetric,
    OptimizationCandidate, OptimizationResult, OptimizationStep, Optimizer,
};
pub use parity::{
    GapMatrixEntry, ParityCheck, ParityFeatureStatus, ParityFixture, check_parity_fixture,
    parse_parity_fixture, validate_gap_matrix,
};
pub use prompts::{
    FewShotExample, JudgeOutputParser, LanguageAdapterRule, MultimodalPromptMessage,
    MultimodalPromptPart, OutputParseDiagnostic, ParsedJudgeOutput, PromptTemplate, PromptValue,
    PromptValueKind, PromptVariables, RenderedPrompt, RepairStrategy,
};
pub use providers::{
    MockEmbeddingProvider, MockLlmProvider, ProviderRegistry, record_provider_usage,
};
pub use release::release_gate_files;
pub use runtime::{
    AsyncExecutor, CacheKey, CallbackManager, CancellationConfig, ExecutorJobResult,
    ExecutorOutcome, ExecutorReport, ProgressEvent, ProgressEventKind, RetryConfig, RunConfig,
    RunConfigBuilder, RunConfigError, RuntimeEvent, RuntimeEventKind, TimeoutConfig, UsageSummary,
    UsageTotals, UsageTracker,
};
pub use schema::{Message, MessageRole, MultiTurnSample, Rubric, ToolCall};
pub use testset::{
    ExtractionBundle, GraphEdge, GraphNode, GraphProperty, KnowledgeGraph, Persona,
    PersonaGenerator, SynthesizedSample, TextChunk, attach_extractions, build_chunk_relationships,
    split_text_into_chunks, synthesize_multi_hop_sample, synthesize_single_hop_sample,
};
pub use validation::{
    MetricRequirements, SampleField, ValidationIssue, ValidationReport, validate_before_evaluate,
    validate_dataset_requirements, validate_single_turn_samples,
};
