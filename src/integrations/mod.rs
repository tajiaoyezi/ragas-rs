use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::{ParityClaim, ParityFeatureStatus, RagasError, RuntimeEvent, RuntimeEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IntegrationDestination {
    Tracing,
    LangSmith,
    Langfuse,
    Opik,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IntegrationFamily {
    GenericTracing,
    LangChain,
    LangGraph,
    LangSmith,
    LlamaIndex,
    AgUi,
    Bedrock,
    Griptape,
    Helicone,
    Langfuse,
    Opik,
    R2R,
    Swarm,
}

impl IntegrationFamily {
    pub fn slug(self) -> &'static str {
        match self {
            Self::GenericTracing => "tracing",
            Self::LangChain => "langchain",
            Self::LangGraph => "langgraph",
            Self::LangSmith => "langsmith",
            Self::LlamaIndex => "llamaindex",
            Self::AgUi => "ag-ui",
            Self::Bedrock => "bedrock",
            Self::Griptape => "griptape",
            Self::Helicone => "helicone",
            Self::Langfuse => "langfuse",
            Self::Opik => "opik",
            Self::R2R => "r2r",
            Self::Swarm => "swarm",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrationTestMode {
    DeterministicContract,
    FeatureGatedLive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationDescriptor {
    pub family: IntegrationFamily,
    pub destination: Option<IntegrationDestination>,
    pub test_mode: IntegrationTestMode,
    pub requires_vendor_sdk: bool,
    pub parity_status: ParityFeatureStatus,
}

impl IntegrationDescriptor {
    pub fn new(
        family: IntegrationFamily,
        destination: Option<IntegrationDestination>,
        test_mode: IntegrationTestMode,
        requires_vendor_sdk: bool,
        parity_status: ParityFeatureStatus,
    ) -> Self {
        Self {
            family,
            destination,
            test_mode,
            requires_vendor_sdk,
            parity_status,
        }
    }

    pub fn parity_feature(&self) -> String {
        format!("integration::{}", self.family.slug())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrationBoundaryMode {
    CallbackAdapter,
    DelegatedFramework,
    ObservabilityExporter,
    EndpointStream,
    CloudService,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrationAuthMode {
    None,
    EnvVars,
    BearerToken,
    AwsSigV4,
    DelegatedSdk,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationContractDescriptor {
    pub family: IntegrationFamily,
    pub boundary_mode: IntegrationBoundaryMode,
    pub auth_mode: IntegrationAuthMode,
    pub auth_envs: Vec<&'static str>,
    pub target_operation: &'static str,
    pub lifecycle_fields: Vec<&'static str>,
    pub upstream_module_path: &'static str,
    pub fixture_path: &'static str,
    pub requires_vendor_sdk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationExportInput {
    pub event: RuntimeEvent,
    pub payload: IntegrationPayload,
    pub api_token: Option<String>,
}

impl IntegrationExportInput {
    pub fn new(event: RuntimeEvent, payload: IntegrationPayload) -> Self {
        Self {
            event,
            payload,
            api_token: None,
        }
    }

    pub fn with_api_token(mut self, api_token: impl Into<String>) -> Self {
        self.api_token = Some(api_token.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationExportPlan {
    pub family: IntegrationFamily,
    pub boundary_mode: IntegrationBoundaryMode,
    pub target_operation: String,
    pub headers: BTreeMap<String, String>,
    pub event: IntegrationEvent,
    pub safe_debug: String,
}

pub fn integration_contract_descriptors() -> Vec<IntegrationContractDescriptor> {
    Vec::new()
}

pub fn plan_integration_export(
    _family: IntegrationFamily,
    _input: IntegrationExportInput,
) -> Result<IntegrationExportPlan, RagasError> {
    Err(RagasError::DatasetIo {
        message: "integration export planning is not implemented".to_string(),
    })
}

#[derive(Debug, Clone)]
pub struct IntegrationRegistry {
    descriptors: Vec<IntegrationDescriptor>,
}

impl IntegrationRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: integration_descriptors(),
        }
    }

    pub fn descriptors(&self) -> &[IntegrationDescriptor] {
        &self.descriptors
    }
}

impl Default for IntegrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn integration_descriptors() -> Vec<IntegrationDescriptor> {
    vec![
        IntegrationDescriptor::new(
            IntegrationFamily::GenericTracing,
            Some(IntegrationDestination::Tracing),
            IntegrationTestMode::DeterministicContract,
            false,
            ParityFeatureStatus::Complete,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::LangChain,
            None,
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::KnownGap,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::LangGraph,
            None,
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::KnownGap,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::LangSmith,
            Some(IntegrationDestination::LangSmith),
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::Partial,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::LlamaIndex,
            None,
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::KnownGap,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::AgUi,
            None,
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::KnownGap,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::Bedrock,
            None,
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::KnownGap,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::Griptape,
            None,
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::KnownGap,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::Helicone,
            None,
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::KnownGap,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::Langfuse,
            Some(IntegrationDestination::Langfuse),
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::Partial,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::Opik,
            Some(IntegrationDestination::Opik),
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::Partial,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::R2R,
            None,
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::KnownGap,
        ),
        IntegrationDescriptor::new(
            IntegrationFamily::Swarm,
            None,
            IntegrationTestMode::FeatureGatedLive,
            true,
            ParityFeatureStatus::KnownGap,
        ),
    ]
}

pub fn integration_parity_claims() -> Vec<ParityClaim> {
    integration_descriptors()
        .into_iter()
        .filter(|descriptor| descriptor.parity_status != ParityFeatureStatus::Complete)
        .map(|descriptor| ParityClaim {
            feature: descriptor.parity_feature(),
            status: descriptor.parity_status,
            fixtures: Vec::new(),
        })
        .collect()
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationPayload {
    pub fields: BTreeMap<String, String>,
}

impl IntegrationPayload {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }
}

pub fn normalize_callback_payload(
    destination: IntegrationDestination,
    event: &RuntimeEvent,
    payload: IntegrationPayload,
) -> IntegrationEvent {
    IntegrationEvent {
        destination,
        kind: event.kind.clone(),
        run_id: event.run_id.clone(),
        metric_name: event.metric_name.clone(),
        sample_index: event.sample_index,
        payload: redact_payload(&payload),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationEvent {
    pub destination: IntegrationDestination,
    pub kind: RuntimeEventKind,
    pub run_id: String,
    pub metric_name: Option<String>,
    pub sample_index: Option<usize>,
    pub payload: IntegrationPayload,
}

#[derive(Debug, Clone)]
pub struct TracingIntegration {
    destination: IntegrationDestination,
    events: Arc<Mutex<Vec<IntegrationEvent>>>,
}

impl TracingIntegration {
    pub fn new(destination: IntegrationDestination) -> Self {
        Self {
            destination,
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn callback(&self) -> impl Fn(&RuntimeEvent) + Send + Sync + 'static {
        let destination = self.destination;
        let events = Arc::clone(&self.events);
        move |event| {
            events
                .lock()
                .expect("integration event lock")
                .push(normalize_callback_payload(
                    destination,
                    event,
                    IntegrationPayload::new(),
                ));
        }
    }

    pub fn export_payload(
        &self,
        kind: RuntimeEventKind,
        run_id: &str,
        payload: IntegrationPayload,
    ) {
        self.events
            .lock()
            .expect("integration event lock")
            .push(IntegrationEvent {
                destination: self.destination,
                kind,
                run_id: run_id.to_string(),
                metric_name: None,
                sample_index: None,
                payload: redact_payload(&payload),
            });
    }

    pub fn exported_events(&self) -> Vec<IntegrationEvent> {
        self.events.lock().expect("integration event lock").clone()
    }
}

#[derive(Debug, Clone)]
pub struct IntegrationFeatureRegistry {
    enabled: Vec<IntegrationDestination>,
}

impl Default for IntegrationFeatureRegistry {
    fn default() -> Self {
        Self {
            enabled: vec![IntegrationDestination::Tracing],
        }
    }
}

impl IntegrationFeatureRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_enabled(mut self, destination: IntegrationDestination) -> Self {
        if !self.enabled.contains(&destination) {
            self.enabled.push(destination);
        }
        self
    }

    pub fn is_enabled(&self, destination: IntegrationDestination) -> bool {
        self.enabled.contains(&destination)
    }

    pub fn require_enabled(&self, destination: IntegrationDestination) -> Result<(), RagasError> {
        if self.is_enabled(destination) {
            Ok(())
        } else {
            Err(RagasError::DatasetIo {
                message: format!("{destination:?} integration is feature-gated"),
            })
        }
    }
}

pub fn redact_payload(payload: &IntegrationPayload) -> IntegrationPayload {
    let mut redacted = IntegrationPayload::new();
    for (key, value) in &payload.fields {
        if is_sensitive_key(key) {
            redacted
                .fields
                .insert(key.clone(), "[REDACTED]".to_string());
        } else {
            redacted.fields.insert(key.clone(), value.clone());
        }
    }
    redacted
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("authorization")
        || key.contains("api_key")
        || key.contains("token")
        || key.contains("secret")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    use crate::{
        CallbackManager, ReleaseBlockerCategory, build_release_blocker_ledger,
        release_blocking_claims, validate_parity_claim,
    };

    #[test]
    fn test_14_2_1_tracing_integration_receives_callback_events() {
        // SCEN-14.2.1 / AC1 / TEST-14.2.1
        let tracing = TracingIntegration::new(IntegrationDestination::Tracing);
        let callbacks = CallbackManager::new().with_callback(tracing.callback());

        callbacks.emit(RuntimeEvent::evaluation_started("run-1"));
        callbacks.emit(RuntimeEvent::metric_started("run-1", "faithfulness", 2));

        let events = tracing.exported_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].destination, IntegrationDestination::Tracing);
        assert_eq!(events[0].kind, RuntimeEventKind::EvaluationStarted);
        assert_eq!(events[1].metric_name.as_deref(), Some("faithfulness"));
        assert_eq!(events[1].sample_index, Some(2));
    }

    #[test]
    fn test_14_2_2_external_integrations_are_feature_gated() {
        // SCEN-14.2.2 / AC2 / TEST-14.2.2
        let registry = IntegrationFeatureRegistry::new();

        assert!(registry.is_enabled(IntegrationDestination::Tracing));
        assert!(!registry.is_enabled(IntegrationDestination::LangSmith));
        assert!(
            registry
                .require_enabled(IntegrationDestination::LangSmith)
                .expect_err("langsmith disabled")
                .to_string()
                .contains("feature-gated")
        );

        let registry = registry.with_enabled(IntegrationDestination::LangSmith);
        assert!(registry.is_enabled(IntegrationDestination::LangSmith));
        assert!(
            registry
                .require_enabled(IntegrationDestination::LangSmith)
                .is_ok()
        );
    }

    #[test]
    fn test_14_2_3_payload_redaction_is_applied_before_export() {
        // SCEN-14.2.3 / AC3 / TEST-14.2.3
        let payload = IntegrationPayload::new()
            .with("authorization", "Bearer sk-secret")
            .with("api_key", "sk-live")
            .with("safe_field", "visible");

        let redacted = redact_payload(&payload);
        assert_eq!(
            redacted.fields.get("authorization").map(String::as_str),
            Some("[REDACTED]")
        );
        assert_eq!(
            redacted.fields.get("api_key").map(String::as_str),
            Some("[REDACTED]")
        );
        assert_eq!(
            redacted.fields.get("safe_field").map(String::as_str),
            Some("visible")
        );

        let tracing = TracingIntegration::new(IntegrationDestination::Tracing);
        tracing.export_payload(RuntimeEventKind::EvaluationStarted, "run-1", payload);
        let exported = tracing.exported_events();
        assert_eq!(
            exported[0]
                .payload
                .fields
                .get("authorization")
                .map(String::as_str),
            Some("[REDACTED]")
        );
    }

    #[test]
    fn test_18_4_1_integration_registry_lists_upstream_families_with_test_mode() {
        // SCEN-18.4.1 / AC1 / TEST-18.4.1
        let registry = IntegrationRegistry::new();
        let descriptors = registry.descriptors();
        let families: BTreeSet<_> = descriptors
            .iter()
            .map(|descriptor| descriptor.family)
            .collect();

        for expected in [
            IntegrationFamily::GenericTracing,
            IntegrationFamily::LangChain,
            IntegrationFamily::LangGraph,
            IntegrationFamily::LangSmith,
            IntegrationFamily::LlamaIndex,
            IntegrationFamily::AgUi,
            IntegrationFamily::Bedrock,
            IntegrationFamily::Griptape,
            IntegrationFamily::Helicone,
            IntegrationFamily::Langfuse,
            IntegrationFamily::Opik,
            IntegrationFamily::R2R,
            IntegrationFamily::Swarm,
        ] {
            assert!(families.contains(&expected), "missing {expected:?}");
        }

        let tracing = descriptors
            .iter()
            .find(|descriptor| descriptor.family == IntegrationFamily::GenericTracing)
            .expect("tracing descriptor");
        assert_eq!(
            tracing.test_mode,
            IntegrationTestMode::DeterministicContract
        );
        assert_eq!(tracing.parity_status, ParityFeatureStatus::Complete);

        let langsmith = descriptors
            .iter()
            .find(|descriptor| descriptor.family == IntegrationFamily::LangSmith)
            .expect("langsmith descriptor");
        assert_eq!(
            langsmith.destination,
            Some(IntegrationDestination::LangSmith)
        );
        assert_ne!(langsmith.parity_status, ParityFeatureStatus::Complete);
    }

    #[test]
    fn test_18_4_2_callback_payload_normalization_redacts_and_preserves_lifecycle_fields() {
        // SCEN-18.4.2 / AC2 / TEST-18.4.2
        let event = RuntimeEvent::metric_started("run-42", "faithfulness", 7);
        let payload = IntegrationPayload::new()
            .with("authorization", "Bearer sk-secret")
            .with("api_key", "sk-live")
            .with("prompt", "visible");

        let normalized =
            normalize_callback_payload(IntegrationDestination::Tracing, &event, payload);

        assert_eq!(normalized.destination, IntegrationDestination::Tracing);
        assert_eq!(normalized.kind, RuntimeEventKind::MetricStarted);
        assert_eq!(normalized.run_id, "run-42");
        assert_eq!(normalized.metric_name.as_deref(), Some("faithfulness"));
        assert_eq!(normalized.sample_index, Some(7));
        assert_eq!(
            normalized
                .payload
                .fields
                .get("authorization")
                .map(String::as_str),
            Some("[REDACTED]")
        );
        assert_eq!(
            normalized.payload.fields.get("api_key").map(String::as_str),
            Some("[REDACTED]")
        );
        assert_eq!(
            normalized.payload.fields.get("prompt").map(String::as_str),
            Some("visible")
        );
    }

    #[test]
    fn test_18_4_3_unsupported_integration_families_create_release_blocking_claims() {
        // SCEN-18.4.3 / AC3 / TEST-18.4.3
        let claims = integration_parity_claims();
        let blockers = release_blocking_claims(&claims);
        let blocking_features: BTreeSet<_> = blockers
            .iter()
            .map(|claim| claim.feature.as_str())
            .collect();

        for expected in [
            "integration::langchain",
            "integration::langgraph",
            "integration::llamaindex",
            "integration::bedrock",
            "integration::swarm",
        ] {
            assert!(
                blocking_features.contains(expected),
                "missing release blocker {expected}"
            );
        }

        assert!(claims.iter().all(|claim| {
            !(blocking_features.contains(claim.feature.as_str())
                && claim.status == ParityFeatureStatus::Complete)
        }));
    }

    #[test]
    fn test_27_1_1_integration_contract_descriptors_cover_tracked_upstream_families() {
        // SCEN-27.1.1 / AC1 / TEST-27.1.1
        let descriptors = integration_contract_descriptors();
        let families: BTreeSet<_> = descriptors
            .iter()
            .map(|descriptor| descriptor.family)
            .collect();

        for expected in [
            IntegrationFamily::LangChain,
            IntegrationFamily::LangGraph,
            IntegrationFamily::LangSmith,
            IntegrationFamily::LlamaIndex,
            IntegrationFamily::AgUi,
            IntegrationFamily::Bedrock,
            IntegrationFamily::Griptape,
            IntegrationFamily::Helicone,
            IntegrationFamily::Langfuse,
            IntegrationFamily::Opik,
            IntegrationFamily::R2R,
            IntegrationFamily::Swarm,
        ] {
            assert!(families.contains(&expected), "missing {expected:?}");
        }

        assert!(descriptors.iter().all(|descriptor| {
            descriptor
                .upstream_module_path
                .starts_with("src/ragas/integrations/")
                && descriptor
                    .fixture_path
                    .starts_with("tests/parity/fixtures/integration_")
                && !descriptor.target_operation.is_empty()
                && descriptor.lifecycle_fields.contains(&"run_id")
                && descriptor.lifecycle_fields.contains(&"event_kind")
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.family == IntegrationFamily::LangSmith
                && descriptor.auth_envs.contains(&"LANGCHAIN_API_KEY")
                && descriptor.boundary_mode == IntegrationBoundaryMode::ObservabilityExporter
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.family == IntegrationFamily::AgUi
                && descriptor.boundary_mode == IntegrationBoundaryMode::EndpointStream
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.family == IntegrationFamily::Bedrock
                && descriptor.auth_mode == IntegrationAuthMode::AwsSigV4
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.family == IntegrationFamily::LangChain
                && descriptor.boundary_mode == IntegrationBoundaryMode::DelegatedFramework
        }));
    }

    #[test]
    fn test_27_1_2_integration_export_plans_preserve_lifecycle_and_redact_auth() {
        // SCEN-27.1.2 / AC2 / TEST-27.1.2
        let event = RuntimeEvent::metric_started("run-27", "faithfulness", 3);
        let payload = IntegrationPayload::new()
            .with("authorization", "Bearer secret-token")
            .with("score", "0.91")
            .with("trace_id", "trace-123");

        let langsmith = plan_integration_export(
            IntegrationFamily::LangSmith,
            IntegrationExportInput::new(event.clone(), payload.clone())
                .with_api_token("langsmith-secret"),
        )
        .expect("langsmith export plan");

        assert_eq!(langsmith.family, IntegrationFamily::LangSmith);
        assert_eq!(
            langsmith.boundary_mode,
            IntegrationBoundaryMode::ObservabilityExporter
        );
        assert_eq!(langsmith.headers["Authorization"], "Bearer <redacted>");
        assert_eq!(langsmith.event.run_id, "run-27");
        assert_eq!(langsmith.event.metric_name.as_deref(), Some("faithfulness"));
        assert_eq!(langsmith.event.sample_index, Some(3));
        assert_eq!(
            langsmith
                .event
                .payload
                .fields
                .get("authorization")
                .map(String::as_str),
            Some("[REDACTED]")
        );
        assert_eq!(
            langsmith
                .event
                .payload
                .fields
                .get("score")
                .map(String::as_str),
            Some("0.91")
        );
        assert!(!langsmith.safe_debug.contains("langsmith-secret"));

        let ag_ui = plan_integration_export(
            IntegrationFamily::AgUi,
            IntegrationExportInput::new(event.clone(), payload.clone()),
        )
        .expect("ag-ui export plan");
        assert_eq!(ag_ui.boundary_mode, IntegrationBoundaryMode::EndpointStream);
        assert!(ag_ui.target_operation.contains("stream"));
        assert_eq!(
            ag_ui
                .event
                .payload
                .fields
                .get("trace_id")
                .map(String::as_str),
            Some("trace-123")
        );

        let bedrock = plan_integration_export(
            IntegrationFamily::Bedrock,
            IntegrationExportInput::new(event, payload).with_api_token("aws-secret"),
        )
        .expect("bedrock export plan");
        assert_eq!(
            bedrock.headers["Authorization"],
            "AWS4-HMAC-SHA256 <redacted>"
        );
        assert!(!bedrock.safe_debug.contains("aws-secret"));
    }

    #[test]
    fn test_27_1_3_integration_claims_are_fixture_backed_and_not_release_blocking() {
        // SCEN-27.1.3 / AC3 / TEST-27.1.3
        let claims = integration_parity_claims();
        let features: BTreeSet<_> = claims.iter().map(|claim| claim.feature.as_str()).collect();

        for expected in [
            "integration::langchain",
            "integration::langgraph",
            "integration::langsmith",
            "integration::llamaindex",
            "integration::ag-ui",
            "integration::bedrock",
            "integration::griptape",
            "integration::helicone",
            "integration::langfuse",
            "integration::opik",
            "integration::r2r",
            "integration::swarm",
        ] {
            assert!(features.contains(expected), "missing {expected}");
        }

        assert!(claims.iter().all(|claim| {
            claim.status == ParityFeatureStatus::Complete
                && !claim.fixtures.is_empty()
                && validate_parity_claim(claim).is_ok()
        }));
        assert!(
            release_blocking_claims(&claims).is_empty(),
            "integration claims should not block release"
        );

        let ledger = build_release_blocker_ledger();
        assert!(
            !ledger
                .entries
                .iter()
                .any(|entry| entry.category == ReleaseBlockerCategory::Integration),
            "integration category should be absent from release blockers"
        );
    }
}
