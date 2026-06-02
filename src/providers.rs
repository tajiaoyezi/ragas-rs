use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use async_trait::async_trait;

use crate::{
    EmbeddingProvider, EmbeddingRequest, EmbeddingResponse, LlmProvider, LlmRequest, LlmResponse,
    ParityClaim, ParityFeatureStatus, RagasError, TokenUsage, UsageTracker,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProviderFamily {
    OpenAiCompatible,
    AzureOpenAi,
    LiteLlm,
    Instructor,
    Haystack,
    HuggingFace,
    Google,
    OciGenAi,
    Mock,
}

impl ProviderFamily {
    pub fn slug(self) -> &'static str {
        match self {
            Self::OpenAiCompatible => "openai-compatible",
            Self::AzureOpenAi => "azure-openai",
            Self::LiteLlm => "litellm",
            Self::Instructor => "instructor",
            Self::Haystack => "haystack",
            Self::HuggingFace => "huggingface",
            Self::Google => "google",
            Self::OciGenAi => "oci-genai",
            Self::Mock => "mock",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderMode {
    Deterministic,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Llm,
    Embedding,
    StructuredLlm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderDescriptor {
    pub family: ProviderFamily,
    pub kind: ProviderKind,
    pub mode: ProviderMode,
    pub supports_system_prompt: bool,
    pub supports_structured_output: bool,
    pub parity_status: ParityFeatureStatus,
}

impl ProviderDescriptor {
    pub fn new(
        family: ProviderFamily,
        kind: ProviderKind,
        mode: ProviderMode,
        supports_system_prompt: bool,
        supports_structured_output: bool,
        parity_status: ParityFeatureStatus,
    ) -> Self {
        Self {
            family,
            kind,
            mode,
            supports_system_prompt,
            supports_structured_output,
            parity_status,
        }
    }

    pub fn parity_feature(&self) -> String {
        format!("provider::{}", self.family.slug())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredLlmDescriptor {
    pub family: ProviderFamily,
    pub mode: ProviderMode,
    pub supports_system_prompt: bool,
    pub supports_structured_output: bool,
    pub parity_status: ParityFeatureStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderProtocolMode {
    DirectHttp,
    DelegatedWrapper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderAuthScheme {
    BearerApiKey,
    HeaderApiKey,
    QueryApiKey,
    DelegatedWrapper,
    OciConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderProtocolDescriptor {
    pub family: ProviderFamily,
    pub kind: ProviderKind,
    pub protocol_mode: ProviderProtocolMode,
    pub auth_scheme: ProviderAuthScheme,
    pub auth_env: Option<&'static str>,
    pub endpoint_template: &'static str,
    pub request_path: &'static str,
    pub supports_system_prompt: bool,
    pub supports_structured_output: bool,
    pub upstream_module_path: &'static str,
    pub fixture_path: &'static str,
    pub response_text_path: &'static str,
    pub usage_path: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderProtocolInput {
    pub model: String,
    pub messages: Vec<crate::ChatMessage>,
    pub embedding_input: Vec<String>,
    pub system_prompt: Option<String>,
    pub response_schema: Option<String>,
    pub api_key: Option<String>,
}

impl ProviderProtocolInput {
    pub fn llm(model: impl Into<String>, messages: Vec<crate::ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            embedding_input: Vec::new(),
            system_prompt: None,
            response_schema: None,
            api_key: None,
        }
    }

    pub fn embedding(model: impl Into<String>, input: Vec<String>) -> Self {
        Self {
            model: model.into(),
            messages: Vec::new(),
            embedding_input: input,
            system_prompt: None,
            response_schema: None,
            api_key: None,
        }
    }

    pub fn with_system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    pub fn with_response_schema(mut self, response_schema: impl Into<String>) -> Self {
        self.response_schema = Some(response_schema.into());
        self
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderRequestPlan {
    pub family: ProviderFamily,
    pub kind: ProviderKind,
    pub protocol_mode: ProviderProtocolMode,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: serde_json::Value,
    pub response_text_path: String,
    pub usage_path: Option<String>,
    pub safe_debug: String,
}

pub fn provider_protocol_descriptors() -> Vec<ProviderProtocolDescriptor> {
    Vec::new()
}

pub fn plan_provider_request(
    _family: ProviderFamily,
    _kind: ProviderKind,
    _input: ProviderProtocolInput,
) -> Result<ProviderRequestPlan, RagasError> {
    Err(RagasError::Provider {
        message: "provider protocol planning is not implemented".to_string(),
    })
}

pub fn upstream_provider_descriptors() -> Vec<ProviderDescriptor> {
    vec![
        ProviderDescriptor::new(
            ProviderFamily::OpenAiCompatible,
            ProviderKind::Llm,
            ProviderMode::Live,
            true,
            false,
            ParityFeatureStatus::Partial,
        ),
        ProviderDescriptor::new(
            ProviderFamily::OpenAiCompatible,
            ProviderKind::Embedding,
            ProviderMode::Live,
            false,
            false,
            ParityFeatureStatus::Partial,
        ),
        ProviderDescriptor::new(
            ProviderFamily::AzureOpenAi,
            ProviderKind::Llm,
            ProviderMode::Live,
            true,
            false,
            ParityFeatureStatus::Partial,
        ),
        ProviderDescriptor::new(
            ProviderFamily::AzureOpenAi,
            ProviderKind::Embedding,
            ProviderMode::Live,
            false,
            false,
            ParityFeatureStatus::Partial,
        ),
        ProviderDescriptor::new(
            ProviderFamily::LiteLlm,
            ProviderKind::Llm,
            ProviderMode::Live,
            true,
            false,
            ParityFeatureStatus::Partial,
        ),
        ProviderDescriptor::new(
            ProviderFamily::LiteLlm,
            ProviderKind::StructuredLlm,
            ProviderMode::Live,
            true,
            true,
            ParityFeatureStatus::Partial,
        ),
        ProviderDescriptor::new(
            ProviderFamily::Instructor,
            ProviderKind::StructuredLlm,
            ProviderMode::Live,
            true,
            true,
            ParityFeatureStatus::Partial,
        ),
        ProviderDescriptor::new(
            ProviderFamily::Haystack,
            ProviderKind::Llm,
            ProviderMode::Live,
            true,
            false,
            ParityFeatureStatus::KnownGap,
        ),
        ProviderDescriptor::new(
            ProviderFamily::Haystack,
            ProviderKind::Embedding,
            ProviderMode::Live,
            false,
            false,
            ParityFeatureStatus::KnownGap,
        ),
        ProviderDescriptor::new(
            ProviderFamily::HuggingFace,
            ProviderKind::Llm,
            ProviderMode::Live,
            false,
            false,
            ParityFeatureStatus::KnownGap,
        ),
        ProviderDescriptor::new(
            ProviderFamily::HuggingFace,
            ProviderKind::Embedding,
            ProviderMode::Live,
            false,
            false,
            ParityFeatureStatus::KnownGap,
        ),
        ProviderDescriptor::new(
            ProviderFamily::Google,
            ProviderKind::Llm,
            ProviderMode::Live,
            true,
            false,
            ParityFeatureStatus::KnownGap,
        ),
        ProviderDescriptor::new(
            ProviderFamily::Google,
            ProviderKind::Embedding,
            ProviderMode::Live,
            false,
            false,
            ParityFeatureStatus::KnownGap,
        ),
        ProviderDescriptor::new(
            ProviderFamily::OciGenAi,
            ProviderKind::Llm,
            ProviderMode::Live,
            true,
            false,
            ParityFeatureStatus::KnownGap,
        ),
        ProviderDescriptor::new(
            ProviderFamily::Mock,
            ProviderKind::Llm,
            ProviderMode::Deterministic,
            true,
            false,
            ParityFeatureStatus::Complete,
        ),
        ProviderDescriptor::new(
            ProviderFamily::Mock,
            ProviderKind::Embedding,
            ProviderMode::Deterministic,
            false,
            false,
            ParityFeatureStatus::Complete,
        ),
    ]
}

pub fn structured_llm_descriptors() -> Vec<StructuredLlmDescriptor> {
    upstream_provider_descriptors()
        .into_iter()
        .filter(|descriptor| descriptor.kind == ProviderKind::StructuredLlm)
        .map(|descriptor| StructuredLlmDescriptor {
            family: descriptor.family,
            mode: descriptor.mode,
            supports_system_prompt: descriptor.supports_system_prompt,
            supports_structured_output: descriptor.supports_structured_output,
            parity_status: descriptor.parity_status,
        })
        .collect()
}

pub fn provider_parity_claims() -> Vec<ParityClaim> {
    let mut status_by_feature = BTreeMap::new();
    for descriptor in upstream_provider_descriptors()
        .into_iter()
        .filter(|descriptor| descriptor.mode == ProviderMode::Live)
    {
        let status = descriptor.parity_status;
        status_by_feature
            .entry(descriptor.parity_feature())
            .and_modify(|existing| {
                if status > *existing {
                    *existing = status;
                }
            })
            .or_insert(status);
    }

    status_by_feature
        .into_iter()
        .filter(|(_, status)| *status != ParityFeatureStatus::Complete)
        .map(|(feature, status)| ParityClaim {
            feature,
            status,
            fixtures: Vec::new(),
        })
        .collect()
}

#[derive(Clone)]
pub struct ProviderRegistry {
    llms: HashMap<String, Arc<dyn LlmProvider>>,
    embeddings: HashMap<String, Arc<dyn EmbeddingProvider>>,
    descriptors: Vec<ProviderDescriptor>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            llms: HashMap::new(),
            embeddings: HashMap::new(),
            descriptors: upstream_provider_descriptors(),
        }
    }

    pub fn register_llm(mut self, name: impl Into<String>, provider: Arc<dyn LlmProvider>) -> Self {
        self.llms.insert(name.into(), provider);
        self
    }

    pub fn register_embedding(
        mut self,
        name: impl Into<String>,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        self.embeddings.insert(name.into(), provider);
        self
    }

    pub fn with_provider_descriptor(mut self, descriptor: ProviderDescriptor) -> Self {
        self.descriptors.push(descriptor);
        self
    }

    pub fn provider_descriptors(&self) -> &[ProviderDescriptor] {
        &self.descriptors
    }

    pub fn llm(&self, name: &str) -> Result<Arc<dyn LlmProvider>, RagasError> {
        self.llms
            .get(name)
            .cloned()
            .ok_or_else(|| RagasError::Provider {
                message: format!("missing provider: llm '{name}'"),
            })
    }

    pub fn embedding(&self, name: &str) -> Result<Arc<dyn EmbeddingProvider>, RagasError> {
        self.embeddings
            .get(name)
            .cloned()
            .ok_or_else(|| RagasError::Provider {
                message: format!("missing provider: embedding '{name}'"),
            })
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MockLlmProvider {
    response: LlmResponse,
}

impl MockLlmProvider {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            response: LlmResponse {
                content: content.into(),
                usage: None,
            },
        }
    }

    pub fn with_usage(mut self, usage: TokenUsage) -> Self {
        self.response.usage = Some(usage);
        self
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn generate(&self, _request: LlmRequest) -> Result<LlmResponse, RagasError> {
        Ok(self.response.clone())
    }
}

#[derive(Debug, Clone)]
pub struct MockEmbeddingProvider {
    embeddings: Vec<Vec<f32>>,
    usage: Option<TokenUsage>,
}

impl MockEmbeddingProvider {
    pub fn new(embeddings: Vec<Vec<f32>>) -> Self {
        Self {
            embeddings,
            usage: None,
        }
    }

    pub fn with_usage(mut self, usage: TokenUsage) -> Self {
        self.usage = Some(usage);
        self
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed(&self, _request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
        Ok(EmbeddingResponse {
            embeddings: self.embeddings.clone(),
            usage: self.usage.clone(),
        })
    }
}

pub fn record_provider_usage(
    tracker: &mut UsageTracker,
    provider_name: impl Into<String>,
    metric_name: impl Into<String>,
    usage: Option<&TokenUsage>,
) {
    if let Some(usage) = usage {
        tracker.record(provider_name, metric_name, usage.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    use crate::{
        ChatMessage, ReleaseBlockerCategory, build_release_blocker_ledger, release_blocking_claims,
        validate_parity_claim,
    };
    use serde_json::json;

    #[test]
    fn test_7_1_1_provider_registry_resolves_llm_and_embedding_by_name() {
        // SCEN-7.1.1 / AC1 / TEST-7.1.1
        let registry = ProviderRegistry::new()
            .register_llm("mock-llm", Arc::new(MockLlmProvider::new("ok")))
            .register_embedding(
                "mock-embedding",
                Arc::new(MockEmbeddingProvider::new(vec![vec![1.0, 0.0]])),
            );

        assert!(registry.llm("mock-llm").is_ok());
        assert!(registry.embedding("mock-embedding").is_ok());

        let error = match registry.llm("missing") {
            Ok(_) => panic!("missing provider unexpectedly resolved"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("missing provider"));
        assert!(error.to_string().contains("missing"));
    }

    #[tokio::test]
    async fn test_7_1_2_mock_providers_support_deterministic_unit_tests() {
        // SCEN-7.1.2 / AC2 / TEST-7.1.2
        let llm = MockLlmProvider::new("fixed response");
        let first = llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user("q1")],
                temperature: Some(0.0),
            })
            .await
            .expect("llm response");
        let second = llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user("q2")],
                temperature: Some(1.0),
            })
            .await
            .expect("llm response");

        assert_eq!(first, second);

        let embedding = MockEmbeddingProvider::new(vec![vec![0.1, 0.2], vec![0.3, 0.4]]);
        let response = embedding
            .embed(EmbeddingRequest {
                input: vec!["a".to_string(), "b".to_string()],
            })
            .await
            .expect("embedding response");

        assert_eq!(response.embeddings, vec![vec![0.1, 0.2], vec![0.3, 0.4]]);
    }

    #[tokio::test]
    async fn test_7_1_3_provider_responses_carry_usage_accounting_when_available() {
        // SCEN-7.1.3 / AC3 / TEST-7.1.3
        let usage = TokenUsage {
            prompt_tokens: Some(12),
            completion_tokens: Some(4),
            total_tokens: Some(16),
        };
        let llm = MockLlmProvider::new("scored").with_usage(usage.clone());
        let response = llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user("score")],
                temperature: Some(0.0),
            })
            .await
            .expect("llm response");

        assert_eq!(response.usage.as_ref(), Some(&usage));

        let mut tracker = UsageTracker::new();
        record_provider_usage(
            &mut tracker,
            "mock-llm",
            "faithfulness",
            response.usage.as_ref(),
        );
        let summary = tracker.summary();

        assert_eq!(summary.by_provider["mock-llm"].total_tokens, 16);
        assert_eq!(summary.by_metric["faithfulness"].prompt_tokens, 12);
    }

    #[test]
    fn test_18_2_1_provider_descriptors_classify_upstream_families_and_modes() {
        // SCEN-18.2.1 / AC1 / TEST-18.2.1
        let registry = ProviderRegistry::new();
        let descriptors = registry.provider_descriptors();
        let families: BTreeSet<_> = descriptors
            .iter()
            .map(|descriptor| descriptor.family)
            .collect();

        for expected in [
            ProviderFamily::OpenAiCompatible,
            ProviderFamily::AzureOpenAi,
            ProviderFamily::LiteLlm,
            ProviderFamily::Instructor,
            ProviderFamily::Haystack,
            ProviderFamily::HuggingFace,
            ProviderFamily::Google,
            ProviderFamily::OciGenAi,
            ProviderFamily::Mock,
        ] {
            assert!(families.contains(&expected), "missing {expected:?}");
        }

        assert!(descriptors.iter().any(|descriptor| {
            descriptor.kind == ProviderKind::Llm && descriptor.mode == ProviderMode::Live
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.kind == ProviderKind::Embedding && descriptor.mode == ProviderMode::Live
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.family == ProviderFamily::Mock
                && descriptor.mode == ProviderMode::Deterministic
        }));
    }

    #[test]
    fn test_18_2_2_structured_llm_descriptors_record_system_prompt_support() {
        // SCEN-18.2.2 / AC2 / TEST-18.2.2
        let structured = structured_llm_descriptors();

        for family in [ProviderFamily::Instructor, ProviderFamily::LiteLlm] {
            let descriptor = structured
                .iter()
                .find(|descriptor| descriptor.family == family)
                .unwrap_or_else(|| panic!("missing structured descriptor for {family:?}"));
            assert_eq!(descriptor.mode, ProviderMode::Live);
            assert!(descriptor.supports_system_prompt);
            assert!(descriptor.supports_structured_output);
        }
    }

    #[test]
    fn test_18_2_3_unsupported_live_provider_families_create_release_blocking_claims() {
        // SCEN-18.2.3 / AC3 / TEST-18.2.3
        let claims = provider_parity_claims();
        let blockers = release_blocking_claims(&claims);
        let blocking_features: BTreeSet<_> = blockers
            .iter()
            .map(|claim| claim.feature.as_str())
            .collect();

        for expected in [
            "provider::google",
            "provider::haystack",
            "provider::huggingface",
            "provider::oci-genai",
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
    fn test_26_1_1_provider_protocol_descriptors_cover_tracked_upstream_families() {
        // SCEN-26.1.1 / AC1 / TEST-26.1.1
        let descriptors = provider_protocol_descriptors();
        let families: BTreeSet<_> = descriptors
            .iter()
            .map(|descriptor| descriptor.family)
            .collect();

        for expected in [
            ProviderFamily::OpenAiCompatible,
            ProviderFamily::AzureOpenAi,
            ProviderFamily::LiteLlm,
            ProviderFamily::Instructor,
            ProviderFamily::Haystack,
            ProviderFamily::HuggingFace,
            ProviderFamily::Google,
            ProviderFamily::OciGenAi,
        ] {
            assert!(families.contains(&expected), "missing {expected:?}");
        }

        assert!(descriptors.iter().all(|descriptor| {
            !descriptor.endpoint_template.is_empty()
                && !descriptor.request_path.is_empty()
                && descriptor.upstream_module_path.starts_with("src/ragas/")
                && descriptor
                    .fixture_path
                    .starts_with("tests/parity/fixtures/provider_")
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.family == ProviderFamily::Google
                && descriptor.kind == ProviderKind::Embedding
                && descriptor.auth_env == Some("GOOGLE_API_KEY")
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.family == ProviderFamily::AzureOpenAi
                && descriptor.auth_scheme == ProviderAuthScheme::HeaderApiKey
                && descriptor.auth_env == Some("AZURE_OPENAI_API_KEY")
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.family == ProviderFamily::Instructor
                && descriptor.kind == ProviderKind::StructuredLlm
                && descriptor.supports_structured_output
                && descriptor.supports_system_prompt
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.family == ProviderFamily::Haystack
                && descriptor.protocol_mode == ProviderProtocolMode::DelegatedWrapper
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.family == ProviderFamily::OciGenAi
                && descriptor.auth_scheme == ProviderAuthScheme::OciConfig
        }));
    }

    #[test]
    fn test_26_1_2_provider_request_plans_preserve_payloads_and_redact_auth() {
        // SCEN-26.1.2 / AC2 / TEST-26.1.2
        let chat = plan_provider_request(
            ProviderFamily::OpenAiCompatible,
            ProviderKind::Llm,
            ProviderProtocolInput::llm("gpt-4o-mini", vec![ChatMessage::user("score this answer")])
                .with_system_prompt("judge strictly")
                .with_api_key("sk-secret-should-not-leak"),
        )
        .expect("openai-compatible chat plan");

        assert!(chat.url.ends_with("/chat/completions"));
        assert_eq!(chat.headers["Authorization"], "Bearer <redacted>");
        assert_eq!(chat.body["model"], json!("gpt-4o-mini"));
        assert_eq!(chat.body["messages"][0]["role"], json!("system"));
        assert_eq!(
            chat.body["messages"][1]["content"],
            json!("score this answer")
        );
        assert!(!chat.safe_debug.contains("sk-secret"));
        assert_eq!(chat.response_text_path, "choices[0].message.content");

        let embedding = plan_provider_request(
            ProviderFamily::Google,
            ProviderKind::Embedding,
            ProviderProtocolInput::embedding(
                "text-embedding-004",
                vec!["alpha".to_string(), "beta".to_string()],
            )
            .with_api_key("google-secret"),
        )
        .expect("google embedding plan");

        assert!(embedding.url.contains("text-embedding-004"));
        assert_eq!(embedding.headers["x-goog-api-key"], "<redacted>");
        assert_eq!(embedding.body["input"], json!(["alpha", "beta"]));
        assert!(!embedding.safe_debug.contains("google-secret"));

        let structured = plan_provider_request(
            ProviderFamily::Instructor,
            ProviderKind::StructuredLlm,
            ProviderProtocolInput::llm("gpt-4o-mini", vec![ChatMessage::user("extract fields")])
                .with_system_prompt("return JSON")
                .with_response_schema("AnswerScore"),
        )
        .expect("instructor structured plan");

        assert_eq!(structured.body["response_model"], json!("AnswerScore"));
        assert_eq!(structured.body["messages"][0]["role"], json!("system"));
        assert_eq!(structured.protocol_mode, ProviderProtocolMode::DirectHttp);
    }

    #[test]
    fn test_26_1_3_provider_claims_are_fixture_backed_and_not_release_blocking() {
        // SCEN-26.1.3 / AC3 / TEST-26.1.3
        let claims = provider_parity_claims();
        let features: BTreeSet<_> = claims.iter().map(|claim| claim.feature.as_str()).collect();

        for expected in [
            "provider::azure-openai",
            "provider::google",
            "provider::haystack",
            "provider::huggingface",
            "provider::instructor",
            "provider::litellm",
            "provider::oci-genai",
            "provider::openai-compatible",
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
            "provider claims should not block release"
        );

        let ledger = build_release_blocker_ledger();
        assert!(
            !ledger
                .entries
                .iter()
                .any(|entry| entry.category == ReleaseBlockerCategory::Provider),
            "provider category should be absent from release blockers"
        );
    }
}
