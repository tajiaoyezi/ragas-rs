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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GapMatrixEntry {
    pub feature: String,
    pub status: ParityFeatureStatus,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParityCheck {
    pub feature: String,
    pub passed: bool,
    pub drift: Option<String>,
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
}
