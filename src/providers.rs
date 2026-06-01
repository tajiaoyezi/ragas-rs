use std::{collections::HashMap, sync::Arc};

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

pub fn upstream_provider_descriptors() -> Vec<ProviderDescriptor> {
    Vec::new()
}

pub fn structured_llm_descriptors() -> Vec<StructuredLlmDescriptor> {
    Vec::new()
}

pub fn provider_parity_claims() -> Vec<ParityClaim> {
    Vec::new()
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

    use crate::{ChatMessage, release_blocking_claims};

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
}
