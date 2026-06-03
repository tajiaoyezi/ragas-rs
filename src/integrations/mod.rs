use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::{RagasError, RuntimeEvent, RuntimeEventKind};

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
    vec![
        integration_contract(
            IntegrationFamily::LangChain,
            IntegrationBoundaryMode::DelegatedFramework,
            IntegrationAuthMode::DelegatedSdk,
            &[],
            "EvaluatorChain.invoke",
            true,
        ),
        integration_contract(
            IntegrationFamily::LangGraph,
            IntegrationBoundaryMode::DelegatedFramework,
            IntegrationAuthMode::DelegatedSdk,
            &[],
            "LangGraph workflow adapter",
            true,
        ),
        integration_contract(
            IntegrationFamily::LangSmith,
            IntegrationBoundaryMode::ObservabilityExporter,
            IntegrationAuthMode::BearerToken,
            &["LANGCHAIN_API_KEY", "LANGCHAIN_ENDPOINT"],
            "Client.run_on_dataset",
            true,
        ),
        integration_contract(
            IntegrationFamily::LlamaIndex,
            IntegrationBoundaryMode::DelegatedFramework,
            IntegrationAuthMode::DelegatedSdk,
            &[],
            "LlamaIndex evaluator adapter",
            true,
        ),
        integration_contract(
            IntegrationFamily::AgUi,
            IntegrationBoundaryMode::EndpointStream,
            IntegrationAuthMode::BearerToken,
            &["AG_UI_API_KEY"],
            "call_ag_ui_endpoint stream events",
            true,
        ),
        integration_contract(
            IntegrationFamily::Bedrock,
            IntegrationBoundaryMode::CloudService,
            IntegrationAuthMode::AwsSigV4,
            &["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY", "AWS_REGION"],
            "Bedrock evaluation job",
            true,
        ),
        integration_contract(
            IntegrationFamily::Griptape,
            IntegrationBoundaryMode::DelegatedFramework,
            IntegrationAuthMode::DelegatedSdk,
            &[],
            "Griptape task adapter",
            true,
        ),
        integration_contract(
            IntegrationFamily::Helicone,
            IntegrationBoundaryMode::ObservabilityExporter,
            IntegrationAuthMode::BearerToken,
            &["HELICONE_API_KEY"],
            "Helicone trace export",
            true,
        ),
        integration_contract(
            IntegrationFamily::Langfuse,
            IntegrationBoundaryMode::ObservabilityExporter,
            IntegrationAuthMode::EnvVars,
            &[
                "LANGFUSE_PUBLIC_KEY",
                "LANGFUSE_SECRET_KEY",
                "LANGFUSE_HOST",
            ],
            "LangfuseTrace sync_trace",
            true,
        ),
        integration_contract(
            IntegrationFamily::Opik,
            IntegrationBoundaryMode::ObservabilityExporter,
            IntegrationAuthMode::EnvVars,
            &["OPIK_API_KEY", "OPIK_WORKSPACE"],
            "OpikTracer log feedback scores",
            true,
        ),
        integration_contract(
            IntegrationFamily::R2R,
            IntegrationBoundaryMode::EndpointStream,
            IntegrationAuthMode::BearerToken,
            &["R2R_API_KEY", "R2R_BASE_URL"],
            "R2R app endpoint",
            true,
        ),
        integration_contract(
            IntegrationFamily::Swarm,
            IntegrationBoundaryMode::DelegatedFramework,
            IntegrationAuthMode::DelegatedSdk,
            &[],
            "Swarm agent adapter",
            true,
        ),
    ]
}

pub fn plan_integration_export(
    family: IntegrationFamily,
    input: IntegrationExportInput,
) -> Result<IntegrationExportPlan, RagasError> {
    let descriptor = integration_contract_descriptors()
        .into_iter()
        .find(|descriptor| descriptor.family == family)
        .ok_or_else(|| RagasError::DatasetIo {
            message: format!("missing integration contract descriptor for {family:?}"),
        })?;
    let headers = integration_export_headers(&descriptor);
    let event = normalize_callback_payload(
        integration_destination_for_family(family),
        &input.event,
        input.payload,
    );
    let safe_debug = format!(
        "integration={} boundary={:?} target={}",
        family.slug(),
        descriptor.boundary_mode,
        descriptor.target_operation
    );

    Ok(IntegrationExportPlan {
        family,
        boundary_mode: descriptor.boundary_mode,
        target_operation: descriptor.target_operation.to_string(),
        headers,
        event,
        safe_debug,
    })
}

fn integration_contract(
    family: IntegrationFamily,
    boundary_mode: IntegrationBoundaryMode,
    auth_mode: IntegrationAuthMode,
    auth_envs: &[&'static str],
    target_operation: &'static str,
    requires_vendor_sdk: bool,
) -> IntegrationContractDescriptor {
    IntegrationContractDescriptor {
        family,
        boundary_mode,
        auth_mode,
        auth_envs: auth_envs.to_vec(),
        target_operation,
        lifecycle_fields: vec![
            "event_kind",
            "run_id",
            "metric_name",
            "sample_index",
            "payload",
        ],
        requires_vendor_sdk,
    }
}

fn integration_export_headers(
    descriptor: &IntegrationContractDescriptor,
) -> BTreeMap<String, String> {
    let mut headers = BTreeMap::new();
    match descriptor.auth_mode {
        IntegrationAuthMode::None => {}
        IntegrationAuthMode::EnvVars => {
            headers.insert(
                "x-ragas-auth-envs".to_string(),
                descriptor.auth_envs.join(","),
            );
        }
        IntegrationAuthMode::BearerToken => {
            headers.insert("Authorization".to_string(), "Bearer <redacted>".to_string());
        }
        IntegrationAuthMode::AwsSigV4 => {
            headers.insert(
                "Authorization".to_string(),
                "AWS4-HMAC-SHA256 <redacted>".to_string(),
            );
        }
        IntegrationAuthMode::DelegatedSdk => {
            headers.insert(
                "x-ragas-integration-boundary".to_string(),
                "delegated-sdk".to_string(),
            );
        }
    }
    headers
}

fn integration_destination_for_family(family: IntegrationFamily) -> IntegrationDestination {
    match family {
        IntegrationFamily::LangSmith => IntegrationDestination::LangSmith,
        IntegrationFamily::Langfuse => IntegrationDestination::Langfuse,
        IntegrationFamily::Opik => IntegrationDestination::Opik,
        _ => IntegrationDestination::Tracing,
    }
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

    use crate::CallbackManager;

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
            !descriptor.target_operation.is_empty()
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
}
