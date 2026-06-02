use std::collections::BTreeMap;

use crate::parity::{
    MetricGoldenComparison, MetricGoldenOutcome, ParityClaim, ParityFeatureStatus,
    release_blocking_claims, validate_parity_claim,
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
    Platform,
    E2E,
    BugLedgerAudit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QualityEvidenceKind {
    Property,
    Fuzz,
    Coverage,
    PanicSafety,
    Mutation,
    Platform,
    E2E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SafetyFailureClass {
    DirectPanic,
    AsyncTaskPanic,
    UnwindBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlatformTarget {
    LinuxX64,
    MacOsArm64,
    WindowsX64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum E2eWorkflow {
    Evaluate,
    ProviderMock,
    DatasetIo,
    Cli,
    DocsExamples,
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
pub struct PlatformEvidenceDescriptor {
    pub gate_id: &'static str,
    pub target: PlatformTarget,
    pub command: &'static str,
    pub scope: &'static str,
    pub mode: QualityGateMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct E2eWorkflowDescriptor {
    pub gate_id: &'static str,
    pub workflow: E2eWorkflow,
    pub command: &'static str,
    pub scope: &'static str,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReleaseBlockerCategory {
    Provider,
    Backend,
    Integration,
    Metric,
    Testset,
    Optimizer,
    Docs,
    Quality,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseBlockerEntry {
    pub id: String,
    pub category: ReleaseBlockerCategory,
    pub feature: String,
    pub severity: BugSeverity,
    pub source: String,
    pub impact: String,
    pub waived: bool,
}

impl ReleaseBlockerEntry {
    pub fn new(
        id: impl Into<String>,
        category: ReleaseBlockerCategory,
        feature: impl Into<String>,
        severity: BugSeverity,
        source: impl Into<String>,
        impact: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            feature: feature.into(),
            severity,
            source: source.into(),
            impact: impact.into(),
            waived: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseBlockerLedger {
    pub entries: Vec<ReleaseBlockerEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseBlockerSummary {
    pub total: usize,
    pub non_waived: usize,
    pub by_category: BTreeMap<ReleaseBlockerCategory, usize>,
    pub release_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GapResolutionKind {
    Fixed,
    Waived,
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FinalAuditEvidenceKind {
    Build,
    Check,
    Unit,
    Parity,
    Examples,
    Quality,
    BlockerLedger,
    BugLedger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalAuditEvidence {
    pub kind: FinalAuditEvidenceKind,
    pub status: GateEvidenceStatus,
    pub detail: String,
}

impl FinalAuditEvidence {
    pub fn new(
        kind: FinalAuditEvidenceKind,
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
pub struct FinalBugZeroAudit {
    pub missing_evidence: Vec<FinalAuditEvidenceKind>,
    pub failed_evidence: Vec<FinalAuditEvidenceKind>,
    pub blocker_summary: GapResolutionSummary,
    pub unresolved_release_blocking_bugs: usize,
    pub release_ready: bool,
    pub statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseWaiver {
    pub blocker_id: String,
    pub scope: String,
    pub rationale: String,
    pub owner: String,
    pub expires_on: String,
    pub risk: String,
    pub rollback_impact: String,
}

impl ReleaseWaiver {
    pub fn new(
        blocker_id: impl Into<String>,
        scope: impl Into<String>,
        rationale: impl Into<String>,
        owner: impl Into<String>,
        expires_on: impl Into<String>,
        risk: impl Into<String>,
        rollback_impact: impl Into<String>,
    ) -> Self {
        Self {
            blocker_id: blocker_id.into(),
            scope: scope.into(),
            rationale: rationale.into(),
            owner: owner.into(),
            expires_on: expires_on.into(),
            risk: risk.into(),
            rollback_impact: rollback_impact.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaiverValidationError {
    MissingField(&'static str),
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapResolutionRecord {
    pub blocker_id: String,
    pub kind: GapResolutionKind,
    pub evidence: String,
    pub waiver: Option<ReleaseWaiver>,
}

impl GapResolutionRecord {
    pub fn fixed(blocker_id: impl Into<String>, evidence: impl Into<String>) -> Self {
        Self {
            blocker_id: blocker_id.into(),
            kind: GapResolutionKind::Fixed,
            evidence: evidence.into(),
            waiver: None,
        }
    }

    pub fn waived(waiver: ReleaseWaiver) -> Self {
        Self {
            blocker_id: waiver.blocker_id.clone(),
            kind: GapResolutionKind::Waived,
            evidence: waiver.rationale.clone(),
            waiver: Some(waiver),
        }
    }

    pub fn deferred(blocker_id: impl Into<String>, evidence: impl Into<String>) -> Self {
        Self {
            blocker_id: blocker_id.into(),
            kind: GapResolutionKind::Deferred,
            evidence: evidence.into(),
            waiver: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapResolutionSummary {
    pub fixed: usize,
    pub waived: usize,
    pub deferred: usize,
    pub still_blocking: usize,
    pub release_ready: bool,
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

pub fn build_release_blocker_ledger() -> ReleaseBlockerLedger {
    let mut entries = Vec::new();

    append_parity_blockers(
        &mut entries,
        ReleaseBlockerCategory::Provider,
        "providers::provider_parity_claims",
        crate::providers::provider_parity_claims(),
    );
    append_parity_blockers(
        &mut entries,
        ReleaseBlockerCategory::Backend,
        "backends::backend_parity_claims",
        crate::backends::backend_parity_claims(),
    );
    append_parity_blockers(
        &mut entries,
        ReleaseBlockerCategory::Integration,
        "integrations::integration_parity_claims",
        crate::integrations::integration_parity_claims(),
    );
    append_parity_blockers(
        &mut entries,
        ReleaseBlockerCategory::Metric,
        "metrics::metric_catalog_parity_claims",
        crate::metrics::metric_catalog_parity_claims(),
    );

    let mut testset_claims = crate::testset::graph_parity_claims();
    testset_claims.extend(crate::testset::transform_parity_claims());
    testset_claims.extend(crate::testset::synthesizer_parity_claims());
    append_parity_blockers(
        &mut entries,
        ReleaseBlockerCategory::Testset,
        "testset::*_parity_claims",
        testset_claims,
    );

    append_parity_blockers(
        &mut entries,
        ReleaseBlockerCategory::Optimizer,
        "optimizers::optimizer_parity_claims",
        crate::optimizers::optimizer_parity_claims(),
    );
    append_parity_blockers(
        &mut entries,
        ReleaseBlockerCategory::Docs,
        "docs_examples::docs_parity_claims",
        crate::docs_examples::docs_parity_claims(),
    );

    let mut quality_descriptors = property_fuzz_coverage_gate_descriptors();
    quality_descriptors.extend(panic_mutation_quality_gate_descriptors());
    quality_descriptors.extend(platform_e2e_quality_gate_descriptors());
    entries.extend(
        required_quality_evidence_blockers(&quality_descriptors, &[])
            .into_iter()
            .map(|finding| ReleaseBlockerEntry {
                id: format!("RB-quality-{}", sanitize_id_component(&finding.gate_id)),
                category: ReleaseBlockerCategory::Quality,
                feature: finding.gate_id,
                severity: BugSeverity::High,
                source: "release::required_quality_evidence_blockers".to_string(),
                impact: format!(
                    "{}; command `{}` must produce release evidence",
                    finding.detail, finding.command
                ),
                waived: false,
            }),
    );

    ReleaseBlockerLedger { entries }
}

pub fn summarize_release_blocker_ledger(ledger: &ReleaseBlockerLedger) -> ReleaseBlockerSummary {
    let mut by_category = BTreeMap::new();
    for entry in &ledger.entries {
        *by_category.entry(entry.category).or_insert(0) += 1;
    }
    let non_waived = ledger.entries.iter().filter(|entry| !entry.waived).count();

    ReleaseBlockerSummary {
        total: ledger.entries.len(),
        non_waived,
        by_category,
        release_ready: non_waived == 0,
    }
}

fn append_parity_blockers(
    entries: &mut Vec<ReleaseBlockerEntry>,
    category: ReleaseBlockerCategory,
    source: &'static str,
    claims: Vec<ParityClaim>,
) {
    entries.extend(
        release_blocking_claims(&claims)
            .into_iter()
            .map(|claim| ReleaseBlockerEntry {
                id: format!(
                    "RB-{}-{}",
                    category_slug(category),
                    sanitize_id_component(&claim.feature)
                ),
                category,
                feature: claim.feature.clone(),
                severity: severity_for_parity_status(claim.status),
                source: source.to_string(),
                impact: format!(
                    "Blocks current-upstream parity because {} is {:?}",
                    claim.feature, claim.status
                ),
                waived: false,
            }),
    );
}

fn severity_for_parity_status(status: ParityFeatureStatus) -> BugSeverity {
    match status {
        ParityFeatureStatus::Partial => BugSeverity::Medium,
        ParityFeatureStatus::Complete
        | ParityFeatureStatus::KnownGap
        | ParityFeatureStatus::NotStarted
        | ParityFeatureStatus::Blocked => BugSeverity::High,
    }
}

fn category_slug(category: ReleaseBlockerCategory) -> &'static str {
    match category {
        ReleaseBlockerCategory::Provider => "provider",
        ReleaseBlockerCategory::Backend => "backend",
        ReleaseBlockerCategory::Integration => "integration",
        ReleaseBlockerCategory::Metric => "metric",
        ReleaseBlockerCategory::Testset => "testset",
        ReleaseBlockerCategory::Optimizer => "optimizer",
        ReleaseBlockerCategory::Docs => "docs",
        ReleaseBlockerCategory::Quality => "quality",
    }
}

fn sanitize_id_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn validate_release_waiver(
    waiver: &ReleaseWaiver,
    as_of: &str,
) -> Result<(), WaiverValidationError> {
    required_waiver_field(&waiver.blocker_id, "blocker_id")?;
    required_waiver_field(&waiver.scope, "scope")?;
    required_waiver_field(&waiver.rationale, "rationale")?;
    required_waiver_field(&waiver.owner, "owner")?;
    required_waiver_field(&waiver.expires_on, "expiry")?;
    required_waiver_field(&waiver.risk, "risk")?;
    required_waiver_field(&waiver.rollback_impact, "rollback_impact")?;
    if waiver.expires_on.as_str() < as_of {
        return Err(WaiverValidationError::Expired);
    }
    Ok(())
}

pub fn summarize_gap_resolutions(
    ledger: &ReleaseBlockerLedger,
    resolutions: &[GapResolutionRecord],
    as_of: &str,
) -> GapResolutionSummary {
    let mut summary = GapResolutionSummary {
        fixed: 0,
        waived: 0,
        deferred: 0,
        still_blocking: 0,
        release_ready: false,
    };

    for entry in &ledger.entries {
        let resolution = resolutions
            .iter()
            .find(|resolution| resolution.blocker_id == entry.id);

        match resolution {
            Some(resolution)
                if resolution.kind == GapResolutionKind::Fixed
                    && !resolution.evidence.trim().is_empty() =>
            {
                summary.fixed += 1;
            }
            Some(resolution) if resolution.kind == GapResolutionKind::Waived => {
                if let Some(waiver) = &resolution.waiver {
                    if validate_release_waiver(waiver, as_of).is_ok() {
                        summary.waived += 1;
                    } else {
                        summary.still_blocking += 1;
                    }
                } else {
                    summary.still_blocking += 1;
                }
            }
            Some(resolution) if resolution.kind == GapResolutionKind::Deferred => {
                summary.deferred += 1;
                summary.still_blocking += 1;
            }
            _ => summary.still_blocking += 1,
        }
    }

    summary.release_ready = summary.still_blocking == 0;
    summary
}

pub fn required_final_audit_evidence() -> Vec<FinalAuditEvidenceKind> {
    vec![
        FinalAuditEvidenceKind::Build,
        FinalAuditEvidenceKind::Check,
        FinalAuditEvidenceKind::Unit,
        FinalAuditEvidenceKind::Parity,
        FinalAuditEvidenceKind::Examples,
        FinalAuditEvidenceKind::Quality,
        FinalAuditEvidenceKind::BlockerLedger,
        FinalAuditEvidenceKind::BugLedger,
    ]
}

pub fn evaluate_final_bug_zero_audit(
    evidence: &[FinalAuditEvidence],
    ledger: &ReleaseBlockerLedger,
    resolutions: &[GapResolutionRecord],
    bugs: &[BugLedgerEntry],
    as_of: &str,
) -> FinalBugZeroAudit {
    let mut missing_evidence = Vec::new();
    let mut failed_evidence = Vec::new();

    for required in required_final_audit_evidence() {
        match evidence.iter().find(|entry| entry.kind == required) {
            Some(entry) => match entry.status {
                GateEvidenceStatus::Passed => {}
                GateEvidenceStatus::Failed => failed_evidence.push(required),
                GateEvidenceStatus::Missing | GateEvidenceStatus::SkippedWithJustification => {
                    missing_evidence.push(required)
                }
            },
            None => missing_evidence.push(required),
        }
    }

    let blocker_summary = summarize_gap_resolutions(ledger, resolutions, as_of);
    let unresolved_release_blocking_bugs = release_blocking_bugs(bugs).len();
    let release_ready = missing_evidence.is_empty()
        && failed_evidence.is_empty()
        && blocker_summary.release_ready
        && unresolved_release_blocking_bugs == 0;
    let statement = if release_ready {
        "Within the verified scope, no known unresolved release-blocking bugs or unwaived blockers remain.".to_string()
    } else {
        format!(
            "Release refused within the verified scope: known unresolved evidence gaps={}, failed evidence={}, blockers={}, high-severity bugs={} remain.",
            missing_evidence.len(),
            failed_evidence.len(),
            blocker_summary.still_blocking,
            unresolved_release_blocking_bugs
        )
    };

    FinalBugZeroAudit {
        missing_evidence,
        failed_evidence,
        blocker_summary,
        unresolved_release_blocking_bugs,
        release_ready,
        statement,
    }
}

pub fn render_final_bug_zero_audit(audit: &FinalBugZeroAudit) -> String {
    audit.statement.clone()
}

fn required_waiver_field(value: &str, field: &'static str) -> Result<(), WaiverValidationError> {
    if value.trim().is_empty() {
        Err(WaiverValidationError::MissingField(field))
    } else {
        Ok(())
    }
}

pub fn release_gate_files() -> Vec<&'static str> {
    vec![
        "Cargo.toml",
        ".github/workflows/ci.yml",
        "docs/release-checklist.md",
    ]
}

pub fn required_quality_evidence_descriptors() -> Vec<QualityGateDescriptor> {
    Vec::new()
}

pub fn release_quality_evidence() -> Vec<QualityCommandEvidence> {
    Vec::new()
}

pub fn final_release_audit_evidence() -> Vec<FinalAuditEvidence> {
    Vec::new()
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
        QualityGateKind::Platform,
        QualityGateKind::E2E,
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

pub fn platform_evidence_matrix() -> Vec<PlatformEvidenceDescriptor> {
    vec![
        PlatformEvidenceDescriptor {
            gate_id: "quality::platform::linux-x64",
            target: PlatformTarget::LinuxX64,
            command: "ci:linux-x64:cargo build && cargo test",
            scope: "src/",
            mode: QualityGateMode::RequiredReleaseEvidence,
        },
        PlatformEvidenceDescriptor {
            gate_id: "quality::platform::macos-arm64",
            target: PlatformTarget::MacOsArm64,
            command: "ci:macos-arm64:cargo build && cargo test",
            scope: "src/",
            mode: QualityGateMode::RequiredReleaseEvidence,
        },
        PlatformEvidenceDescriptor {
            gate_id: "quality::platform::windows-x64",
            target: PlatformTarget::WindowsX64,
            command: "ci:windows-x64:cargo build && cargo test",
            scope: "src/",
            mode: QualityGateMode::RequiredReleaseEvidence,
        },
    ]
}

pub fn e2e_workflow_matrix() -> Vec<E2eWorkflowDescriptor> {
    vec![
        E2eWorkflowDescriptor {
            gate_id: "quality::e2e::evaluate",
            workflow: E2eWorkflow::Evaluate,
            command: "cargo test eval::",
            scope: "src/eval.rs",
            mode: QualityGateMode::RequiredReleaseEvidence,
        },
        E2eWorkflowDescriptor {
            gate_id: "quality::e2e::provider-mock",
            workflow: E2eWorkflow::ProviderMock,
            command: "cargo test providers::",
            scope: "src/providers.rs",
            mode: QualityGateMode::RequiredReleaseEvidence,
        },
        E2eWorkflowDescriptor {
            gate_id: "quality::e2e::dataset-io",
            workflow: E2eWorkflow::DatasetIo,
            command: "cargo test dataset::tests::test_5_2",
            scope: "src/dataset.rs",
            mode: QualityGateMode::RequiredReleaseEvidence,
        },
        E2eWorkflowDescriptor {
            gate_id: "quality::e2e::cli",
            workflow: E2eWorkflow::Cli,
            command: "cargo test cli::",
            scope: "src/cli/",
            mode: QualityGateMode::RequiredReleaseEvidence,
        },
        E2eWorkflowDescriptor {
            gate_id: "quality::e2e::docs-examples",
            workflow: E2eWorkflow::DocsExamples,
            command: "cargo build --examples",
            scope: "examples/",
            mode: QualityGateMode::RequiredReleaseEvidence,
        },
    ]
}

pub fn platform_e2e_quality_gate_descriptors() -> Vec<QualityGateDescriptor> {
    platform_evidence_matrix()
        .into_iter()
        .map(|descriptor| QualityGateDescriptor {
            gate_id: descriptor.gate_id,
            evidence_kind: QualityEvidenceKind::Platform,
            gate_kind: QualityGateKind::Platform,
            command: descriptor.command,
            scope: descriptor.scope,
            mode: descriptor.mode,
        })
        .chain(
            e2e_workflow_matrix()
                .into_iter()
                .map(|descriptor| QualityGateDescriptor {
                    gate_id: descriptor.gate_id,
                    evidence_kind: QualityEvidenceKind::E2E,
                    gate_kind: QualityGateKind::E2E,
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
        MetricGoldenComparison, MetricGoldenOutcome, backend_parity_claims, docs_parity_claims,
        integration_parity_claims, metric_catalog_parity_claims, provider_parity_claims,
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
            QualityGateKind::Platform,
            QualityGateKind::E2E,
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
        assert!(
            descriptors
                .iter()
                .all(|gate| !gate.failure_classes.is_empty())
        );
        assert!(descriptors.iter().any(|gate| {
            gate.gate_id == "quality::panic::unwind-boundaries"
                && gate.command == "cargo test panic_safety::"
                && gate.scope == "src/"
                && gate
                    .failure_classes
                    .contains(&SafetyFailureClass::UnwindBoundary)
                && gate
                    .failure_classes
                    .contains(&SafetyFailureClass::AsyncTaskPanic)
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
    fn test_22_3_1_platform_matrix_covers_supported_targets() {
        // SCEN-22.3.1 / AC1 / TEST-22.3.1
        let matrix = platform_evidence_matrix();
        let targets: BTreeSet<_> = matrix.iter().map(|entry| entry.target).collect();

        assert!(targets.contains(&PlatformTarget::LinuxX64));
        assert!(targets.contains(&PlatformTarget::MacOsArm64));
        assert!(targets.contains(&PlatformTarget::WindowsX64));
        assert!(matrix.iter().all(|entry| !entry.gate_id.is_empty()));
        assert!(matrix.iter().all(|entry| !entry.command.is_empty()));
        assert!(matrix.iter().all(|entry| !entry.scope.is_empty()));
        assert!(
            matrix
                .iter()
                .all(|entry| entry.mode == QualityGateMode::RequiredReleaseEvidence)
        );
    }

    #[test]
    fn test_22_3_2_e2e_matrix_covers_critical_flows() {
        // SCEN-22.3.2 / AC2 / TEST-22.3.2
        let matrix = e2e_workflow_matrix();
        let workflows: BTreeSet<_> = matrix.iter().map(|entry| entry.workflow).collect();

        for workflow in [
            E2eWorkflow::Evaluate,
            E2eWorkflow::ProviderMock,
            E2eWorkflow::DatasetIo,
            E2eWorkflow::Cli,
            E2eWorkflow::DocsExamples,
        ] {
            assert!(workflows.contains(&workflow), "missing {workflow:?}");
        }
        assert!(matrix.iter().all(|entry| !entry.gate_id.is_empty()));
        assert!(matrix.iter().all(|entry| !entry.command.is_empty()));
        assert!(matrix.iter().all(|entry| !entry.scope.is_empty()));
    }

    #[test]
    fn test_22_3_3_missing_platform_or_e2e_evidence_blocks_release() {
        // SCEN-22.3.3 / AC3 / TEST-22.3.3
        let descriptors = platform_e2e_quality_gate_descriptors();
        let evidence = vec![
            QualityCommandEvidence::new(
                "quality::platform::windows-x64",
                GateEvidenceStatus::Passed,
                "Windows local verification passed",
            ),
            QualityCommandEvidence::new(
                "quality::e2e::evaluate",
                GateEvidenceStatus::Passed,
                "evaluate workflow passed",
            ),
        ];

        let blockers = required_quality_evidence_blockers(&descriptors, &evidence);

        assert!(blockers.iter().any(|finding| {
            finding.gate_id == "quality::platform::linux-x64"
                && finding.evidence_kind == QualityEvidenceKind::Platform
                && finding.release_blocking
        }));
        assert!(blockers.iter().any(|finding| {
            finding.gate_id == "quality::e2e::cli"
                && finding.evidence_kind == QualityEvidenceKind::E2E
                && finding.command == "cargo test cli::"
                && finding.release_blocking
        }));
    }

    #[test]
    fn test_23_1_1_ledger_aggregates_release_blocker_sources() {
        // SCEN-23.1.1 / AC1 / TEST-23.1.1
        let ledger = build_release_blocker_ledger();
        let categories: BTreeSet<_> = ledger.entries.iter().map(|entry| entry.category).collect();

        assert!(categories.contains(&ReleaseBlockerCategory::Quality));
        assert!(
            !categories.contains(&ReleaseBlockerCategory::Metric),
            "closed metric claims should not keep Metric in blocker ledger"
        );
        assert!(
            !categories.contains(&ReleaseBlockerCategory::Testset),
            "closed testset claims should not keep Testset in blocker ledger"
        );
        assert!(
            !categories.contains(&ReleaseBlockerCategory::Optimizer),
            "closed optimizer claims should not keep Optimizer in blocker ledger"
        );

        let docs_claims = docs_parity_claims();
        assert!(
            docs_claims
                .iter()
                .any(|claim| claim.feature == "docs::quickstart::experiments"
                    && claim.status == ParityFeatureStatus::Complete)
        );
        assert!(
            release_blocking_claims(&docs_claims).is_empty(),
            "closed docs quickstarts should not keep Docs in blocker ledger"
        );

        let backend_claims = backend_parity_claims();
        assert!(
            release_blocking_claims(&backend_claims).is_empty(),
            "closed backend claims should not keep Backend in blocker ledger"
        );

        let provider_claims = provider_parity_claims();
        assert!(
            release_blocking_claims(&provider_claims).is_empty(),
            "closed provider claims should not keep Provider in blocker ledger"
        );

        let integration_claims = integration_parity_claims();
        assert!(
            release_blocking_claims(&integration_claims).is_empty(),
            "closed integration claims should not keep Integration in blocker ledger"
        );
    }

    #[test]
    fn test_23_1_2_blockers_have_release_metadata() {
        // SCEN-23.1.2 / AC2 / TEST-23.1.2
        let blocker = ReleaseBlockerEntry::new(
            "RB-provider-litellm",
            ReleaseBlockerCategory::Provider,
            "provider::litellm",
            BugSeverity::High,
            "providers::provider_parity_claims",
            "Blocks current-upstream provider parity",
        );

        assert_eq!(blocker.category, ReleaseBlockerCategory::Provider);
        assert_eq!(blocker.feature, "provider::litellm");
        assert_eq!(blocker.severity, BugSeverity::High);
        assert_eq!(blocker.source, "providers::provider_parity_claims");
        assert!(blocker.impact.contains("provider parity"));
        assert!(!blocker.id.is_empty());
        assert!(!blocker.waived);
    }

    #[test]
    fn test_23_1_3_non_waived_blocker_prevents_release() {
        // SCEN-23.1.3 / AC3 / TEST-23.1.3
        let ledger = ReleaseBlockerLedger {
            entries: vec![ReleaseBlockerEntry::new(
                "RB-quality-linux",
                ReleaseBlockerCategory::Quality,
                "quality::platform::linux-x64",
                BugSeverity::High,
                "release::required_quality_evidence_blockers",
                "Missing Linux x64 release evidence",
            )],
        };

        let summary = summarize_release_blocker_ledger(&ledger);

        assert_eq!(summary.total, 1);
        assert_eq!(summary.non_waived, 1);
        assert_eq!(summary.by_category[&ReleaseBlockerCategory::Quality], 1);
        assert!(!summary.release_ready);
    }

    #[test]
    fn test_23_2_1_waiver_requires_audit_fields() {
        // SCEN-23.2.1 / AC1 / TEST-23.2.1
        let waiver = ReleaseWaiver::new(
            "RB-docs-template",
            "docs::quickstart::agent",
            "Template is excluded from Rust-native v1 until upstream stabilizes",
            "release-owner",
            "2026-12-31",
            "Users cannot follow the agent quickstart from Rust docs",
            "Keep release blocked if Python parity is required",
        );

        validate_release_waiver(&waiver, "2026-06-02").expect("complete waiver validates");
        assert_eq!(waiver.scope, "docs::quickstart::agent");
        assert!(waiver.rationale.contains("excluded"));
        assert_eq!(waiver.owner, "release-owner");
        assert_eq!(waiver.expires_on, "2026-12-31");
        assert!(waiver.risk.contains("quickstart"));
        assert!(waiver.rollback_impact.contains("release blocked"));

        let incomplete = ReleaseWaiver::new(
            "RB-docs-template",
            "",
            "missing scope",
            "release-owner",
            "2026-12-31",
            "risk",
            "rollback",
        );
        assert_eq!(
            validate_release_waiver(&incomplete, "2026-06-02"),
            Err(WaiverValidationError::MissingField("scope"))
        );
    }

    #[test]
    fn test_23_2_2_expired_or_incomplete_waiver_does_not_unblock_release() {
        // SCEN-23.2.2 / AC2 / TEST-23.2.2
        let ledger = ReleaseBlockerLedger {
            entries: vec![ReleaseBlockerEntry::new(
                "RB-quality-linux",
                ReleaseBlockerCategory::Quality,
                "quality::platform::linux-x64",
                BugSeverity::High,
                "release::required_quality_evidence_blockers",
                "Missing Linux x64 release evidence",
            )],
        };
        let expired = ReleaseWaiver::new(
            "RB-quality-linux",
            "quality::platform::linux-x64",
            "Temporarily accept missing Linux evidence",
            "release-owner",
            "2026-01-01",
            "Linux-specific failures could ship",
            "Re-run Linux CI before release",
        );

        assert_eq!(
            validate_release_waiver(&expired, "2026-06-02"),
            Err(WaiverValidationError::Expired)
        );
        let summary = summarize_gap_resolutions(
            &ledger,
            &[GapResolutionRecord::waived(expired)],
            "2026-06-02",
        );

        assert_eq!(summary.waived, 0);
        assert_eq!(summary.still_blocking, 1);
        assert!(!summary.release_ready);
    }

    #[test]
    fn test_23_2_3_release_summary_separates_fixed_waived_and_blocking_gaps() {
        // SCEN-23.2.3 / AC3 / TEST-23.2.3
        let ledger = ReleaseBlockerLedger {
            entries: vec![
                ReleaseBlockerEntry::new(
                    "RB-provider",
                    ReleaseBlockerCategory::Provider,
                    "provider::litellm",
                    BugSeverity::High,
                    "providers::provider_parity_claims",
                    "provider gap",
                ),
                ReleaseBlockerEntry::new(
                    "RB-docs",
                    ReleaseBlockerCategory::Docs,
                    "docs::quickstart::agent",
                    BugSeverity::Medium,
                    "docs_examples::docs_parity_claims",
                    "docs gap",
                ),
                ReleaseBlockerEntry::new(
                    "RB-quality",
                    ReleaseBlockerCategory::Quality,
                    "quality::coverage::llvm-cov-summary",
                    BugSeverity::High,
                    "release::required_quality_evidence_blockers",
                    "coverage gap",
                ),
            ],
        };
        let valid_waiver = ReleaseWaiver::new(
            "RB-docs",
            "docs::quickstart::agent",
            "Documented exclusion for Rust-native v1",
            "release-owner",
            "2026-12-31",
            "Docs workflow remains unavailable",
            "Keep docs gap visible in release notes",
        );
        let resolutions = vec![
            GapResolutionRecord::fixed("RB-provider", "LiteLLM descriptor implemented"),
            GapResolutionRecord::waived(valid_waiver),
        ];

        let summary = summarize_gap_resolutions(&ledger, &resolutions, "2026-06-02");

        assert_eq!(summary.fixed, 1);
        assert_eq!(summary.waived, 1);
        assert_eq!(summary.deferred, 0);
        assert_eq!(summary.still_blocking, 1);
        assert!(!summary.release_ready);
    }

    #[test]
    fn test_23_3_1_final_audit_requires_all_evidence() {
        // SCEN-23.3.1 / AC1 / TEST-23.3.1
        let required: BTreeSet<_> = required_final_audit_evidence().into_iter().collect();

        for kind in [
            FinalAuditEvidenceKind::Build,
            FinalAuditEvidenceKind::Check,
            FinalAuditEvidenceKind::Unit,
            FinalAuditEvidenceKind::Parity,
            FinalAuditEvidenceKind::Examples,
            FinalAuditEvidenceKind::Quality,
            FinalAuditEvidenceKind::BlockerLedger,
            FinalAuditEvidenceKind::BugLedger,
        ] {
            assert!(required.contains(&kind), "missing final evidence {kind:?}");
        }

        let checklist =
            std::fs::read_to_string("docs/release-checklist.md").expect("release checklist");
        assert!(checklist.contains("Final audit evidence"));
        assert!(checklist.contains("blocker ledger"));
        assert!(checklist.contains("bug ledger"));
        assert!(checklist.contains("quality gate evidence"));
    }

    #[test]
    fn test_23_3_2_final_audit_refuses_unresolved_blockers_or_high_bugs() {
        // SCEN-23.3.2 / AC2 / TEST-23.3.2
        let evidence = required_final_audit_evidence()
            .into_iter()
            .map(|kind| FinalAuditEvidence::new(kind, GateEvidenceStatus::Passed, "passed"))
            .collect::<Vec<_>>();
        let ledger = ReleaseBlockerLedger {
            entries: vec![ReleaseBlockerEntry::new(
                "RB-quality",
                ReleaseBlockerCategory::Quality,
                "quality::coverage::llvm-cov-summary",
                BugSeverity::High,
                "release::required_quality_evidence_blockers",
                "coverage evidence missing",
            )],
        };
        let bugs = vec![BugLedgerEntry::new(
            "BUG-PARITY",
            BugSeverity::High,
            BugStatus::Open,
            BugClass::Parity,
            "metric::faithfulness",
            "fixture drift remains unresolved",
            "TEST-23.3.2",
        )];

        let audit = evaluate_final_bug_zero_audit(&evidence, &ledger, &[], &bugs, "2026-06-02");

        assert_eq!(audit.blocker_summary.still_blocking, 1);
        assert_eq!(audit.unresolved_release_blocking_bugs, 1);
        assert!(!audit.release_ready);
    }

    #[test]
    fn test_23_3_3_final_wording_states_scope_without_absolute_bug_free_claim() {
        // SCEN-23.3.3 / AC3 / TEST-23.3.3
        let audit = FinalBugZeroAudit {
            missing_evidence: vec![FinalAuditEvidenceKind::Quality],
            failed_evidence: Vec::new(),
            blocker_summary: GapResolutionSummary {
                fixed: 0,
                waived: 0,
                deferred: 0,
                still_blocking: 1,
                release_ready: false,
            },
            unresolved_release_blocking_bugs: 0,
            release_ready: false,
            statement:
                "Release refused within the verified scope: known unresolved blockers remain."
                    .to_string(),
        };

        let rendered = render_final_bug_zero_audit(&audit);
        let lower = rendered.to_ascii_lowercase();

        assert!(rendered.contains("verified scope"));
        assert!(rendered.contains("known unresolved"));
        assert!(!lower.contains("bug-free"));
        assert!(!lower.contains("zero potential bugs"));
        assert!(!lower.contains("no bugs"));
    }

    #[test]
    fn test_31_1_1_release_quality_evidence_covers_required_gate_ids() {
        // SCEN-31.1.1 / AC1 / TEST-31.1.1
        let descriptors = required_quality_evidence_descriptors();
        let required_ids = descriptors
            .iter()
            .filter(|descriptor| descriptor.mode.is_required())
            .map(|descriptor| descriptor.gate_id)
            .collect::<BTreeSet<_>>();
        let evidence = release_quality_evidence();
        let evidence_ids = evidence
            .iter()
            .map(|record| record.gate_id.as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(required_ids.len(), 13);
        assert!(required_ids.contains("quality::coverage::llvm-cov-summary"));
        assert!(required_ids.contains("quality::mutation::release-threshold"));
        assert!(!required_ids.contains("quality::fuzz::long-running-campaign"));
        assert_eq!(evidence_ids, required_ids);
        assert!(evidence.iter().all(|record| {
            record.status == GateEvidenceStatus::Passed && !record.detail.trim().is_empty()
        }));
        assert!(required_quality_evidence_blockers(&descriptors, &evidence).is_empty());
    }

    #[test]
    fn test_31_1_2_release_blocker_ledger_is_empty_after_quality_evidence() {
        // SCEN-31.1.2 / AC2 / TEST-31.1.2
        let ledger = build_release_blocker_ledger();
        let summary = summarize_release_blocker_ledger(&ledger);

        assert!(ledger.entries.is_empty(), "remaining entries: {:?}", ledger.entries);
        assert_eq!(summary.total, 0);
        assert_eq!(summary.non_waived, 0);
        assert!(summary.by_category.is_empty());
        assert!(summary.release_ready);
    }

    #[test]
    fn test_31_1_3_final_audit_ready_with_scoped_wording() {
        // SCEN-31.1.3 / AC3 / TEST-31.1.3
        let evidence = final_release_audit_evidence();
        let ledger = build_release_blocker_ledger();
        let audit = evaluate_final_bug_zero_audit(&evidence, &ledger, &[], &[], "2026-06-02");

        assert!(audit.release_ready);
        assert!(audit.missing_evidence.is_empty());
        assert!(audit.failed_evidence.is_empty());
        assert_eq!(audit.unresolved_release_blocking_bugs, 0);
        assert_eq!(audit.blocker_summary.still_blocking, 0);

        let rendered = render_final_bug_zero_audit(&audit);
        let lower = rendered.to_ascii_lowercase();
        assert!(rendered.contains("verified scope"));
        assert!(rendered.contains("no known unresolved release-blocking bugs"));
        assert!(!lower.contains("bug-free"));
        assert!(!lower.contains("zero potential bugs"));
        assert!(!lower.contains("no bugs"));
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
        let mut catalog_claims = metric_catalog_parity_claims();
        catalog_claims.push(ParityClaim {
            feature: "metric::synthetic_missing_fixture".to_string(),
            status: ParityFeatureStatus::Partial,
            fixtures: Vec::new(),
        });
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
                .any(|claim| claim.feature == "metric::synthetic_missing_fixture")
        );
        assert!(blockers.iter().any(|blocker| {
            blocker.source == MetricReleaseBlockerSource::Catalog
                && blocker.feature == "metric::synthetic_missing_fixture"
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
