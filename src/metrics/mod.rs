pub mod base;
pub mod rag;
pub mod registry;
pub mod result;

pub use base::{
    MetricMetadata, MetricProviderRequirement, MetricSampleKind, MultiTurnMetric,
    SingleTurnMetric,
};
pub use rag::{
    ContextPrecisionVariant, FaithfulnessJudgeContract, FactualCorrectnessCounts,
    context_entity_recall, context_precision_from_relevance, context_recall, context_relevance,
    factual_correctness, id_based_context_precision, response_groundedness,
};
pub use result::{
    DetailedMetricResult, MetricError, MetricErrorKind, MetricEvidence, MetricValueType,
    ScoreNormalizationPolicy, normalize_score,
};
pub use registry::{MetricRegistry, MetricRegistryEntry, ParityStatus};
