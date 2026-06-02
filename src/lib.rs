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
pub use cli::{
    CliCommand, CliContractSnapshot, CliErrorSnapshot, CliOutput, CliRuntime, WorkflowDescriptor,
    WorkflowFamily, WorkflowSurface, cli_contract_snapshot, cli_error_snapshot, run_cli_command,
    workflow_descriptors, workflow_parity_claims,
};
pub use dataset::{
    EvaluationDataset, EvaluationDatasetBuilder, EvaluationSample, SingleTurnSample,
};
pub use docs_examples::{
    DocExample, ExampleOutputType, QuickstartDescriptor, RunnableExampleMetadata,
    docs_parity_claims, public_workflow_examples, quickstart_descriptors,
    runnable_example_metadata,
};
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
    CandidateGenerator, DspyCacheContract, GeneticOptimizer, GeneticOptimizerConfig,
    ObjectiveMetric, OptimizationCandidate, OptimizationResult, OptimizationStep, Optimizer,
    OptimizerFamily, OptimizerFamilyDescriptor, OptimizerRuntime, dspy_cache_contract,
    optimizer_family_descriptors, optimizer_parity_claims,
};
pub use parity::{
    GapMatrixEntry, MetricGoldenComparison, MetricGoldenFixture, MetricGoldenOutcome, ParityCheck,
    ParityClaim, ParityFeatureStatus, ParityFixture, ParityFixtureMetadata, ParityFixtureMode,
    UpstreamBaseline, UpstreamInventoryEntry, check_parity_fixture, compare_metric_golden_fixture,
    latest_upstream_baseline, latest_upstream_inventory, parse_metric_golden_fixture,
    parse_parity_fixture, release_blocking_claims, release_blocking_inventory, validate_gap_matrix,
    validate_metric_golden_claim, validate_parity_claim,
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
    MetricReleaseBlocker, MetricReleaseBlockerSource, MetricReleaseBlockerSummary,
    E2eWorkflow, E2eWorkflowDescriptor, MutationGateDescriptor, PanicSafetyGateDescriptor,
    PlatformEvidenceDescriptor, PlatformTarget, QualityCommandEvidence, QualityEvidenceFinding,
    QualityEvidenceKind, QualityGateDescriptor, QualityGateEvidence, QualityGateKind,
    QualityGateMode, QualityGateSummary, ReleaseBlockerCategory, ReleaseBlockerEntry,
    ReleaseBlockerLedger, ReleaseBlockerSummary, ReleaseGateReport, SafetyFailureClass,
    build_release_blocker_ledger, e2e_workflow_matrix, metric_release_blockers,
    mutation_gate_descriptors, panic_mutation_quality_gate_descriptors,
    panic_safety_gate_descriptors, platform_e2e_quality_gate_descriptors, platform_evidence_matrix,
    property_fuzz_coverage_gate_descriptors, quality_gate_blockers, release_blocking_bugs,
    release_gate_files, required_quality_evidence_blockers, required_quality_gates,
    summarize_bug_zero_audit, summarize_metric_release_blockers, summarize_quality_gates,
    summarize_release_blocker_ledger,
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
    ExtractionBundle, GraphEdge, GraphNode, GraphParityFixture, GraphProperty,
    GraphQueryCapability, GraphQueryDescriptor, KnowledgeGraph, Persona, PersonaGenerator,
    RenderedSynthesizerPromptMessage, RenderedSynthesizerPromptSnapshot, SynthesizedSample,
    SynthesizerDescriptor, SynthesizerPromptMessage, SynthesizerPromptSnapshot,
    SynthesizerSampleComparison, SynthesizerStrategy, TextChunk, TransformStageDescriptor,
    TransformStageFamily, TransformStageMode, attach_extractions, build_chunk_relationships,
    compare_synthesized_sample_fixture, graph_parity_claims, graph_query_descriptors,
    normalize_extraction_properties, parse_graph_parity_fixture, render_synthesizer_prompt_snapshot,
    serialize_graph_parity_fixture, split_text_into_chunks, synthesize_multi_hop_sample,
    synthesize_single_hop_sample, synthesizer_descriptors, synthesizer_parity_claims,
    transform_parity_claims, transform_stage_descriptors,
};
pub use validation::{
    MetricRequirements, SampleField, ValidationIssue, ValidationReport, validate_before_evaluate,
    validate_dataset_requirements, validate_single_turn_samples,
};
