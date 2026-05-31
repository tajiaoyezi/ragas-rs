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
        |_event| {}
    }

    pub fn export_payload(
        &self,
        _kind: RuntimeEventKind,
        _run_id: &str,
        _payload: IntegrationPayload,
    ) {
    }

    pub fn exported_events(&self) -> Vec<IntegrationEvent> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct IntegrationFeatureRegistry {
    enabled: Vec<IntegrationDestination>,
}

impl IntegrationFeatureRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_enabled(mut self, destination: IntegrationDestination) -> Self {
        self.enabled.push(destination);
        self
    }

    pub fn is_enabled(&self, _destination: IntegrationDestination) -> bool {
        false
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
    payload.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
