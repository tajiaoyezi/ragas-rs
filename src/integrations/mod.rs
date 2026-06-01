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

    use crate::{CallbackManager, release_blocking_claims};

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
}
