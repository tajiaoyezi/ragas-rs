use serde::{Deserialize, Serialize};

use crate::{ParityClaim, ParityFeatureStatus, ParityFixtureMetadata, ParityFixtureMode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocExample {
    pub workflow: String,
    pub example_path: String,
    pub upstream_section: String,
    pub feature_flags: Vec<String>,
    pub known_parity_gaps: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExampleOutputType {
    Json,
    Dataset,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickstartDescriptor {
    pub upstream_template: String,
    pub rust_example: Option<String>,
    pub parity_status: ParityFeatureStatus,
    pub known_gap: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnableExampleMetadata {
    pub example_path: String,
    pub command: String,
    pub expected_output: ExampleOutputType,
    pub feature_flags: Vec<String>,
}

pub fn public_workflow_examples() -> Vec<DocExample> {
    vec![
        DocExample {
            workflow: "evaluate".to_string(),
            example_path: "examples/evaluate.rs".to_string(),
            upstream_section: "Evaluate a RAG application".to_string(),
            feature_flags: vec!["default".to_string(), "tokio".to_string()],
            known_parity_gaps: vec!["no Python runtime bridge".to_string()],
        },
        DocExample {
            workflow: "testset".to_string(),
            example_path: "examples/testset.rs".to_string(),
            upstream_section: "Generate a testset".to_string(),
            feature_flags: vec!["default".to_string()],
            known_parity_gaps: vec!["knowledge graph generation".to_string()],
        },
        DocExample {
            workflow: "benchmark".to_string(),
            example_path: "examples/benchmark.rs".to_string(),
            upstream_section: "Compare and monitor evaluation cost".to_string(),
            feature_flags: vec!["default".to_string(), "tokio".to_string()],
            known_parity_gaps: vec!["provider latency percentiles".to_string()],
        },
    ]
}

pub fn quickstart_descriptors() -> Vec<QuickstartDescriptor> {
    vec![
        quickstart_descriptor(
            "Evaluate a RAG application",
            Some("examples/evaluate.rs"),
            ParityFeatureStatus::Complete,
            None,
        ),
        quickstart_descriptor(
            "Generate a testset",
            Some("examples/testset.rs"),
            ParityFeatureStatus::Complete,
            None,
        ),
        quickstart_descriptor(
            "Compare and monitor evaluation cost",
            Some("examples/benchmark.rs"),
            ParityFeatureStatus::Complete,
            None,
        ),
        quickstart_descriptor(
            "Run experiments",
            None,
            ParityFeatureStatus::KnownGap,
            Some("no runnable Rust experiment quickstart example is available yet"),
        ),
    ]
}

pub fn runnable_example_metadata() -> Vec<RunnableExampleMetadata> {
    public_workflow_examples()
        .into_iter()
        .map(|example| RunnableExampleMetadata {
            command: format!("cargo run --example {}", example.workflow),
            expected_output: example_output_type(&example.workflow),
            feature_flags: example.feature_flags,
            example_path: example.example_path,
        })
        .collect()
}

pub fn docs_parity_claims() -> Vec<ParityClaim> {
    quickstart_descriptors()
        .into_iter()
        .map(|descriptor| {
            let feature = format!(
                "docs::quickstart::{}",
                quickstart_slug(&descriptor.upstream_template)
            );
            let fixtures = if descriptor.parity_status == ParityFeatureStatus::Complete {
                vec![docs_fixture_metadata(
                    &feature,
                    &descriptor.upstream_template,
                    descriptor.rust_example.as_deref().unwrap_or(""),
                )]
            } else {
                Vec::new()
            };
            ParityClaim {
                feature,
                status: descriptor.parity_status,
                fixtures,
            }
        })
        .collect()
}

fn quickstart_descriptor(
    upstream_template: impl Into<String>,
    rust_example: Option<&str>,
    parity_status: ParityFeatureStatus,
    known_gap: Option<&str>,
) -> QuickstartDescriptor {
    QuickstartDescriptor {
        upstream_template: upstream_template.into(),
        rust_example: rust_example.map(str::to_string),
        parity_status,
        known_gap: known_gap.map(str::to_string),
    }
}

fn example_output_type(workflow: &str) -> ExampleOutputType {
    match workflow {
        "testset" => ExampleOutputType::Dataset,
        "evaluate" | "benchmark" => ExampleOutputType::Json,
        _ => ExampleOutputType::Text,
    }
}

fn quickstart_slug(upstream_template: &str) -> &'static str {
    match upstream_template {
        "Evaluate a RAG application" => "evaluate",
        "Generate a testset" => "testset",
        "Compare and monitor evaluation cost" => "benchmark",
        "Run experiments" => "experiments",
        _ => "unknown",
    }
}

fn docs_fixture_metadata(
    feature: &str,
    upstream_template: &str,
    example_path: &str,
) -> ParityFixtureMetadata {
    ParityFixtureMetadata::new(
        feature,
        format!("docs/quickstarts/{}.md", quickstart_slug(upstream_template)),
        Some(example_path.to_string()),
        format!(
            "tests/parity/fixtures/docs_quickstart_{}.json",
            quickstart_slug(upstream_template)
        ),
        ParityFixtureMode::DeterministicMock,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ReleaseBlockerCategory, build_release_blocker_ledger, release_blocking_claims};
    use std::{collections::BTreeSet, path::Path};

    #[test]
    fn test_16_2_1_each_public_workflow_has_a_runnable_rust_example() {
        // SCEN-16.2.1 / AC1 / TEST-16.2.1
        let examples = public_workflow_examples();
        let workflows = examples
            .iter()
            .map(|example| example.workflow.as_str())
            .collect::<BTreeSet<_>>();

        assert!(workflows.contains("evaluate"));
        assert!(workflows.contains("testset"));
        assert!(workflows.contains("benchmark"));
        for example in &examples {
            assert!(
                Path::new(&example.example_path).exists(),
                "{} should exist",
                example.example_path
            );
            assert!(
                example.example_path.ends_with(".rs"),
                "{} should be a Rust example",
                example.example_path
            );
        }
    }

    #[test]
    fn test_16_2_2_examples_map_to_upstream_docs_section_names() {
        // SCEN-16.2.2 / AC2 / TEST-16.2.2
        let sections = public_workflow_examples()
            .into_iter()
            .map(|example| example.upstream_section)
            .collect::<BTreeSet<_>>();

        assert!(sections.contains("Evaluate a RAG application"));
        assert!(sections.contains("Generate a testset"));
        assert!(sections.contains("Compare and monitor evaluation cost"));
    }

    #[test]
    fn test_16_2_3_docs_state_feature_flags_and_known_parity_gaps() {
        // SCEN-16.2.3 / AC3 / TEST-16.2.3
        let docs = std::fs::read_to_string("docs/ragas-rs-user-guide.md").expect("user guide");

        assert!(docs.contains("Feature flags"));
        assert!(docs.contains("Known parity gaps"));
        assert!(docs.contains("no Python runtime bridge"));
        assert!(docs.contains("knowledge graph generation"));
        assert!(
            public_workflow_examples()
                .iter()
                .any(|example| example.feature_flags.contains(&"default".to_string()))
        );
    }

    #[test]
    fn test_21_3_1_quickstart_registry_maps_upstream_templates() {
        // SCEN-21.3.1 / AC1 / TEST-21.3.1
        let descriptors = quickstart_descriptors();
        let by_template = descriptors
            .iter()
            .map(|descriptor| (descriptor.upstream_template.as_str(), descriptor))
            .collect::<std::collections::BTreeMap<_, _>>();

        for expected in [
            "Evaluate a RAG application",
            "Generate a testset",
            "Compare and monitor evaluation cost",
            "Run experiments",
        ] {
            assert!(by_template.contains_key(expected), "missing {expected}");
        }

        let evaluate = by_template
            .get("Evaluate a RAG application")
            .expect("evaluate quickstart");
        assert_eq!(evaluate.rust_example.as_deref(), Some("examples/evaluate.rs"));
        assert_eq!(evaluate.parity_status, ParityFeatureStatus::Complete);

        let experiments = by_template
            .get("Run experiments")
            .expect("experiments quickstart");
        assert_eq!(experiments.rust_example, None);
        assert_eq!(experiments.parity_status, ParityFeatureStatus::KnownGap);
        assert!(
            experiments
                .known_gap
                .as_deref()
                .is_some_and(|gap| gap.contains("example"))
        );
    }

    #[test]
    fn test_21_3_2_runnable_example_metadata_includes_command_output_and_flags() {
        // SCEN-21.3.2 / AC2 / TEST-21.3.2
        let metadata = runnable_example_metadata();
        let by_path = metadata
            .iter()
            .map(|example| (example.example_path.as_str(), example))
            .collect::<std::collections::BTreeMap<_, _>>();

        for expected in [
            "examples/evaluate.rs",
            "examples/testset.rs",
            "examples/benchmark.rs",
        ] {
            let example = by_path.get(expected).unwrap_or_else(|| {
                panic!("missing runnable example metadata for {expected}")
            });
            assert!(Path::new(&example.example_path).exists());
            assert!(example.command.starts_with("cargo run --example "));
            assert!(example.feature_flags.contains(&"default".to_string()));
        }

        assert_eq!(
            by_path["examples/evaluate.rs"].expected_output,
            ExampleOutputType::Json
        );
        assert_eq!(
            by_path["examples/testset.rs"].expected_output,
            ExampleOutputType::Dataset
        );
        assert_eq!(
            by_path["examples/benchmark.rs"].expected_output,
            ExampleOutputType::Json
        );
    }

    #[test]
    fn test_21_3_3_missing_docs_examples_create_release_blocking_claims() {
        // SCEN-21.3.3 / AC3 / TEST-21.3.3
        let claims = docs_parity_claims();
        let blockers = release_blocking_claims(&claims);
        let blocking_features = blockers
            .iter()
            .map(|claim| claim.feature.as_str())
            .collect::<BTreeSet<_>>();

        assert!(
            blocking_features.contains("docs::quickstart::experiments"),
            "missing experiments quickstart release blocker"
        );

        let complete_features = claims
            .iter()
            .filter(|claim| claim.status == ParityFeatureStatus::Complete)
            .map(|claim| claim.feature.as_str())
            .collect::<BTreeSet<_>>();
        for expected in [
            "docs::quickstart::evaluate",
            "docs::quickstart::testset",
            "docs::quickstart::benchmark",
        ] {
            assert!(
                complete_features.contains(expected),
                "missing complete docs claim {expected}"
            );
        }
    }

    #[test]
    fn test_24_1_1_experiment_quickstart_maps_to_runnable_example() {
        // SCEN-24.1.1 / AC1 / TEST-24.1.1
        let descriptors = quickstart_descriptors();
        let experiments = descriptors
            .iter()
            .find(|descriptor| descriptor.upstream_template == "Run experiments")
            .expect("Run experiments quickstart descriptor");

        assert_eq!(
            experiments.rust_example.as_deref(),
            Some("examples/experiment.rs")
        );
        assert_eq!(experiments.parity_status, ParityFeatureStatus::Complete);
        assert_eq!(experiments.known_gap, None);

        let metadata = runnable_example_metadata();
        let experiment_metadata = metadata
            .iter()
            .find(|example| example.example_path == "examples/experiment.rs")
            .expect("experiment quickstart runnable metadata");
        assert_eq!(experiment_metadata.command, "cargo run --example experiment");
        assert_eq!(experiment_metadata.expected_output, ExampleOutputType::Json);
        assert!(experiment_metadata.feature_flags.contains(&"default".to_string()));
    }

    #[test]
    fn test_24_1_2_experiment_quickstart_uses_deterministic_experiment_apis() {
        // SCEN-24.1.2 / AC2 / TEST-24.1.2
        let source =
            std::fs::read_to_string("examples/experiment.rs").expect("experiment example source");

        assert!(source.contains("ExperimentRecord::new"));
        assert!(source.contains("summarize_experiment"));
        assert!(source.contains("compare_runs"));
        assert!(!source.contains("OpenAi"));
        assert!(!source.contains("reqwest"));
        assert!(!source.contains("std::env::var"));
    }

    #[test]
    fn test_24_1_3_experiment_quickstart_is_absent_from_docs_release_blockers() {
        // SCEN-24.1.3 / AC3 / TEST-24.1.3
        let claims = docs_parity_claims();
        let experiment = claims
            .iter()
            .find(|claim| claim.feature == "docs::quickstart::experiments")
            .expect("experiment docs parity claim");

        assert_eq!(experiment.status, ParityFeatureStatus::Complete);
        assert!(
            !experiment.fixtures.is_empty(),
            "complete docs claims require fixture metadata"
        );

        let ledger = build_release_blocker_ledger();
        assert!(!ledger.entries.iter().any(|entry| {
            entry.category == ReleaseBlockerCategory::Docs
                && entry.feature == "docs::quickstart::experiments"
        }));
    }
}
