use std::collections::{BTreeMap, BTreeSet};

use super::{MetricMetadata, MetricProviderRequirement, MetricSampleKind, MetricValueType};
use crate::{ParityClaim, ParityFeatureStatus, RagasError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MetricCatalogFamily {
    ContextPrecision,
    ContextRecall,
    ContextEntityRecall,
    ContextRelevance,
    Faithfulness,
    ResponseRelevancy,
    ResponseGroundedness,
    FactualCorrectness,
    AnswerRelevancy,
    AnswerCorrectness,
    NoiseSensitivity,
    ExactMatch,
    Bleu,
    RougeL,
    Chrf,
    SemanticSimilarity,
    StringSimilarity,
    Rubrics,
    AspectCritic,
    ToolCallAccuracy,
    ToolCallF1,
    AgentGoalAccuracy,
    TopicAdherence,
    SqlSemanticEquivalence,
    Multimodal,
    Summarization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricFixtureCoverage {
    FixtureBacked,
    Missing,
    NotRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCatalogDescriptor {
    pub family: MetricCatalogFamily,
    pub upstream_name: &'static str,
    pub rust_owner: &'static str,
    pub sample_kind: MetricSampleKind,
    pub provider_requirements: Vec<MetricProviderRequirement>,
    pub output_type: MetricValueType,
    pub fixture_coverage: MetricFixtureCoverage,
    pub parity_status: ParityFeatureStatus,
}

impl MetricCatalogDescriptor {
    pub fn new(
        family: MetricCatalogFamily,
        upstream_name: &'static str,
        rust_owner: &'static str,
        sample_kind: MetricSampleKind,
        provider_requirements: Vec<MetricProviderRequirement>,
        output_type: MetricValueType,
        fixture_coverage: MetricFixtureCoverage,
        parity_status: ParityFeatureStatus,
    ) -> Self {
        Self {
            family,
            upstream_name,
            rust_owner,
            sample_kind,
            provider_requirements,
            output_type,
            fixture_coverage,
            parity_status,
        }
    }

    pub fn parity_feature(&self) -> String {
        format!("metric::{}", self.upstream_name)
    }
}

pub fn metric_catalog() -> Vec<MetricCatalogDescriptor> {
    use MetricCatalogFamily::*;
    use MetricFixtureCoverage::*;
    use MetricProviderRequirement::*;
    use MetricSampleKind::*;
    use MetricValueType::*;
    use ParityFeatureStatus::*;

    vec![
        MetricCatalogDescriptor::new(
            ContextPrecision,
            "context_precision",
            "context_precision_from_relevance",
            SingleTurn,
            vec![Llm],
            Numeric,
            FixtureBacked,
            Complete,
        ),
        MetricCatalogDescriptor::new(
            ContextRecall,
            "context_recall",
            "context_recall",
            SingleTurn,
            vec![Llm],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            ContextEntityRecall,
            "context_entity_recall",
            "context_entity_recall",
            SingleTurn,
            Vec::new(),
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            ContextRelevance,
            "context_relevance",
            "context_relevance",
            SingleTurn,
            vec![Llm],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            Faithfulness,
            "faithfulness",
            "FaithfulnessMetric",
            SingleTurn,
            vec![Llm],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            ResponseRelevancy,
            "response_relevancy",
            "ResponseRelevancyMetric",
            SingleTurn,
            vec![Embedding],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            ResponseGroundedness,
            "response_groundedness",
            "response_groundedness",
            SingleTurn,
            vec![Llm],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            FactualCorrectness,
            "factual_correctness",
            "factual_correctness",
            SingleTurn,
            vec![Llm],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            AnswerRelevancy,
            "answer_relevancy",
            "answer_relevancy_from_embedding_similarity",
            SingleTurn,
            vec![Embedding],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            AnswerCorrectness,
            "answer_correctness",
            "answer_correctness",
            SingleTurn,
            vec![Llm, Embedding],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            NoiseSensitivity,
            "noise_sensitivity",
            "noise_sensitivity",
            SingleTurn,
            vec![Llm],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            ExactMatch,
            "exact_match",
            "exact_match",
            SingleTurn,
            Vec::new(),
            Discrete,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            Bleu,
            "bleu",
            "bleu_unigram",
            SingleTurn,
            Vec::new(),
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            RougeL,
            "rouge_l",
            "rouge_l_recall",
            SingleTurn,
            Vec::new(),
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            Chrf,
            "chrf",
            "chrf_score",
            SingleTurn,
            Vec::new(),
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            SemanticSimilarity,
            "semantic_similarity",
            "semantic_similarity_from_vectors",
            SingleTurn,
            vec![Embedding],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            StringSimilarity,
            "string_similarity",
            "string_distance_similarity",
            SingleTurn,
            Vec::new(),
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            Rubrics,
            "rubrics",
            "RubricMetric",
            SingleTurn,
            vec![Llm],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            AspectCritic,
            "aspect_critic",
            "score_aspect_critic",
            SingleTurn,
            vec![Llm],
            Discrete,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            ToolCallAccuracy,
            "tool_call_accuracy",
            "tool_call_accuracy",
            MultiTurn,
            Vec::new(),
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            ToolCallF1,
            "tool_call_f1",
            "tool_call_f1",
            MultiTurn,
            Vec::new(),
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            AgentGoalAccuracy,
            "agent_goal_accuracy",
            "agent_goal_accuracy",
            MultiTurn,
            vec![Llm],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            TopicAdherence,
            "topic_adherence",
            "topic_adherence",
            MultiTurn,
            vec![Llm],
            Numeric,
            Missing,
            Partial,
        ),
        MetricCatalogDescriptor::new(
            SqlSemanticEquivalence,
            "sql_semantic_equivalence",
            "sql_semantic_equivalence",
            SingleTurn,
            vec![Llm],
            Discrete,
            Missing,
            KnownGap,
        ),
        MetricCatalogDescriptor::new(
            Multimodal,
            "multimodal",
            "multimodal_metric_from_prompt",
            SingleTurn,
            vec![Llm],
            Numeric,
            Missing,
            KnownGap,
        ),
        MetricCatalogDescriptor::new(
            Summarization,
            "summarization",
            "summarization_score_from_judge_output",
            SingleTurn,
            vec![Llm],
            Numeric,
            Missing,
            KnownGap,
        ),
    ]
}

pub fn metric_catalog_parity_claims() -> Vec<ParityClaim> {
    metric_catalog()
        .into_iter()
        .filter(|descriptor| {
            !(descriptor.parity_status == ParityFeatureStatus::Complete
                && descriptor.fixture_coverage == MetricFixtureCoverage::FixtureBacked)
        })
        .map(|descriptor| ParityClaim {
            feature: descriptor.parity_feature(),
            status: descriptor.parity_status,
            fixtures: Vec::new(),
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityStatus {
    ParityComplete,
    SemanticApproximation,
    KnownGap,
    NotStarted,
}

impl ParityStatus {
    pub fn as_label(&self) -> &'static str {
        match self {
            ParityStatus::ParityComplete => "parity-complete",
            ParityStatus::SemanticApproximation => "semantic-approximation",
            ParityStatus::KnownGap => "known-gap",
            ParityStatus::NotStarted => "not-started",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRegistryEntry {
    name: String,
    metadata: MetricMetadata,
    feature: Option<String>,
    parity_status: ParityStatus,
}

impl MetricRegistryEntry {
    pub fn new(name: impl Into<String>, metadata: MetricMetadata) -> Self {
        Self {
            name: name.into(),
            metadata,
            feature: None,
            parity_status: ParityStatus::NotStarted,
        }
    }

    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.feature = Some(feature.into());
        self
    }

    pub fn with_parity_status(mut self, status: ParityStatus) -> Self {
        self.parity_status = status;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn feature(&self) -> Option<&str> {
        self.feature.as_deref()
    }

    pub fn parity_status(&self) -> ParityStatus {
        self.parity_status
    }

    pub fn parity_label(&self) -> &'static str {
        self.parity_status.as_label()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MetricRegistry {
    entries: BTreeMap<String, MetricRegistryEntry>,
    enabled_features: BTreeSet<String>,
}

impl MetricRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(mut self, entry: MetricRegistryEntry) -> Self {
        self.entries.insert(entry.name.clone(), entry);
        self
    }

    pub fn enable_feature(mut self, feature: impl Into<String>) -> Self {
        self.enabled_features.insert(feature.into());
        self
    }

    pub fn resolve(&self, name: &str) -> Result<&MetricRegistryEntry, RagasError> {
        let entry = self.entries.get(name).ok_or_else(|| RagasError::Provider {
            message: format!("missing metric: {name}"),
        })?;
        if self.is_visible(entry) {
            Ok(entry)
        } else {
            Err(RagasError::Provider {
                message: format!(
                    "metric '{name}' requires feature '{}'",
                    entry.feature.as_deref().unwrap_or("")
                ),
            })
        }
    }

    pub fn list_visible_names(&self) -> Vec<String> {
        self.entries
            .values()
            .filter(|entry| self.is_visible(entry))
            .map(|entry| entry.name.clone())
            .collect()
    }

    fn is_visible(&self, entry: &MetricRegistryEntry) -> bool {
        entry
            .feature
            .as_ref()
            .is_none_or(|feature| self.enabled_features.contains(feature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::release_blocking_claims;

    fn entry(name: &str) -> MetricRegistryEntry {
        MetricRegistryEntry::new(
            name,
            MetricMetadata::new(name, MetricSampleKind::SingleTurn)
                .with_provider_requirements(vec![MetricProviderRequirement::Llm]),
        )
    }

    #[test]
    fn test_9_3_1_metric_registry_resolves_builtins_by_stable_names() {
        // SCEN-9.3.1 / AC1 / TEST-9.3.1
        let registry = MetricRegistry::new()
            .register(entry("faithfulness"))
            .register(entry("response_relevancy"))
            .register(entry("context_precision"));

        assert_eq!(
            registry.resolve("faithfulness").expect("entry").name(),
            "faithfulness"
        );
        assert_eq!(
            registry
                .resolve("response_relevancy")
                .expect("entry")
                .name(),
            "response_relevancy"
        );
        assert!(registry.resolve("missing_metric").is_err());
    }

    #[test]
    fn test_9_3_2_feature_gated_metrics_are_hidden_unless_enabled() {
        // SCEN-9.3.2 / AC2 / TEST-9.3.2
        let registry = MetricRegistry::new()
            .register(entry("faithfulness"))
            .register(entry("sql_semantic_equivalence").with_feature("sql"));

        assert_eq!(
            registry.list_visible_names(),
            vec!["faithfulness".to_string()]
        );
        assert!(registry.resolve("sql_semantic_equivalence").is_err());

        let enabled = registry.enable_feature("sql");
        assert_eq!(
            enabled.list_visible_names(),
            vec![
                "faithfulness".to_string(),
                "sql_semantic_equivalence".to_string()
            ]
        );
        assert_eq!(
            enabled
                .resolve("sql_semantic_equivalence")
                .expect("enabled metric")
                .feature(),
            Some("sql")
        );
    }

    #[test]
    fn test_9_3_3_parity_status_labels_are_exported_for_docs_and_tests() {
        // SCEN-9.3.3 / AC3 / TEST-9.3.3
        assert_eq!(ParityStatus::ParityComplete.as_label(), "parity-complete");
        assert_eq!(
            ParityStatus::SemanticApproximation.as_label(),
            "semantic-approximation"
        );
        assert_eq!(ParityStatus::KnownGap.as_label(), "known-gap");
        assert_eq!(ParityStatus::NotStarted.as_label(), "not-started");

        let registered = entry("faithfulness").with_parity_status(ParityStatus::KnownGap);
        assert_eq!(registered.parity_label(), "known-gap");
    }

    #[test]
    fn test_19_1_1_metric_catalog_descriptors_list_upstream_families_and_owners() {
        // SCEN-19.1.1 / AC1 / TEST-19.1.1
        let catalog = metric_catalog();
        let families: BTreeSet<_> = catalog.iter().map(|descriptor| descriptor.family).collect();

        for expected in [
            MetricCatalogFamily::ContextPrecision,
            MetricCatalogFamily::ContextRecall,
            MetricCatalogFamily::Faithfulness,
            MetricCatalogFamily::ResponseRelevancy,
            MetricCatalogFamily::FactualCorrectness,
            MetricCatalogFamily::AnswerCorrectness,
            MetricCatalogFamily::SemanticSimilarity,
            MetricCatalogFamily::AspectCritic,
            MetricCatalogFamily::ToolCallAccuracy,
            MetricCatalogFamily::TopicAdherence,
            MetricCatalogFamily::SqlSemanticEquivalence,
            MetricCatalogFamily::Multimodal,
            MetricCatalogFamily::Summarization,
        ] {
            assert!(families.contains(&expected), "missing {expected:?}");
        }

        assert!(catalog.iter().all(|descriptor| {
            !descriptor.upstream_name.is_empty() && !descriptor.rust_owner.is_empty()
        }));
    }

    #[test]
    fn test_19_1_2_metric_catalog_records_scoring_contract_metadata() {
        // SCEN-19.1.2 / AC2 / TEST-19.1.2
        let catalog = metric_catalog();
        let faithfulness = catalog
            .iter()
            .find(|descriptor| descriptor.family == MetricCatalogFamily::Faithfulness)
            .expect("faithfulness descriptor");

        assert_eq!(faithfulness.sample_kind, MetricSampleKind::SingleTurn);
        assert!(
            faithfulness
                .provider_requirements
                .contains(&MetricProviderRequirement::Llm)
        );
        assert_eq!(faithfulness.output_type, MetricValueType::Numeric);
        assert_ne!(
            faithfulness.fixture_coverage,
            MetricFixtureCoverage::NotRequired
        );

        let semantic = catalog
            .iter()
            .find(|descriptor| descriptor.family == MetricCatalogFamily::SemanticSimilarity)
            .expect("semantic descriptor");
        assert!(
            semantic
                .provider_requirements
                .contains(&MetricProviderRequirement::Embedding)
        );
    }

    #[test]
    fn test_19_1_3_metrics_without_complete_fixture_parity_block_release() {
        // SCEN-19.1.3 / AC3 / TEST-19.1.3
        let claims = metric_catalog_parity_claims();
        let blockers = release_blocking_claims(&claims);
        let blocking_features: BTreeSet<_> = blockers
            .iter()
            .map(|claim| claim.feature.as_str())
            .collect();

        for expected in [
            "metric::summarization",
            "metric::multimodal",
            "metric::sql_semantic_equivalence",
        ] {
            assert!(
                blocking_features.contains(expected),
                "missing metric release blocker {expected}"
            );
        }

        assert!(claims.iter().all(|claim| {
            !(blocking_features.contains(claim.feature.as_str())
                && claim.status == ParityFeatureStatus::Complete)
        }));
    }
}
