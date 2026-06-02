use std::collections::{BTreeMap, BTreeSet};

use super::{MetricMetadata, MetricProviderRequirement, MetricSampleKind, MetricValueType};
use crate::{
    ParityClaim, ParityFeatureStatus, ParityFixtureMetadata, ParityFixtureMode, RagasError,
};

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

    let mut catalog = vec![
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
    ];

    let fixture_backed = metric_golden_fixture_metadata()
        .into_iter()
        .map(|fixture| fixture.feature)
        .collect::<BTreeSet<_>>();
    for descriptor in &mut catalog {
        if fixture_backed.contains(descriptor.upstream_name) {
            descriptor.fixture_coverage = FixtureBacked;
            descriptor.parity_status = Complete;
        }
    }

    catalog
}

pub fn metric_catalog_parity_claims() -> Vec<ParityClaim> {
    let mut fixtures_by_metric: BTreeMap<String, Vec<ParityFixtureMetadata>> = BTreeMap::new();
    for fixture in metric_golden_fixture_metadata() {
        fixtures_by_metric
            .entry(fixture.feature.clone())
            .or_default()
            .push(fixture);
    }

    metric_catalog()
        .into_iter()
        .map(|descriptor| ParityClaim {
            feature: descriptor.parity_feature(),
            status: descriptor.parity_status,
            fixtures: fixtures_by_metric
                .remove(descriptor.upstream_name)
                .unwrap_or_default(),
        })
        .collect()
}

pub fn metric_golden_fixture_metadata() -> Vec<ParityFixtureMetadata> {
    vec![
        metric_fixture(
            "context_precision",
            "src/ragas/metrics/collections/context_precision/metric.py",
            "metric_context_precision.json",
        ),
        metric_fixture(
            "context_recall",
            "src/ragas/metrics/collections/context_recall/metric.py",
            "metric_context_recall.json",
        ),
        metric_fixture(
            "context_entity_recall",
            "src/ragas/metrics/collections/context_entity_recall/metric.py",
            "metric_context_entity_recall.json",
        ),
        metric_fixture(
            "context_relevance",
            "src/ragas/metrics/collections/context_relevance/metric.py",
            "metric_context_relevance.json",
        ),
        metric_fixture(
            "faithfulness",
            "src/ragas/metrics/collections/faithfulness/metric.py",
            "metric_faithfulness.json",
        ),
        metric_fixture(
            "response_relevancy",
            "src/ragas/metrics/_answer_relevance.py",
            "metric_response_relevancy.json",
        ),
        metric_fixture(
            "response_groundedness",
            "src/ragas/metrics/collections/response_groundedness/metric.py",
            "metric_response_groundedness.json",
        ),
        metric_fixture(
            "factual_correctness",
            "src/ragas/metrics/collections/factual_correctness/metric.py",
            "metric_factual_correctness.json",
        ),
        metric_fixture(
            "answer_relevancy",
            "src/ragas/metrics/collections/answer_relevancy/metric.py",
            "metric_answer_relevancy.json",
        ),
        metric_fixture(
            "answer_correctness",
            "src/ragas/metrics/collections/answer_correctness/metric.py",
            "metric_answer_correctness.json",
        ),
        metric_fixture(
            "noise_sensitivity",
            "src/ragas/metrics/collections/noise_sensitivity/metric.py",
            "metric_noise_sensitivity.json",
        ),
        metric_fixture(
            "exact_match",
            "src/ragas/metrics/_string.py",
            "metric_exact_match.json",
        ),
        metric_fixture(
            "bleu",
            "src/ragas/metrics/_bleu_score.py",
            "metric_bleu.json",
        ),
        metric_fixture(
            "rouge_l",
            "src/ragas/metrics/_rouge_score.py",
            "metric_rouge_l.json",
        ),
        metric_fixture(
            "chrf",
            "src/ragas/metrics/_chrf_score.py",
            "metric_chrf.json",
        ),
        metric_fixture(
            "semantic_similarity",
            "src/ragas/metrics/collections/_semantic_similarity.py",
            "metric_semantic_similarity.json",
        ),
        metric_fixture(
            "string_similarity",
            "src/ragas/metrics/collections/_string.py",
            "metric_string_similarity.json",
        ),
        metric_fixture(
            "rubrics",
            "src/ragas/metrics/collections/domain_specific_rubrics/metric.py",
            "metric_rubrics.json",
        ),
        metric_fixture(
            "aspect_critic",
            "src/ragas/metrics/_aspect_critic.py",
            "metric_aspect_critic.json",
        ),
        metric_fixture(
            "tool_call_accuracy",
            "src/ragas/metrics/collections/tool_call_accuracy/metric.py",
            "metric_tool_call_accuracy.json",
        ),
        metric_fixture(
            "tool_call_f1",
            "src/ragas/metrics/collections/tool_call_f1/metric.py",
            "metric_tool_call_f1.json",
        ),
        metric_fixture(
            "agent_goal_accuracy",
            "src/ragas/metrics/collections/agent_goal_accuracy/metric.py",
            "metric_agent_goal_accuracy.json",
        ),
        metric_fixture(
            "topic_adherence",
            "src/ragas/metrics/collections/topic_adherence/metric.py",
            "metric_topic_adherence.json",
        ),
        metric_fixture(
            "sql_semantic_equivalence",
            "src/ragas/metrics/collections/sql_semantic_equivalence/metric.py",
            "metric_sql_semantic_equivalence.json",
        ),
        metric_fixture(
            "multimodal",
            "src/ragas/metrics/collections/multi_modal_faithfulness/metric.py",
            "metric_multimodal.json",
        ),
        metric_fixture(
            "summarization",
            "src/ragas/metrics/collections/summary_score/metric.py",
            "metric_summarization.json",
        ),
    ]
}

