use crate::parity::{
    MetricGoldenComparison, MetricGoldenOutcome, ParityClaim, ParityFeatureStatus,
    validate_parity_claim,
};

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
    PanicSafety,
    Mutation,
    BugLedgerAudit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QualityEvidenceKind {
    Property,
    Fuzz,
    Coverage,
    PanicSafety,
    Mutation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SafetyFailureClass {
    DirectPanic,
    AsyncTaskPanic,
    UnwindBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QualityGateMode {
    RequiredDefaultCi,
    RequiredReleaseEvidence,
    OptionalLongRunning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityGateDescriptor {
    pub gate_id: &'static str,
    pub evidence_kind: QualityEvidenceKind,
    pub gate_kind: QualityGateKind,
    pub command: &'static str,
    pub scope: &'static str,
    pub mode: QualityGateMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanicSafetyGateDescriptor {
    pub gate_id: &'static str,
    pub command: &'static str,
    pub scope: &'static str,
    pub failure_classes: Vec<SafetyFailureClass>,
    pub mode: QualityGateMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationGateDescriptor {
    pub gate_id: &'static str,
    pub tool: &'static str,
    pub command: &'static str,
    pub scope: &'static str,
    pub threshold_percent: u8,
    pub mode: QualityGateMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityCommandEvidence {
    pub gate_id: String,
    pub status: GateEvidenceStatus,
    pub detail: String,
}

impl QualityCommandEvidence {
    pub fn new(
        gate_id: impl Into<String>,
        status: GateEvidenceStatus,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            gate_id: gate_id.into(),
            status,
            detail: detail.into(),
        }
    }
}

impl QualityGateMode {
    pub fn is_required(self) -> bool {
        matches!(
            self,
            QualityGateMode::RequiredDefaultCi | QualityGateMode::RequiredReleaseEvidence
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityEvidenceFinding {
    pub gate_id: String,
    pub evidence_kind: QualityEvidenceKind,
    pub command: String,
    pub detail: String,
    pub release_blocking: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MetricReleaseBlockerSource {
    Catalog,
    FixtureDrift,
    Unclassified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricReleaseBlocker {
    pub feature: String,
    pub source: MetricReleaseBlockerSource,
    pub status: ParityFeatureStatus,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricReleaseBlockerSummary {
    pub blocker_count: usize,
    pub features: Vec<String>,
    pub release_ready: bool,
}

pub fn metric_release_blockers(
    catalog_claims: &[ParityClaim],
    fixture_comparisons: &[MetricGoldenComparison],
    unclassified_metric_names: &[&str],
) -> Vec<MetricReleaseBlocker> {
    let mut blockers = Vec::new();

    for claim in catalog_claims {
        if claim.status != ParityFeatureStatus::Complete || validate_parity_claim(claim).is_err() {
            blockers.push(MetricReleaseBlocker {
                feature: claim.feature.clone(),
                source: MetricReleaseBlockerSource::Catalog,
                status: claim.status,
                detail: format!("metric catalog parity status is {:?}", claim.status),
            });
        }
    }

    for comparison in fixture_comparisons {
        let status = match comparison.outcome {
            MetricGoldenOutcome::ExactMatch | MetricGoldenOutcome::ToleratedNumericDrift => None,
            MetricGoldenOutcome::KnownGap => Some(ParityFeatureStatus::KnownGap),
            MetricGoldenOutcome::UndeclaredDrift => Some(ParityFeatureStatus::Blocked),
        };
        if let Some(status) = status {
            blockers.push(MetricReleaseBlocker {
                feature: comparison.feature.clone(),
                source: MetricReleaseBlockerSource::FixtureDrift,
                status,
                detail: comparison.drift.clone().unwrap_or_else(|| {
                    format!("metric fixture outcome is {:?}", comparison.outcome)
                }),
            });
        }
    }

    for metric_name in unclassified_metric_names {
        let feature = if metric_name.starts_with("metric::") {
            (*metric_name).to_string()
        } else {
            format!("metric::{metric_name}")
        };
        blockers.push(MetricReleaseBlocker {
            feature,
            source: MetricReleaseBlockerSource::Unclassified,
            status: ParityFeatureStatus::NotStarted,
            detail: "metric is absent from the upstream catalog inventory".to_string(),
        });
    }

    blockers
}

pub fn summarize_metric_release_blockers(
    blockers: &[MetricReleaseBlocker],
) -> MetricReleaseBlockerSummary {
    MetricReleaseBlockerSummary {
        blocker_count: blockers.len(),
        features: blockers
            .iter()
            .map(|blocker| blocker.feature.clone())
            .collect(),
        release_ready: blockers.is_empty(),
    }
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
        QualityGateKind::PanicSafety,
        QualityGateKind::Mutation,
        QualityGateKind::BugLedgerAudit,
    ]
}

pub fn property_fuzz_coverage_gate_descriptors() -> Vec<QualityGateDescriptor> {
    vec![
        QualityGateDescriptor {
            gate_id: "quality::property::invariants",
            evidence_kind: QualityEvidenceKind::Property,
            gate_kind: QualityGateKind::FuzzProperty,
            command: "cargo test property::",
            scope: "src/",
            mode: QualityGateMode::RequiredDefaultCi,
        },
        QualityGateDescriptor {
            gate_id: "quality::fuzz::smoke-corpus",
            evidence_kind: QualityEvidenceKind::Fuzz,
            gate_kind: QualityGateKind::FuzzProperty,
            command: "cargo test fuzz_smoke::",
            scope: "src/",
            mode: QualityGateMode::RequiredDefaultCi,
        },
        QualityGateDescriptor {
            gate_id: "quality::coverage::llvm-cov-summary",
            evidence_kind: QualityEvidenceKind::Coverage,
            gate_kind: QualityGateKind::Coverage,
            command: "cargo llvm-cov --summary-only",
            scope: "src/",
            mode: QualityGateMode::RequiredReleaseEvidence,
        },
        QualityGateDescriptor {
            gate_id: "quality::fuzz::long-running-campaign",
            evidence_kind: QualityEvidenceKind::Fuzz,
            gate_kind: QualityGateKind::FuzzProperty,
            command: "cargo fuzz run ragas_evaluation -- -max_total_time=3600",
            scope: "fuzz/",
            mode: QualityGateMode::OptionalLongRunning,
        },
    ]
}

pub fn required_quality_evidence_blockers(
    descriptors: &[QualityGateDescriptor],
    evidence: &[QualityCommandEvidence],
) -> Vec<QualityEvidenceFinding> {
    descriptors
        .iter()
        .filter(|descriptor| descriptor.mode.is_required())
        .filter_map(|descriptor| {
            match evidence
                .iter()
                .find(|evidence| evidence.gate_id == descriptor.gate_id)
            {
                Some(record) => match record.status {
                    GateEvidenceStatus::Passed | GateEvidenceStatus::SkippedWithJustification => {
                        None
                    }
                    GateEvidenceStatus::Failed | GateEvidenceStatus::Missing => {
                        Some(QualityEvidenceFinding {
                            gate_id: descriptor.gate_id.to_string(),
                            evidence_kind: descriptor.evidence_kind,
                            command: descriptor.command.to_string(),
                            detail: record.detail.clone(),
                            release_blocking: true,
                        })
                    }
                },
                None => Some(QualityEvidenceFinding {
                    gate_id: descriptor.gate_id.to_string(),
                    evidence_kind: descriptor.evidence_kind,
                    command: descriptor.command.to_string(),
                    detail: "required quality evidence is missing".to_string(),
                    release_blocking: true,
                }),
            }
        })
        .collect()
}

pub fn panic_safety_gate_descriptors() -> Vec<PanicSafetyGateDescriptor> {
    vec![PanicSafetyGateDescriptor {
        gate_id: "quality::panic::unwind-boundaries",
        command: "cargo test panic_safety::",
        scope: "src/",
        failure_classes: vec![
            SafetyFailureClass::DirectPanic,
            SafetyFailureClass::AsyncTaskPanic,
            SafetyFailureClass::UnwindBoundary,
        ],
        mode: QualityGateMode::RequiredDefaultCi,
    }]
}

pub fn mutation_gate_descriptors() -> Vec<MutationGateDescriptor> {
    vec![
        MutationGateDescriptor {
            gate_id: "quality::mutation::release-threshold",
            tool: "cargo-mutants",
            command: "cargo mutants --minimum-test-timeout 60 --timeout 300",
            scope: "src/",
            threshold_percent: 80,
            mode: QualityGateMode::RequiredReleaseEvidence,
        },
        MutationGateDescriptor {
            gate_id: "quality::mutation::extended-campaign",
            tool: "cargo-mutants",
            command: "cargo mutants --minimum-test-timeout 60 --timeout 900",
            scope: "src/",
            threshold_percent: 90,
            mode: QualityGateMode::OptionalLongRunning,
        },
    ]
}

pub fn panic_mutation_quality_gate_descriptors() -> Vec<QualityGateDescriptor> {
    panic_safety_gate_descriptors()
        .into_iter()
        .map(|descriptor| QualityGateDescriptor {
            gate_id: descriptor.gate_id,
            evidence_kind: QualityEvidenceKind::PanicSafety,
            gate_kind: QualityGateKind::PanicSafety,
            command: descriptor.command,
            scope: descriptor.scope,
            mode: descriptor.mode,
        })
        .chain(
            mutation_gate_descriptors()
                .into_iter()
                .map(|descriptor| QualityGateDescriptor {
                    gate_id: descriptor.gate_id,
                    evidence_kind: QualityEvidenceKind::Mutation,
                    gate_kind: QualityGateKind::Mutation,
                    command: descriptor.command,
                    scope: descriptor.scope,
                    mode: descriptor.mode,
                }),
        )
        .collect()
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

    use crate::{
        MetricGoldenComparison, MetricGoldenOutcome, metric_catalog_parity_claims,
        release_blocking_claims,
    };

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
            QualityGateKind::PanicSafety,
            QualityGateKind::Mutation,
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
    fn test_22_1_1_property_fuzz_and_coverage_gates_declare_commands_scope_and_mode() {
        // SCEN-22.1.1 / AC1 / TEST-22.1.1
        let descriptors = property_fuzz_coverage_gate_descriptors();

        assert!(descriptors.iter().all(|gate| !gate.gate_id.is_empty()));
        assert!(descriptors.iter().all(|gate| !gate.command.is_empty()));
        assert!(descriptors.iter().all(|gate| !gate.scope.is_empty()));
        assert!(descriptors.iter().any(|gate| {
            gate.evidence_kind == QualityEvidenceKind::Property
                && gate.gate_kind == QualityGateKind::FuzzProperty
                && gate.command == "cargo test property::"
                && gate.scope == "src/"
                && gate.mode == QualityGateMode::RequiredDefaultCi
        }));
        assert!(descriptors.iter().any(|gate| {
            gate.evidence_kind == QualityEvidenceKind::Fuzz
                && gate.gate_kind == QualityGateKind::FuzzProperty
                && gate.command == "cargo test fuzz_smoke::"
                && gate.scope == "src/"
                && gate.mode == QualityGateMode::RequiredDefaultCi
        }));
        assert!(descriptors.iter().any(|gate| {
            gate.evidence_kind == QualityEvidenceKind::Coverage
                && gate.gate_kind == QualityGateKind::Coverage
                && gate.command == "cargo llvm-cov --summary-only"
                && gate.scope == "src/"
                && gate.mode == QualityGateMode::RequiredReleaseEvidence
        }));
    }

    #[test]
    fn test_22_1_2_missing_required_quality_evidence_blocks_release() {
        // SCEN-22.1.2 / AC2 / TEST-22.1.2
        let descriptors = property_fuzz_coverage_gate_descriptors();
        let evidence = vec![QualityCommandEvidence::new(
            "quality::property::invariants",
            GateEvidenceStatus::Passed,
            "bounded property suite passed",
        )];

        let blockers = required_quality_evidence_blockers(&descriptors, &evidence);

        assert!(blockers.iter().any(|finding| {
            finding.gate_id == "quality::coverage::llvm-cov-summary"
                && finding.evidence_kind == QualityEvidenceKind::Coverage
                && finding.command == "cargo llvm-cov --summary-only"
                && finding.release_blocking
        }));
        assert!(blockers.iter().any(|finding| {
            finding.gate_id == "quality::fuzz::smoke-corpus"
                && finding.evidence_kind == QualityEvidenceKind::Fuzz
                && finding.command == "cargo test fuzz_smoke::"
                && finding.release_blocking
        }));
        assert!(
            !blockers
                .iter()
                .any(|finding| finding.gate_id == "quality::fuzz::long-running-campaign")
        );
    }

    #[test]
    fn test_22_1_3_optional_long_running_gates_do_not_block_default_ci() {
        // SCEN-22.1.3 / AC3 / TEST-22.1.3
        let descriptors = property_fuzz_coverage_gate_descriptors();
        let optional = descriptors
            .iter()
            .find(|gate| gate.gate_id == "quality::fuzz::long-running-campaign")
            .expect("optional long-running fuzz gate is visible");
        let evidence = vec![
            QualityCommandEvidence::new(
                "quality::property::invariants",
                GateEvidenceStatus::Passed,
                "property suite passed",
            ),
            QualityCommandEvidence::new(
                "quality::fuzz::smoke-corpus",
                GateEvidenceStatus::Passed,
                "fuzz smoke corpus replay passed",
            ),
            QualityCommandEvidence::new(
                "quality::coverage::llvm-cov-summary",
                GateEvidenceStatus::Passed,
                "coverage summary captured",
            ),
        ];

        let blockers = required_quality_evidence_blockers(&descriptors, &evidence);

        assert_eq!(optional.mode, QualityGateMode::OptionalLongRunning);
        assert_eq!(
            optional.command,
            "cargo fuzz run ragas_evaluation -- -max_total_time=3600"
        );
        assert!(blockers.is_empty());
    }

    #[test]
    fn test_22_2_1_panic_safety_gates_declare_scope_command_and_failure_classes() {
        // SCEN-22.2.1 / AC1 / TEST-22.2.1
        let descriptors = panic_safety_gate_descriptors();

        assert!(descriptors.iter().all(|gate| !gate.gate_id.is_empty()));
        assert!(descriptors.iter().all(|gate| !gate.command.is_empty()));
        assert!(descriptors.iter().all(|gate| !gate.scope.is_empty()));
        assert!(descriptors.iter().all(|gate| !gate.failure_classes.is_empty()));
        assert!(descriptors.iter().any(|gate| {
            gate.gate_id == "quality::panic::unwind-boundaries"
                && gate.command == "cargo test panic_safety::"
                && gate.scope == "src/"
                && gate.failure_classes.contains(&SafetyFailureClass::UnwindBoundary)
                && gate.failure_classes.contains(&SafetyFailureClass::AsyncTaskPanic)
                && gate.mode == QualityGateMode::RequiredDefaultCi
        }));
    }

    #[test]
    fn test_22_2_2_mutation_gates_declare_tool_threshold_and_mode() {
        // SCEN-22.2.2 / AC2 / TEST-22.2.2
        let descriptors = mutation_gate_descriptors();

        assert!(descriptors.iter().all(|gate| !gate.gate_id.is_empty()));
        assert!(descriptors.iter().all(|gate| !gate.tool.is_empty()));
        assert!(descriptors.iter().all(|gate| !gate.command.is_empty()));
        assert!(descriptors.iter().all(|gate| gate.threshold_percent > 0));
        assert!(descriptors.iter().any(|gate| {
            gate.gate_id == "quality::mutation::release-threshold"
                && gate.tool == "cargo-mutants"
                && gate.command == "cargo mutants --minimum-test-timeout 60 --timeout 300"
                && gate.scope == "src/"
                && gate.threshold_percent == 80
                && gate.mode == QualityGateMode::RequiredReleaseEvidence
        }));
        assert!(descriptors.iter().any(|gate| {
            gate.gate_id == "quality::mutation::extended-campaign"
                && gate.tool == "cargo-mutants"
                && gate.mode == QualityGateMode::OptionalLongRunning
        }));
    }

    #[test]
    fn test_22_2_3_missing_required_panic_or_mutation_evidence_blocks_release() {
        // SCEN-22.2.3 / AC3 / TEST-22.2.3
        let descriptors = panic_mutation_quality_gate_descriptors();
        let evidence = vec![QualityCommandEvidence::new(
            "quality::panic::unwind-boundaries",
            GateEvidenceStatus::Passed,
            "panic safety tests passed",
        )];

        let blockers = required_quality_evidence_blockers(&descriptors, &evidence);

        assert!(blockers.iter().any(|finding| {
            finding.gate_id == "quality::mutation::release-threshold"
                && finding.evidence_kind == QualityEvidenceKind::Mutation
                && finding.command == "cargo mutants --minimum-test-timeout 60 --timeout 300"
                && finding.release_blocking
        }));
        assert!(
            !blockers
                .iter()
                .any(|finding| finding.gate_id == "quality::mutation::extended-campaign")
        );
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

    #[test]
    fn test_19_3_1_metric_release_blockers_aggregate_catalog_fixture_and_drift_failures() {
        // SCEN-19.3.1 / AC1 / TEST-19.3.1
        let catalog_claims = metric_catalog_parity_claims();
        let drift = MetricGoldenComparison {
            feature: "metric::faithfulness".to_string(),
            outcome: MetricGoldenOutcome::UndeclaredDrift,
            drift: Some("score baseline=0.8 rust=0.6".to_string()),
        };

        let blockers = metric_release_blockers(&catalog_claims, &[drift], &[]);
        let catalog_blockers = release_blocking_claims(&catalog_claims);

        assert!(
            catalog_blockers
                .iter()
                .any(|claim| claim.feature == "metric::summarization")
        );
        assert!(blockers.iter().any(|blocker| {
            blocker.source == MetricReleaseBlockerSource::Catalog
                && blocker.feature == "metric::summarization"
        }));
        assert!(blockers.iter().any(|blocker| {
            blocker.source == MetricReleaseBlockerSource::FixtureDrift
                && blocker.feature == "metric::faithfulness"
        }));
    }

    #[test]
    fn test_19_3_2_unclassified_metric_names_block_release_by_default() {
        // SCEN-19.3.2 / AC2 / TEST-19.3.2
        let blockers = metric_release_blockers(&[], &[], &["new_upstream_metric"]);

        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].source, MetricReleaseBlockerSource::Unclassified);
        assert_eq!(blockers[0].feature, "metric::new_upstream_metric");
        assert_eq!(blockers[0].status, ParityFeatureStatus::NotStarted);
    }

    #[test]
    fn test_19_3_3_metric_release_summary_exposes_count_and_features() {
        // SCEN-19.3.3 / AC3 / TEST-19.3.3
        let blockers = vec![
            MetricReleaseBlocker {
                feature: "metric::summarization".to_string(),
                source: MetricReleaseBlockerSource::Catalog,
                status: ParityFeatureStatus::KnownGap,
                detail: "missing fixture".to_string(),
            },
            MetricReleaseBlocker {
                feature: "metric::faithfulness".to_string(),
                source: MetricReleaseBlockerSource::FixtureDrift,
                status: ParityFeatureStatus::Blocked,
                detail: "undeclared drift".to_string(),
            },
        ];

        let summary = summarize_metric_release_blockers(&blockers);

        assert_eq!(summary.blocker_count, 2);
        assert!(!summary.release_ready);
        assert_eq!(
            summary.features,
            vec![
                "metric::summarization".to_string(),
                "metric::faithfulness".to_string()
            ]
        );
    }
}
