pub mod agentic;
pub mod backends;
pub mod benchmarks;
pub mod cli;
pub mod config;
pub mod dataset;
pub mod error;
pub mod eval;
pub mod experiments;
pub mod integrations;
pub mod llm;
pub mod metric;
pub mod metrics;
pub mod optimizers;
pub mod prompts;
pub mod providers;
pub mod resilience;
pub mod runtime;
pub mod schema;
pub mod testset;
pub mod validation;

pub use agentic::{
    AgentGoalAccuracyWithReferenceMetric, AgentGoalAccuracyWithoutReferenceMetric,
    ToolCallAccuracyMetric, ToolCallF1Metric, TopicAdherenceMetric, TopicAdherenceMode,
};
pub use backends::{
    CsvDatasetBackend, DatasetBackend, DiskCacheCompatibility, GDriveAuthMode, GDriveBackendConfig,
    GDriveSheetTransport, GoogleDriveDatasetBackend, InMemoryDatasetBackend,
    InMemoryGoogleSheetTransport, JsonlDatasetBackend,
};
pub use benchmarks::{
    BenchmarkMeasurement, BenchmarkPrompt, BenchmarkProvider, BenchmarkReport, CostRates,
    CostSummary, run_provider_benchmark,
};
pub use cli::{
    CliCommand, CliContractSnapshot, CliErrorSnapshot, CliOutput, CliRuntime,
    cli_contract_snapshot, cli_error_snapshot, run_cli_command, run_cli_command_with_provider,
};
pub use config::{
    DEFAULT_BASE_URL, DEFAULT_EMBEDDING_MODEL, DEFAULT_MODEL, ENV_API_KEY, ENV_BASE_URL,
    ENV_EMBEDDING_API_KEY, ENV_EMBEDDING_BASE_URL, ENV_EMBEDDING_MODEL, ENV_MODEL, ProviderConfig,
};
pub use dataset::{
    EvaluationDataset, EvaluationDatasetBuilder, EvaluationSample, SingleTurnSample,
};
pub use error::RagasError;
pub use eval::{
    EvaluationConfig, EvaluationOptions, EvaluationReport, SampleEvaluation, evaluate,
    evaluate_with, evaluate_with_config,
};
pub use experiments::{
    ExperimentRecord, ExperimentSummary, RunComparison, compare_runs, summarize_experiment,
};
pub use integrations::{
    IntegrationAuthMode, IntegrationBoundaryMode, IntegrationContractDescriptor,
    IntegrationDestination, IntegrationEvent, IntegrationExportInput, IntegrationExportPlan,
    IntegrationFamily, IntegrationFeatureRegistry, IntegrationPayload, TracingIntegration,
    integration_contract_descriptors, normalize_callback_payload, plan_integration_export,
    redact_payload,
};
pub use llm::{
    AzureOpenAiConfig, ChatMessage, EmbeddingAdapter, EmbeddingProvider, EmbeddingRequest,
    EmbeddingResponse, LlmProvider, LlmRequest, LlmResponse, OpenAiCompatibleClient,
    OpenAiCompatibleConfig, TokenUsage, normalize_embedding_vector, parse_chat_response,
    parse_embedding_response,
};
pub use metric::{
    AnswerAccuracyMetric, AnswerCorrectnessMetric, AspectCriticMetric, BleuScoreMetric,
    ChrfScoreMetric, ContextEntityRecallMetric, ContextPrecisionMetric, ContextRelevanceMetric,
    ContextUtilizationMetric, DataCompyMode, DataCompyScoreMetric, ExactMatchMetric,
    FactualCorrectnessMetric, FaithfulnessMetric, FnMetric, IdBasedContextPrecisionMetric,
    IdBasedContextRecallMetric, InstanceSpecificRubricsMetric, LlmContextRecallMetric, Metric,
    MetricResult, MetricValue, NoiseSensitivityMetric, NoiseSensitivityMode,
    NonLlmContextPrecisionMetric, NonLlmContextRecallMetric, RankingItem,
    ResponseGroundednessMetric, ResponseRelevancyMetric, RubricsScoreMetric,
    SemanticSimilarityMetric, SimpleCriteriaScoreMetric, SqlSemanticEquivalenceMetric,
    StringPresenceMetric, StringSimilarityMetric, SummarizationScoreMetric, cosine_similarity,
};
pub use metrics::{
    AgentGoalOutcome, AnswerCorrectnessWeights, AspectCriticConfig, AspectCriticMode,
    ContextPrecisionVariant, DetailedMetricResult, DistanceMeasure, DomainRubric,
    FactualCorrectnessCounts, FaithfulnessJudgeContract, InstanceRubric, MetricError,
    MetricErrorKind, MetricEvidence, MetricMetadata, MetricProviderRequirement, MetricRegistry,
    MetricRegistryEntry, MetricSampleKind, MetricValueType, MultiTurnMetric, MultimodalMetricKind,
    QuotedSpan, RougeMode, RougeScore, RougeScores, RougeType, RubricCriterion, RubricMetric,
    ScoreNormalizationPolicy, SemanticThresholdPolicy, SingleTurnMetric, SqlJudgeVerdict,
    SummarizationSignals, ToolCallOrderPolicy, TopicAdherence, agent_goal_accuracy,
    answer_correctness, answer_relevancy_from_embedding_similarity,
    answer_relevancy_from_judge_output, bleu_score, bleu_unigram, chrf, chrf_score,
    context_entity_recall, context_precision_from_relevance, context_recall, context_relevance,
    exact_match, extract_quoted_spans, factual_correctness, id_based_context_precision,
    lexical_tokenizer_assumptions, multimodal_metric_from_prompt, noise_sensitivity,
    normalize_score, quoted_citation_coverage, quoted_span_overlap, response_groundedness,
    rouge_l_recall, score_aspect_critic, semantic_similarity_batch,
    semantic_similarity_from_vectors, sql_semantic_equivalence, string_distance_similarity,
    string_distance_similarity_with, summarization_score_from_judge_output,
    threshold_semantic_similarity, tool_call_accuracy, tool_call_f1, topic_adherence,
};
pub use optimizers::{
    CandidateGenerator, DspyCacheContract, GeneticOptimizer, GeneticOptimizerConfig, MiproV2Trial,
    ObjectiveMetric, OptimizationCandidate, OptimizationResult, OptimizationStep, Optimizer,
    OptimizerFamily, OptimizerRuntime, dspy_cache_contract, plan_mipro_v2_trials,
};
pub use prompts::{
    FewShotExample, JudgeOutputParser, LanguageAdapterRule, MultimodalPromptMessage,
    MultimodalPromptPart, OutputParseDiagnostic, ParsedJudgeOutput, PromptTemplate, PromptValue,
    PromptValueKind, PromptVariables, RenderedPrompt, RepairStrategy,
};
pub use providers::{
    MockEmbeddingProvider, MockLlmProvider, ProviderAuthScheme, ProviderFamily, ProviderKind,
    ProviderProtocolDescriptor, ProviderProtocolInput, ProviderProtocolMode, ProviderRegistry,
    ProviderRequestPlan, plan_provider_request, provider_protocol_descriptors,
    record_provider_usage,
};
pub use resilience::{
    CachingEmbeddingProvider, CachingLlmProvider, ResilientEmbeddingProvider, ResilientLlmProvider,
    UsageRecordingLlmProvider,
};
pub use runtime::{
    AsyncExecutor, CacheKey, CallbackManager, CancellationConfig, ExecutorJobResult,
    ExecutorOutcome, ExecutorReport, LazyTokenizer, ModelTokenUsage, ProgressEvent,
    ProgressEventKind, RetryConfig, RunConfig, RunConfigBuilder, RunConfigError, RuntimeCacheKey,
    RuntimeEvent, RuntimeEventKind, TimeoutConfig, UsageSummary, UsageTotals, UsageTracker,
    generate_runtime_cache_key, total_model_cost,
};
#[cfg(feature = "tokenizer")]
pub use runtime::{num_tokens_from_string, tiktoken_encoding_for_model};
pub use schema::{Message, MessageRole, MultiTurnSample, Rubric, ToolCall};
pub use testset::{
    CustomNodeFilter, EmbeddingExtractor, ExtractionBundle, GraphAdvancedQuery, GraphCluster,
    GraphEdge, GraphNode, GraphParityFixture, GraphProperty, GraphTransform, KnowledgeGraph,
    LlmExtractor, LlmExtractorKind, MultiHopScenario, MultiHopSpecificSynthesizer, Persona,
    PersonaGenerator, QueryLength, QueryStyle, RenderedSynthesizerPromptMessage,
    RenderedSynthesizerPromptSnapshot, SingleHopScenario, SingleHopSpecificSynthesizer,
    SynthesizedSample, Synthesizer, SynthesizerPromptMessage, SynthesizerPromptSnapshot,
    SynthesizerSampleComparison, SynthesizerStrategy, TextChunk, apply_transforms,
    attach_extractions, build_chunk_relationships, build_cosine_relationships,
    build_overlap_relationships, cluster_graph_by_property, compare_synthesized_sample_fixture,
    extract_bundle, filter_graph_by_property, find_n_indirect_clusters, generate_personas_from_kg,
    generate_testset, match_themes_to_personas, normalize_extraction_properties,
    parse_graph_parity_fixture, parse_llm_extractor_output, prepare_multi_hop_specific_scenarios,
    prepare_single_hop_scenarios, query_graph_advanced, render_synthesizer_prompt_snapshot,
    serialize_graph_parity_fixture, split_text_into_chunks, synthesize_multi_hop_sample,
    synthesize_pre_chunked_samples, synthesize_single_hop_sample,
};
pub use validation::{
    MetricRequirements, SampleField, ValidationIssue, ValidationReport, validate_before_evaluate,
    validate_dataset_requirements, validate_single_turn_samples,
};
