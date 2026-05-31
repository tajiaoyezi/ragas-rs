use std::collections::{BTreeMap, BTreeSet};

use super::MetricMetadata;
use crate::RagasError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityStatus {
    ParityComplete,
    SemanticApproximation,
    KnownGap,
    NotStarted,
}

impl ParityStatus {
    pub fn as_label(&self) -> &'static str {
        unimplemented!("task 9.3 RED skeleton")
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
    pub fn new(_name: impl Into<String>, _metadata: MetricMetadata) -> Self {
        unimplemented!("task 9.3 RED skeleton")
    }

    pub fn with_feature(self, _feature: impl Into<String>) -> Self {
        unimplemented!("task 9.3 RED skeleton")
    }

    pub fn with_parity_status(self, _status: ParityStatus) -> Self {
        unimplemented!("task 9.3 RED skeleton")
    }

    pub fn name(&self) -> &str {
        unimplemented!("task 9.3 RED skeleton")
    }

    pub fn feature(&self) -> Option<&str> {
        unimplemented!("task 9.3 RED skeleton")
    }

    pub fn parity_status(&self) -> ParityStatus {
        unimplemented!("task 9.3 RED skeleton")
    }

    pub fn parity_label(&self) -> &'static str {
        unimplemented!("task 9.3 RED skeleton")
    }
}

#[derive(Debug, Clone, Default)]
pub struct MetricRegistry {
    entries: BTreeMap<String, MetricRegistryEntry>,
    enabled_features: BTreeSet<String>,
}

impl MetricRegistry {
    pub fn new() -> Self {
        unimplemented!("task 9.3 RED skeleton")
    }

    pub fn register(self, _entry: MetricRegistryEntry) -> Self {
        unimplemented!("task 9.3 RED skeleton")
    }

    pub fn enable_feature(self, _feature: impl Into<String>) -> Self {
        unimplemented!("task 9.3 RED skeleton")
    }

    pub fn resolve(&self, _name: &str) -> Result<&MetricRegistryEntry, RagasError> {
        unimplemented!("task 9.3 RED skeleton")
    }

    pub fn list_visible_names(&self) -> Vec<String> {
        unimplemented!("task 9.3 RED skeleton")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MetricProviderRequirement, MetricSampleKind};

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

        assert_eq!(registry.resolve("faithfulness").expect("entry").name(), "faithfulness");
        assert_eq!(
            registry.resolve("response_relevancy").expect("entry").name(),
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

        assert_eq!(registry.list_visible_names(), vec!["faithfulness".to_string()]);
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
}
