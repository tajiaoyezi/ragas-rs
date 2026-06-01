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
        kind: QualityGateKind,
        status: GateEvidenceStatus,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            status,
            detail: detail.into(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BugSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BugStatus {
    Open,
    InProgress,
    Resolved,
    Waived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BugClass {
    Correctness,
    Safety,
    DataLoss,
    Panic,
    Security,
    Parity,
    Documentation,
    Performance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BugLedgerEntry {
    pub id: String,
    pub severity: BugSeverity,
    pub status: BugStatus,
    pub class: BugClass,
    pub affected_feature: String,
    pub evidence: String,
    pub regression_test: String,
}

impl BugLedgerEntry {
    pub fn new(
        id: impl Into<String>,
        severity: BugSeverity,
        status: BugStatus,
        class: BugClass,
        affected_feature: impl Into<String>,
        evidence: impl Into<String>,
        regression_test: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity,
            status,
            class,
            affected_feature: affected_feature.into(),
            evidence: evidence.into(),
            regression_test: regression_test.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BugZeroAudit {
    pub unresolved_release_blocking: usize,
    pub release_ready: bool,
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
        QualityGateKind::Integration,
        QualityGateKind::Parity,
        QualityGateKind::Examples,
        QualityGateKind::Coverage,
        QualityGateKind::FuzzProperty,
        QualityGateKind::BugLedgerAudit,
    ]
}

pub fn summarize_quality_gates(report: &ReleaseGateReport) -> QualityGateSummary {
    report.evidence.iter().fold(
        QualityGateSummary {
            passed: 0,
            failed: 0,
            skipped_with_justification: 0,
            missing: 0,
        },
        |mut summary, evidence| {
            match evidence.status {
                GateEvidenceStatus::Passed => summary.passed += 1,
                GateEvidenceStatus::Failed => summary.failed += 1,
                GateEvidenceStatus::SkippedWithJustification => {
                    summary.skipped_with_justification += 1
                }
                GateEvidenceStatus::Missing => summary.missing += 1,
            }
            summary
        },
    )
}

pub fn quality_gate_blockers(report: &ReleaseGateReport) -> Vec<QualityGateEvidence> {
    report
        .evidence
        .iter()
        .filter(|evidence| {
            matches!(
                evidence.status,
                GateEvidenceStatus::Failed | GateEvidenceStatus::Missing
            )
        })
        .cloned()
        .collect()
}

pub fn release_blocking_bugs(entries: &[BugLedgerEntry]) -> Vec<BugLedgerEntry> {
    entries
        .iter()
        .filter(|entry| is_unresolved(entry.status))
        .filter(|entry| is_release_blocking_severity(entry.severity))
        .filter(|entry| is_release_blocking_class(entry.class))
        .cloned()
        .collect()
}

pub fn summarize_bug_zero_audit(entries: &[BugLedgerEntry]) -> BugZeroAudit {
    let unresolved_release_blocking = release_blocking_bugs(entries).len();
    BugZeroAudit {
        unresolved_release_blocking,
        release_ready: unresolved_release_blocking == 0,
    }
}

fn is_unresolved(status: BugStatus) -> bool {
    matches!(status, BugStatus::Open | BugStatus::InProgress)
}

fn is_release_blocking_severity(severity: BugSeverity) -> bool {
    matches!(severity, BugSeverity::Critical | BugSeverity::High)
}

fn is_release_blocking_class(class: BugClass) -> bool {
    matches!(
        class,
        BugClass::Correctness
            | BugClass::Safety
            | BugClass::DataLoss
            | BugClass::Panic
            | BugClass::Security
            | BugClass::Parity
    )
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

    #[test]
    fn test_17_4_1_bug_ledger_entries_record_release_evidence() {
        // SCEN-17.4.1 / AC1 / TEST-17.4.1
        let entry = BugLedgerEntry::new(
            "BUG-001",
            BugSeverity::High,
            BugStatus::Open,
            BugClass::Parity,
            "context_precision",
            "fixture drift score baseline=0.75 rust=0.50",
            "TEST-17.4.1",
        );

        assert_eq!(entry.id, "BUG-001");
        assert_eq!(entry.severity, BugSeverity::High);
        assert_eq!(entry.status, BugStatus::Open);
        assert_eq!(entry.class, BugClass::Parity);
        assert_eq!(entry.affected_feature, "context_precision");
        assert!(entry.evidence.contains("fixture drift"));
        assert_eq!(entry.regression_test, "TEST-17.4.1");
    }

    #[test]
    fn test_17_4_2_unresolved_high_or_critical_correctness_classes_block_release() {
        // SCEN-17.4.2 / AC2 / TEST-17.4.2
        let bugs = vec![
            BugLedgerEntry::new(
                "BUG-SEC",
                BugSeverity::Critical,
                BugStatus::Open,
                BugClass::Security,
                "provider_errors",
                "auth header leaked",
                "TEST-security-redaction",
            ),
            BugLedgerEntry::new(
                "BUG-PARITY",
                BugSeverity::High,
                BugStatus::InProgress,
                BugClass::Parity,
                "faithfulness",
                "golden fixture mismatch",
                "TEST-faithfulness-parity",
            ),
            BugLedgerEntry::new(
                "BUG-DOC",
                BugSeverity::High,
                BugStatus::Open,
                BugClass::Documentation,
                "quickstart",
                "wording issue",
                "TEST-docs",
            ),
            BugLedgerEntry::new(
                "BUG-OLD",
                BugSeverity::Critical,
                BugStatus::Resolved,
                BugClass::Correctness,
                "dataset",
                "fixed",
                "TEST-dataset-regression",
            ),
        ];

        let blockers = release_blocking_bugs(&bugs);

        assert_eq!(blockers.len(), 2);
        assert_eq!(blockers[0].id, "BUG-SEC");
        assert_eq!(blockers[1].id, "BUG-PARITY");
    }

    #[test]
    fn test_17_4_3_bug_zero_audit_reports_no_unresolved_blockers_before_ready() {
        // SCEN-17.4.3 / AC3 / TEST-17.4.3
        let bugs = vec![BugLedgerEntry::new(
            "BUG-RESOLVED",
            BugSeverity::High,
            BugStatus::Resolved,
            BugClass::Correctness,
            "dataset",
            "regression test passes",
            "TEST-dataset-regression",
        )];

        let audit = summarize_bug_zero_audit(&bugs);

        assert_eq!(audit.unresolved_release_blocking, 0);
        assert!(audit.release_ready);

        let checklist =
            std::fs::read_to_string("docs/release-checklist.md").expect("release checklist");
        assert!(checklist.contains("No-known-bug audit"));
        assert!(checklist.contains("zero unresolved release-blocking bugs"));
    }
}
