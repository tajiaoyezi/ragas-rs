pub mod base;
pub mod rag;
pub mod registry;
pub mod result;

pub use base::{
    MetricMetadata, MetricProviderRequirement, MetricSampleKind, MultiTurnMetric,
    SingleTurnMetric,
};
pub use rag::{
    ContextPrecisionVariant, context_entity_recall, context_precision_from_relevance,
    context_recall, context_relevance, id_based_context_precision,
};
pub use result::{
    DetailedMetricResult, MetricError, MetricErrorKind, MetricEvidence, MetricValueType,
    ScoreNormalizationPolicy, normalize_score,
};
pub use registry::{MetricRegistry, MetricRegistryEntry, ParityStatus};
