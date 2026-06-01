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
    BackendCapability, BackendDescriptor, BackendFamily, BackendMode, BackendRegistry,
    CsvDatasetBackend, DatasetBackend, DiskCacheCompatibility, InMemoryDatasetBackend,
    JsonlDatasetBackend, backend_descriptors, backend_parity_claims,
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
    IntegrationDescriptor, IntegrationDestination, IntegrationEvent, IntegrationFamily,
    IntegrationFeatureRegistry, IntegrationPayload, IntegrationRegistry, IntegrationTestMode,
    TracingIntegration, integration_descriptors, integration_parity_claims,
    normalize_callback_payload, redact_payload,
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
    FaithfulnessJudgeContract, InstanceRubric, MetricCatalogDescriptor, MetricCatalogFamily,
    MetricError, MetricErrorKind, MetricEvidence, MetricFixtureCoverage, MetricMetadata,
    MetricProviderRequirement, MetricRegistry, MetricRegistryEntry, MetricSampleKind,
    MetricValueType, MultiTurnMetric, MultimodalMetricKind, ParityStatus, QuotedSpan,
    RubricCriterion, RubricMetric, ScoreNormalizationPolicy, SemanticThresholdPolicy,
    SingleTurnMetric, SqlJudgeVerdict, SummarizationSignals, ToolCallOrderPolicy, TopicAdherence,
    agent_goal_accuracy, answer_correctness, answer_relevancy_from_embedding_similarity,
    answer_relevancy_from_judge_output, bleu_unigram, chrf_score, context_entity_recall,
    context_precision_from_relevance, context_recall, context_relevance, exact_match,
    extract_quoted_spans, factual_correctness, id_based_context_precision,
    lexical_tokenizer_assumptions, metric_catalog, metric_catalog_parity_claims,
    multimodal_metric_from_prompt, noise_sensitivity, normalize_score, quoted_citation_coverage,
    quoted_span_overlap, response_groundedness, rouge_l_recall, score_aspect_critic,
    semantic_similarity_batch, semantic_similarity_from_vectors, sql_semantic_equivalence,
    string_distance_similarity, summarization_score_from_judge_output,
    threshold_semantic_similarity, tool_call_accuracy, tool_call_f1, topic_adherence,
};
pub use optimizers::{
    CandidateGenerator, GeneticOptimizer, GeneticOptimizerConfig, ObjectiveMetric,
    OptimizationCandidate, OptimizationResult, OptimizationStep, Optimizer,
};
pub use parity::{
    GapMatrixEntry, ParityCheck, ParityClaim, ParityFeatureStatus, ParityFixture,
    ParityFixtureMetadata, ParityFixtureMode, UpstreamBaseline, UpstreamInventoryEntry,
    check_parity_fixture, latest_upstream_baseline, latest_upstream_inventory,
    parse_parity_fixture, release_blocking_claims, release_blocking_inventory, validate_gap_matrix,
    validate_parity_claim,
};
pub use prompts::{
    FewShotExample, JudgeOutputParser, LanguageAdapterRule, MultimodalPromptMessage,
    MultimodalPromptPart, OutputParseDiagnostic, ParsedJudgeOutput, PromptTemplate, PromptValue,
    PromptValueKind, PromptVariables, RenderedPrompt, RepairStrategy,
};
pub use providers::{
    MockEmbeddingProvider, MockLlmProvider, ProviderDescriptor, ProviderFamily, ProviderKind,
    ProviderMode, ProviderRegistry, StructuredLlmDescriptor, provider_parity_claims,
    record_provider_usage, structured_llm_descriptors, upstream_provider_descriptors,
};
pub use release::{
    BugClass, BugLedgerEntry, BugSeverity, BugStatus, BugZeroAudit, GateEvidenceStatus,
    QualityGateEvidence, QualityGateKind, QualityGateSummary, ReleaseGateReport,
    quality_gate_blockers, release_blocking_bugs, release_gate_files, required_quality_gates,
    summarize_bug_zero_audit, summarize_quality_gates,
};
pub use runtime::{
    AsyncExecutor, CacheKey, CallbackManager, CancellationConfig, ExecutorJobResult,
    ExecutorOutcome, ExecutorReport, LazyTokenizer, ModelTokenUsage, ProgressEvent,
    ProgressEventKind, RetryConfig, RunConfig, RunConfigBuilder, RunConfigError, RuntimeCacheKey,
    RuntimeEvent, RuntimeEventKind, TimeoutConfig, UsageSummary, UsageTotals, UsageTracker,
    generate_runtime_cache_key, total_model_cost,
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
