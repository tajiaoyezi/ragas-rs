use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::RagasError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParityFixture {
    pub feature: String,
    pub upstream_commit: String,
    pub python_baseline: Value,
    pub rust_output: Value,
    pub tolerance: Option<f64>,
    pub known_gap: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ParityFeatureStatus {
    Complete,
    Partial,
    KnownGap,
    NotStarted,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GapMatrixEntry {
    pub feature: String,
    pub status: ParityFeatureStatus,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamBaseline {
    pub main_commit: String,
    pub release_tag: String,
    pub release_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamInventoryEntry {
    pub category: String,
    pub upstream_file_count: usize,
    pub status: ParityFeatureStatus,
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParityFixtureMode {
    DeterministicMock,
    CapturedHttp,
    LiveProvider,
    ManualBaseline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParityFixtureMetadata {
    pub feature: String,
    pub upstream_module_path: String,
    pub upstream_test_path: Option<String>,
    pub fixture_path: String,
    pub mode: ParityFixtureMode,
    pub tolerance: Option<f64>,
}

impl ParityFixtureMetadata {
    pub fn new(
        feature: impl Into<String>,
        upstream_module_path: impl Into<String>,
        upstream_test_path: Option<String>,
        fixture_path: impl Into<String>,
        mode: ParityFixtureMode,
        tolerance: Option<f64>,
    ) -> Self {
        Self {
            feature: feature.into(),
            upstream_module_path: upstream_module_path.into(),
            upstream_test_path,
            fixture_path: fixture_path.into(),
            mode,
            tolerance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParityClaim {
    pub feature: String,
    pub status: ParityFeatureStatus,
    pub fixtures: Vec<ParityFixtureMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParityCheck {
    pub feature: String,
    pub passed: bool,
    pub drift: Option<String>,
}

pub fn latest_upstream_baseline() -> UpstreamBaseline {
    UpstreamBaseline {
        main_commit: "298b68274234c060deacab3cf5fb52aa3a20e885".to_string(),
        release_tag: "v0.4.3".to_string(),
        release_commit: "4ecab384fda829ca50bec3f07cc49589d756e172".to_string(),
    }
}

pub fn latest_upstream_inventory() -> Vec<UpstreamInventoryEntry> {
    vec![
        inventory_entry(
            "top-level",
            22,
            ParityFeatureStatus::Partial,
            "runtime modules exist but full analytics/config/sdk/tokenizer parity is not proven",
        ),
        inventory_entry(
            "backends",
            10,
            ParityFeatureStatus::Partial,
            "in-memory and local JSONL/CSV exist; gdrive and registry parity remain",
        ),
        inventory_entry(
            "embeddings",
            8,
            ParityFeatureStatus::Partial,
            "generic embedding traits exist; provider-specific parity remains",
        ),
        inventory_entry(
            "integrations",
            15,
            ParityFeatureStatus::KnownGap,
            "generic tracing/redaction exists; upstream integration adapters remain",
        ),
        inventory_entry(
            "llms",
            9,
            ParityFeatureStatus::Partial,
            "OpenAI-compatible and Azure configuration exist; Instructor/LiteLLM/Haystack/OCI parity remains",
        ),
        inventory_entry(
            "metrics",
            114,
            ParityFeatureStatus::Partial,
            "metric families exist but full golden parity fixtures are not complete",
        ),
        inventory_entry(
            "optimizers",
            7,
            ParityFeatureStatus::Partial,
            "genetic optimizer exists; DSPy/MIPROv2 parity remains",
        ),
        inventory_entry(
            "prompt",
            23,
            ParityFeatureStatus::Partial,
            "prompt scaffolding exists; language adaptation and save/load parity remain",
        ),
        inventory_entry(
            "testset",
            33,
            ParityFeatureStatus::KnownGap,
            "graph and synthesizer scaffolds exist; upstream LLM-driven generation parity remains",
        ),
    ]
}

pub fn release_blocking_inventory(
    entries: &[UpstreamInventoryEntry],
) -> Vec<&UpstreamInventoryEntry> {
    entries
        .iter()
        .filter(|entry| entry.status != ParityFeatureStatus::Complete)
        .collect()
}

pub fn validate_parity_claim(claim: &ParityClaim) -> Result<(), RagasError> {
    if claim.status == ParityFeatureStatus::Complete && claim.fixtures.is_empty() {
        return Err(RagasError::Parse {
            message: format!(
                "parity-complete claim for {} requires fixture evidence",
                claim.feature
            ),
        });
    }
    Ok(())
}

pub fn release_blocking_claims(claims: &[ParityClaim]) -> Vec<&ParityClaim> {
    claims
        .iter()
        .filter(|claim| {
            claim.status != ParityFeatureStatus::Complete || validate_parity_claim(claim).is_err()
        })
        .collect()
}

fn inventory_entry(
    category: &str,
    upstream_file_count: usize,
    status: ParityFeatureStatus,
    rationale: &str,
) -> UpstreamInventoryEntry {
    UpstreamInventoryEntry {
        category: category.to_string(),
        upstream_file_count,
        status,
        rationale: rationale.to_string(),
    }
}

pub fn parse_parity_fixture(input: &str) -> Result<ParityFixture, RagasError> {
    serde_json::from_str(input).map_err(|error| RagasError::Parse {
        message: format!("parity fixture parse failed: {error}"),
    })
}

pub fn validate_gap_matrix(entries: &[GapMatrixEntry]) -> BTreeSet<ParityFeatureStatus> {
    entries.iter().map(|entry| entry.status).collect()
}

pub fn check_parity_fixture(
    fixture: &ParityFixture,
    status: ParityFeatureStatus,
) -> Result<ParityCheck, RagasError> {
    let drift = semantic_drift(fixture);
    if let Some(drift) = drift {
        if status == ParityFeatureStatus::Complete && fixture.known_gap.is_none() {
            return Err(RagasError::Parse {
                message: format!("undeclared semantic drift for {}: {drift}", fixture.feature),
            });
        }
        return Ok(ParityCheck {
            feature: fixture.feature.clone(),
            passed: false,
            drift: Some(drift),
        });
    }

    Ok(ParityCheck {
        feature: fixture.feature.clone(),
        passed: true,
        drift: None,
    })
}

fn semantic_drift(fixture: &ParityFixture) -> Option<String> {
    let tolerance = fixture.tolerance.unwrap_or(0.0);
    let baseline_score = fixture.python_baseline.get("score").and_then(Value::as_f64);
    let rust_score = fixture.rust_output.get("score").and_then(Value::as_f64);
    if let (Some(baseline_score), Some(rust_score)) = (baseline_score, rust_score) {
        if (baseline_score - rust_score).abs() <= tolerance {
            return None;
        }
        return Some(format!(
            "score baseline={baseline_score} rust={rust_score} tolerance={tolerance}"
        ));
    }

    (fixture.python_baseline != fixture.rust_output).then(|| {
        format!(
            "baseline={} rust={}",
            fixture.python_baseline, fixture.rust_output
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_16_1_1_parity_fixture_format_stores_python_baseline_and_rust_output() {
        // SCEN-16.1.1 / AC1 / TEST-16.1.1
        let fixture = parse_parity_fixture(include_str!(
            "../../tests/parity/fixtures/context_precision.json"
        ))
        .expect("fixture parses");

        assert_eq!(fixture.feature, "context_precision");
        assert_eq!(fixture.upstream_commit, "298b682");
        assert_eq!(fixture.python_baseline["score"], 0.75);
        assert_eq!(fixture.rust_output["score"], 0.75);
        assert_eq!(fixture.tolerance, Some(1e-9));
    }

    #[test]
    fn test_16_1_2_gap_matrix_lists_complete_partial_and_known_gap() {
        // SCEN-16.1.2 / AC2 / TEST-16.1.2
        let entries = vec![
            GapMatrixEntry {
                feature: "context_precision".to_string(),
                status: ParityFeatureStatus::Complete,
                rationale: "fixture exact".to_string(),
            },
            GapMatrixEntry {
                feature: "summarization".to_string(),
                status: ParityFeatureStatus::Partial,
                rationale: "judge wording differs".to_string(),
            },
            GapMatrixEntry {
                feature: "knowledge_graph_generation".to_string(),
                status: ParityFeatureStatus::KnownGap,
                rationale: "out of v1.0 scope".to_string(),
            },
        ];

        let statuses = validate_gap_matrix(&entries);

        assert!(statuses.contains(&ParityFeatureStatus::Complete));
        assert!(statuses.contains(&ParityFeatureStatus::Partial));
        assert!(statuses.contains(&ParityFeatureStatus::KnownGap));
        assert_eq!(statuses.len(), 3);
    }

    #[test]
    fn test_16_1_3_parity_tests_fail_on_undeclared_semantic_drift() {
        // SCEN-16.1.3 / AC3 / TEST-16.1.3
        let drift_fixture = ParityFixture {
            feature: "faithfulness".to_string(),
            upstream_commit: "298b682".to_string(),
            python_baseline: serde_json::json!({"score": 0.8}),
            rust_output: serde_json::json!({"score": 0.6}),
            tolerance: Some(0.0),
            known_gap: None,
        };

        let error = check_parity_fixture(&drift_fixture, ParityFeatureStatus::Complete)
            .expect_err("undeclared drift should fail");
        assert!(error.to_string().contains("undeclared semantic drift"));

        let known_gap = check_parity_fixture(&drift_fixture, ParityFeatureStatus::KnownGap)
            .expect("known gap records drift without failing");
        assert!(!known_gap.passed);
        assert!(known_gap.drift.expect("drift detail").contains("0.8"));
    }

    #[test]
    fn test_17_1_1_latest_upstream_baseline_records_main_and_release() {
        // SCEN-17.1.1 / AC1 / TEST-17.1.1
        let baseline = latest_upstream_baseline();

        assert_eq!(
            baseline.main_commit,
            "298b68274234c060deacab3cf5fb52aa3a20e885"
        );
        assert_eq!(baseline.release_tag, "v0.4.3");
        assert_eq!(
            baseline.release_commit,
            "4ecab384fda829ca50bec3f07cc49589d756e172"
        );
    }

    #[test]
    fn test_17_1_2_inventory_covers_upstream_source_categories() {
        // SCEN-17.1.2 / AC2 / TEST-17.1.2
        let inventory = latest_upstream_inventory();
        let categories: BTreeSet<_> = inventory
            .iter()
            .map(|entry| entry.category.as_str())
            .collect();

        for expected in [
            "top-level",
            "backends",
            "embeddings",
            "integrations",
            "llms",
            "metrics",
            "optimizers",
            "prompt",
            "testset",
        ] {
            assert!(categories.contains(expected), "missing category {expected}");
        }
        assert_eq!(inventory.len(), 9);
        assert!(inventory.iter().all(|entry| entry.upstream_file_count > 0));
    }

    #[test]
    fn test_17_1_3_incomplete_inventory_blocks_release_readiness() {
        // SCEN-17.1.3 / AC3 / TEST-17.1.3
        let entries = vec![
            UpstreamInventoryEntry {
                category: "metrics".to_string(),
                upstream_file_count: 114,
                status: ParityFeatureStatus::Complete,
                rationale: "all fixtures present".to_string(),
            },
            UpstreamInventoryEntry {
                category: "testset".to_string(),
                upstream_file_count: 33,
                status: ParityFeatureStatus::Partial,
                rationale: "synthesizer prompt parity missing".to_string(),
            },
        ];

        let blockers = release_blocking_inventory(&entries);

        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].category, "testset");
    }

    #[test]
    fn test_17_2_1_parity_complete_requires_fixture_evidence() {
        // SCEN-17.2.1 / AC1 / TEST-17.2.1
        let claim = ParityClaim {
            feature: "faithfulness".to_string(),
            status: ParityFeatureStatus::Complete,
            fixtures: Vec::new(),
        };

        let error = validate_parity_claim(&claim).expect_err("missing fixtures should fail");
        assert!(error.to_string().contains("fixture evidence"));
    }

    #[test]
    fn test_17_2_2_fixture_metadata_records_source_mode_and_tolerance() {
        // SCEN-17.2.2 / AC2 / TEST-17.2.2
        let metadata = ParityFixtureMetadata::new(
            "context_precision",
            "src/ragas/metrics/collections/context_precision/metric.py",
            Some("tests/e2e/metrics_migration/test_context_precision_migration.py".to_string()),
            "tests/parity/fixtures/context_precision.json",
            ParityFixtureMode::DeterministicMock,
            Some(1e-9),
        );

        assert_eq!(metadata.feature, "context_precision");
        assert_eq!(
            metadata.upstream_module_path,
            "src/ragas/metrics/collections/context_precision/metric.py"
        );
        assert_eq!(
            metadata.upstream_test_path.as_deref(),
            Some("tests/e2e/metrics_migration/test_context_precision_migration.py")
        );
        assert_eq!(
            metadata.fixture_path,
            "tests/parity/fixtures/context_precision.json"
        );
        assert_eq!(metadata.mode, ParityFixtureMode::DeterministicMock);
        assert_eq!(metadata.tolerance, Some(1e-9));
    }

    #[test]
    fn test_17_2_3_partial_and_known_gap_claims_block_release_by_default() {
        // SCEN-17.2.3 / AC3 / TEST-17.2.3
        let complete = ParityClaim {
            feature: "context_precision".to_string(),
            status: ParityFeatureStatus::Complete,
            fixtures: vec![ParityFixtureMetadata::new(
                "context_precision",
                "src/ragas/metrics/collections/context_precision/metric.py",
                None,
                "tests/parity/fixtures/context_precision.json",
                ParityFixtureMode::DeterministicMock,
                Some(1e-9),
            )],
        };
        let partial = ParityClaim {
            feature: "summarization".to_string(),
            status: ParityFeatureStatus::Partial,
            fixtures: Vec::new(),
        };
        let known_gap = ParityClaim {
            feature: "knowledge_graph_generation".to_string(),
            status: ParityFeatureStatus::KnownGap,
            fixtures: Vec::new(),
        };

        let claims = vec![complete, partial, known_gap];
        let blockers = release_blocking_claims(&claims);

        assert_eq!(blockers.len(), 2);
        assert_eq!(blockers[0].feature, "summarization");
        assert_eq!(blockers[1].feature, "knowledge_graph_generation");
    }
}