fn metric_fixture(
    feature: &'static str,
    upstream_module_path: &'static str,
    fixture_filename: &'static str,
) -> ParityFixtureMetadata {
    ParityFixtureMetadata::new(
        feature,
        upstream_module_path,
        None,
        format!("tests/parity/fixtures/{fixture_filename}"),
        ParityFixtureMode::DeterministicMock,
        Some(1e-9),
    )
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
    use crate::parity::{
        MetricGoldenOutcome, compare_metric_golden_fixture, parse_metric_golden_fixture,
        validate_metric_golden_claim,
    };
    use crate::release::{
        ReleaseBlockerCategory, build_release_blocker_ledger, summarize_release_blocker_ledger,
    };
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
        let claims = vec![
            ParityClaim {
                feature: "metric::synthetic_without_fixture".to_string(),
                status: ParityFeatureStatus::Complete,
                fixtures: Vec::new(),
            },
            ParityClaim {
                feature: "metric::synthetic_partial".to_string(),
                status: ParityFeatureStatus::Partial,
                fixtures: Vec::new(),
            },
        ];
        let blockers = release_blocking_claims(&claims);
        let blocking_features: BTreeSet<_> = blockers
            .iter()
            .map(|claim| claim.feature.as_str())
            .collect();

        for expected in [
            "metric::synthetic_without_fixture",
            "metric::synthetic_partial",
        ] {
            assert!(
                blocking_features.contains(expected),
                "missing metric release blocker {expected}"
            );
        }

        assert_eq!(blockers.len(), 2);
        assert!(blockers.iter().any(|claim| {
            claim.feature == "metric::synthetic_without_fixture"
                && claim.status == ParityFeatureStatus::Complete
                && claim.fixtures.is_empty()
        }));
    }

    #[test]
    fn test_28_1_1_metric_catalog_descriptors_are_complete_and_fixture_backed() {
        // SCEN-28.1.1 / AC1 / TEST-28.1.1
        let catalog = metric_catalog();
        let metadata = metric_golden_fixture_metadata();
        let fixture_features = metadata
            .iter()
            .map(|fixture| fixture.feature.as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(
            metadata.len(),
            catalog.len(),
            "each metric catalog descriptor must have one golden fixture metadata entry"
        );

        for descriptor in &catalog {
            assert_eq!(
                descriptor.parity_status,
                ParityFeatureStatus::Complete,
                "{} must be marked complete only after fixture evidence exists",
                descriptor.upstream_name
            );
            assert_eq!(
                descriptor.fixture_coverage,
                MetricFixtureCoverage::FixtureBacked,
                "{} must be fixture-backed",
                descriptor.upstream_name
            );
            assert!(
                fixture_features.contains(descriptor.upstream_name),
                "{} missing golden fixture metadata",
                descriptor.upstream_name
            );
        }

        assert!(metadata.iter().all(|fixture| {
            fixture
                .upstream_module_path
                .starts_with("src/ragas/metrics/")
                && fixture
                    .fixture_path
                    .starts_with("tests/parity/fixtures/metric_")
        }));
    }

    #[test]
    fn test_28_1_2_metric_golden_fixtures_parse_and_compare_for_all_tracked_metrics() {
        // SCEN-28.1.2 / AC2 / TEST-28.1.2
        let metadata = metric_golden_fixture_metadata();
        assert_eq!(metadata.len(), metric_catalog().len());

        let mut checked = BTreeSet::new();
        for fixture_metadata in metadata {
            let fixture_path = fixture_metadata.fixture_path.clone();
            let raw = std::fs::read_to_string(&fixture_path).expect("metric fixture file");
            let golden =
                parse_metric_golden_fixture(&raw, fixture_metadata).expect("metric fixture parses");
            let comparison = compare_metric_golden_fixture(&golden);
            assert!(
                matches!(
                    comparison.outcome,
                    MetricGoldenOutcome::ExactMatch | MetricGoldenOutcome::ToleratedNumericDrift
                ),
                "{} has undeclared drift: {:?}",
                comparison.feature,
                comparison.drift
            );
            validate_metric_golden_claim(&ParityClaim {
                feature: format!("metric::{}", golden.metadata.feature),
                status: ParityFeatureStatus::Complete,
                fixtures: vec![golden.metadata.clone()],
            })
            .expect("complete metric claim validates");
            checked.insert(golden.metadata.feature);
        }

        assert_eq!(checked.len(), metric_catalog().len());
    }

    #[test]
    fn test_28_1_3_metric_release_ledger_has_no_metric_blockers_after_fixture_closure() {
        // SCEN-28.1.3 / AC3 / TEST-28.1.3
        let ledger = build_release_blocker_ledger();
        let summary = summarize_release_blocker_ledger(&ledger);
        let categories = ledger
            .entries
            .iter()
            .map(|entry| entry.category)
            .collect::<BTreeSet<_>>();

        assert!(
            !categories.contains(&ReleaseBlockerCategory::Metric),
            "metric release blockers must be fully closed"
        );
        assert!(
            !summary
                .by_category
                .contains_key(&ReleaseBlockerCategory::Metric)
        );

        assert!(
            !categories.contains(&ReleaseBlockerCategory::Testset),
            "closed testset claims should not keep Testset in blocker ledger"
        );

        assert!(
            !categories.contains(&ReleaseBlockerCategory::Optimizer),
            "closed optimizer claims should not keep Optimizer in blocker ledger"
        );
        assert!(
            !categories.contains(&ReleaseBlockerCategory::Quality),
            "closed quality evidence should not keep Quality in blocker ledger"
        );
        assert!(summary.release_ready);
    }
}
