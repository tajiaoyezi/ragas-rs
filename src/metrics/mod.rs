pub mod base;
pub mod rag;
pub mod registry;
pub mod result;

pub use base::{
    MetricMetadata, MetricProviderRequirement, MetricSampleKind, MultiTurnMetric,
    SingleTurnMetric,
};
pub use rag::{
    AnswerCorrectnessWeights, ContextPrecisionVariant, FaithfulnessJudgeContract,
    FactualCorrectnessCounts, answer_correctness, answer_relevancy_from_embedding_similarity,
    answer_relevancy_from_judge_output, context_entity_recall, context_precision_from_relevance,
    context_recall, context_relevance, factual_correctness, id_based_context_precision,
    noise_sensitivity, response_groundedness,
};
pub use result::{
    DetailedMetricResult, MetricError, MetricErrorKind, MetricEvidence, MetricValueType,
    ScoreNormalizationPolicy, normalize_score,
};
pub use registry::{MetricRegistry, MetricRegistryEntry, ParityStatus};
