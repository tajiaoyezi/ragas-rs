#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QualityGateKind {
    Build,
    Typecheck,
    Unit,
    Integration,
    Parity,
    Examples,
    Coverage,
    FuzzProperty,
    BugLedgerAudit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GateEvidenceStatus {
    Passed,
    Failed,
    SkippedWithJustification,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityGateEvidence {
    pub kind: QualityGateKind,
    pub status: GateEvidenceStatus,
    pub detail: String,
}

impl QualityGateEvidence {
    pub fn new(
        _kind: QualityGateKind,
        _status: GateEvidenceStatus,
        _detail: impl Into<String>,
    ) -> Self {
        Self {
            kind: QualityGateKind::Build,
            status: GateEvidenceStatus::Missing,
            detail: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityGateSummary {
    pub passed: usize,
    pub failed: usize,
    pub skipped_with_justification: usize,
    pub missing: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseGateReport {
    pub evidence: Vec<QualityGateEvidence>,
}

pub fn release_gate_files() -> Vec<&'static str> {
    vec![
        "Cargo.toml",
        ".github/workflows/ci.yml",
        "docs/release-checklist.md",
    ]
}

pub fn required_quality_gates() -> Vec<QualityGateKind> {
    vec![
        QualityGateKind::Build,
        QualityGateKind::Typecheck,
        QualityGateKind::Unit,
    ]
}

pub fn summarize_quality_gates(_report: &ReleaseGateReport) -> QualityGateSummary {
    QualityGateSummary {
        passed: 0,
        failed: 0,
        skipped_with_justification: 0,
        missing: 0,
    }
}

pub fn quality_gate_blockers(_report: &ReleaseGateReport) -> Vec<QualityGateEvidence> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn test_16_3_1_cargo_features_match_optional_capability_groups() {
        // SCEN-16.3.1 / AC1 / TEST-16.3.1
        let cargo = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");

        assert!(cargo.contains("[features]"));
        assert!(cargo.contains("default = ["));
        assert!(cargo.contains("runtime-tokio"));
        assert!(cargo.contains("providers-openai"));
        assert!(cargo.contains("integrations"));
        assert!(cargo.contains("benchmarks"));
        assert!(cargo.contains("parity"));
        assert!(cargo.contains("docs-examples"));
    }

    #[test]
    fn test_16_3_2_ci_runs_build_check_test_and_parity_gates() {
        // SCEN-16.3.2 / AC2 / TEST-16.3.2
        let ci = std::fs::read_to_string(".github/workflows/ci.yml").expect("CI workflow");

        assert!(ci.contains("cargo build"));
        assert!(ci.contains("cargo check"));
        assert!(ci.contains("cargo test"));
        assert!(ci.contains("cargo test parity::"));
        assert!(release_gate_files().contains(&".github/workflows/ci.yml"));
    }

    #[test]
    fn test_16_3_3_release_checklist_includes_versioning_and_rollback_steps() {
        // SCEN-16.3.3 / AC3 / TEST-16.3.3
        let checklist =
            std::fs::read_to_string("docs/release-checklist.md").expect("release checklist");

        assert!(checklist.contains("Versioning"));
        assert!(checklist.contains("Rollback"));
        assert!(checklist.contains("cargo publish --dry-run"));
        assert!(checklist.contains("cargo yank"));
        assert!(checklist.contains("dependency lock rollback"));
    }

    #[test]
    fn test_17_3_1_required_gate_types_cover_strong_quality_model() {
        // SCEN-17.3.1 / AC1 / TEST-17.3.1
        let gates: BTreeSet<_> = required_quality_gates().into_iter().collect();

        for gate in [
            QualityGateKind::Build,
            QualityGateKind::Typecheck,
            QualityGateKind::Unit,
            QualityGateKind::Integration,
            QualityGateKind::Parity,
            QualityGateKind::Examples,
            QualityGateKind::Coverage,
            QualityGateKind::FuzzProperty,
            QualityGateKind::BugLedgerAudit,
        ] {
            assert!(gates.contains(&gate), "missing gate {gate:?}");
        }
    }

    #[test]
    fn test_17_3_2_release_gate_report_distinguishes_evidence_states() {
        // SCEN-17.3.2 / AC2 / TEST-17.3.2
        let report = ReleaseGateReport {
            evidence: vec![
                QualityGateEvidence::new(QualityGateKind::Build, GateEvidenceStatus::Passed, "ok"),
                QualityGateEvidence::new(
                    QualityGateKind::Integration,
                    GateEvidenceStatus::Failed,
                    "integration failure",
                ),
                QualityGateEvidence::new(
                    QualityGateKind::Coverage,
                    GateEvidenceStatus::SkippedWithJustification,
                    "tool unavailable on this platform",
                ),
                QualityGateEvidence::new(
                    QualityGateKind::FuzzProperty,
                    GateEvidenceStatus::Missing,
                    "not run",
                ),
            ],
        };

        let summary = summarize_quality_gates(&report);

        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.skipped_with_justification, 1);
        assert_eq!(summary.missing, 1);
    }

    #[test]
    fn test_17_3_3_missing_required_gate_evidence_blocks_release() {
        // SCEN-17.3.3 / AC3 / TEST-17.3.3
        let report = ReleaseGateReport {
            evidence: vec![
                QualityGateEvidence::new(QualityGateKind::Build, GateEvidenceStatus::Passed, "ok"),
                QualityGateEvidence::new(
                    QualityGateKind::Parity,
                    GateEvidenceStatus::Missing,
                    "parity suite did not run",
                ),
            ],
        };

        let blockers = quality_gate_blockers(&report);

        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].kind, QualityGateKind::Parity);
    }
}
