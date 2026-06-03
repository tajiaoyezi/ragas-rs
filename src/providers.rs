use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use async_trait::async_trait;

use crate::{
    EmbeddingProvider, EmbeddingRequest, EmbeddingResponse, LlmProvider, LlmRequest, LlmResponse,
    RagasError, TokenUsage, UsageTracker,
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
pub enum ProviderKind {
    Llm,
    Embedding,
    StructuredLlm,
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
    vec![
        ProviderProtocolDescriptor {
            family: ProviderFamily::OpenAiCompatible,
            kind: ProviderKind::Llm,
            protocol_mode: ProviderProtocolMode::DirectHttp,
            auth_scheme: ProviderAuthScheme::BearerApiKey,
            auth_env: Some("OPENAI_API_KEY"),
            endpoint_template: "https://api.openai.com/v1",
            request_path: "/chat/completions",
            supports_system_prompt: true,
            supports_structured_output: false,
            response_text_path: "choices[0].message.content",
            usage_path: Some("usage"),
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::OpenAiCompatible,
            kind: ProviderKind::Embedding,
            protocol_mode: ProviderProtocolMode::DirectHttp,
            auth_scheme: ProviderAuthScheme::BearerApiKey,
            auth_env: Some("OPENAI_API_KEY"),
            endpoint_template: "https://api.openai.com/v1",
            request_path: "/embeddings",
            supports_system_prompt: false,
            supports_structured_output: false,
            response_text_path: "data[*].embedding",
            usage_path: Some("usage"),
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::AzureOpenAi,
            kind: ProviderKind::Llm,
            protocol_mode: ProviderProtocolMode::DirectHttp,
            auth_scheme: ProviderAuthScheme::HeaderApiKey,
            auth_env: Some("AZURE_OPENAI_API_KEY"),
            endpoint_template: "https://{resource}.openai.azure.com",
            request_path: "/openai/deployments/{deployment}/chat/completions?api-version={api_version}",
            supports_system_prompt: true,
            supports_structured_output: false,
            response_text_path: "choices[0].message.content",
            usage_path: Some("usage"),
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::AzureOpenAi,
            kind: ProviderKind::Embedding,
            protocol_mode: ProviderProtocolMode::DirectHttp,
            auth_scheme: ProviderAuthScheme::HeaderApiKey,
            auth_env: Some("AZURE_OPENAI_API_KEY"),
            endpoint_template: "https://{resource}.openai.azure.com",
            request_path: "/openai/deployments/{deployment}/embeddings?api-version={api_version}",
            supports_system_prompt: false,
            supports_structured_output: false,
            response_text_path: "data[*].embedding",
            usage_path: Some("usage"),
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::LiteLlm,
            kind: ProviderKind::Llm,
            protocol_mode: ProviderProtocolMode::DirectHttp,
            auth_scheme: ProviderAuthScheme::BearerApiKey,
            auth_env: Some("LITELLM_API_KEY"),
            endpoint_template: "litellm://completion",
            request_path: "/completion",
            supports_system_prompt: true,
            supports_structured_output: false,
            response_text_path: "choices[0].message.content",
            usage_path: Some("usage"),
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::LiteLlm,
            kind: ProviderKind::StructuredLlm,
            protocol_mode: ProviderProtocolMode::DirectHttp,
            auth_scheme: ProviderAuthScheme::BearerApiKey,
            auth_env: Some("LITELLM_API_KEY"),
            endpoint_template: "litellm://completion",
            request_path: "/completion",
            supports_system_prompt: true,
            supports_structured_output: true,
            response_text_path: "choices[0].message.content",
            usage_path: Some("usage"),
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::LiteLlm,
            kind: ProviderKind::Embedding,
            protocol_mode: ProviderProtocolMode::DirectHttp,
            auth_scheme: ProviderAuthScheme::BearerApiKey,
            auth_env: Some("LITELLM_API_KEY"),
            endpoint_template: "litellm://embedding",
            request_path: "/embedding",
            supports_system_prompt: false,
            supports_structured_output: false,
            response_text_path: "data[*].embedding",
            usage_path: Some("usage"),
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::Instructor,
            kind: ProviderKind::StructuredLlm,
            protocol_mode: ProviderProtocolMode::DirectHttp,
            auth_scheme: ProviderAuthScheme::BearerApiKey,
            auth_env: Some("OPENAI_API_KEY"),
            endpoint_template: "instructor://chat",
            request_path: "/chat/completions",
            supports_system_prompt: true,
            supports_structured_output: true,
            response_text_path: "parsed",
            usage_path: Some("usage"),
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::Haystack,
            kind: ProviderKind::Llm,
            protocol_mode: ProviderProtocolMode::DelegatedWrapper,
            auth_scheme: ProviderAuthScheme::DelegatedWrapper,
            auth_env: None,
            endpoint_template: "haystack://generator",
            request_path: "/run",
            supports_system_prompt: true,
            supports_structured_output: false,
            response_text_path: "replies[0]",
            usage_path: None,
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::Haystack,
            kind: ProviderKind::Embedding,
            protocol_mode: ProviderProtocolMode::DelegatedWrapper,
            auth_scheme: ProviderAuthScheme::DelegatedWrapper,
            auth_env: None,
            endpoint_template: "haystack://embedder",
            request_path: "/run",
            supports_system_prompt: false,
            supports_structured_output: false,
            response_text_path: "embedding",
            usage_path: None,
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::HuggingFace,
            kind: ProviderKind::Llm,
            protocol_mode: ProviderProtocolMode::DelegatedWrapper,
            auth_scheme: ProviderAuthScheme::DelegatedWrapper,
            auth_env: None,
            endpoint_template: "huggingface://pipeline",
            request_path: "/generate",
            supports_system_prompt: false,
            supports_structured_output: false,
            response_text_path: "generated_text",
            usage_path: None,
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::HuggingFace,
            kind: ProviderKind::Embedding,
            protocol_mode: ProviderProtocolMode::DelegatedWrapper,
            auth_scheme: ProviderAuthScheme::DelegatedWrapper,
            auth_env: None,
            endpoint_template: "huggingface://sentence-transformer",
            request_path: "/encode",
            supports_system_prompt: false,
            supports_structured_output: false,
            response_text_path: "embeddings",
            usage_path: None,
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::Google,
            kind: ProviderKind::Llm,
            protocol_mode: ProviderProtocolMode::DirectHttp,
            auth_scheme: ProviderAuthScheme::QueryApiKey,
            auth_env: Some("GOOGLE_API_KEY"),
            endpoint_template: "https://generativelanguage.googleapis.com/v1beta/models/{model}",
            request_path: ":generateContent",
            supports_system_prompt: true,
            supports_structured_output: false,
            response_text_path: "candidates[0].content.parts[0].text",
            usage_path: Some("usageMetadata"),
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::Google,
            kind: ProviderKind::Embedding,
            protocol_mode: ProviderProtocolMode::DirectHttp,
            auth_scheme: ProviderAuthScheme::QueryApiKey,
            auth_env: Some("GOOGLE_API_KEY"),
            endpoint_template: "https://generativelanguage.googleapis.com/v1beta/models/{model}",
            request_path: ":embedContent",
            supports_system_prompt: false,
            supports_structured_output: false,
            response_text_path: "embedding.values",
            usage_path: None,
        },
        ProviderProtocolDescriptor {
            family: ProviderFamily::OciGenAi,
            kind: ProviderKind::Llm,
            protocol_mode: ProviderProtocolMode::DirectHttp,
            auth_scheme: ProviderAuthScheme::OciConfig,
            auth_env: Some("OCI_CONFIG_FILE"),
            endpoint_template: "https://inference.generativeai.{region}.oci.oraclecloud.com",
            request_path: "/20231130/actions/generateText",
            supports_system_prompt: true,
            supports_structured_output: false,
            response_text_path: "modelOutput.generatedTexts[0].text",
            usage_path: Some("modelOutput.usage"),
        },
    ]
}

pub fn plan_provider_request(
    family: ProviderFamily,
    kind: ProviderKind,
    input: ProviderProtocolInput,
) -> Result<ProviderRequestPlan, RagasError> {
    let descriptor = provider_protocol_descriptors()
        .into_iter()
        .find(|descriptor| descriptor.family == family && descriptor.kind == kind)
        .ok_or_else(|| RagasError::Provider {
            message: format!("missing provider protocol descriptor for {family:?}/{kind:?}"),
        })?;
    let url = provider_plan_url(&descriptor, &input);
    let headers = provider_plan_headers(&descriptor);
    let body = provider_plan_body(&descriptor, &input);
    let safe_debug = format!(
        "provider={} kind={:?} mode={:?} url={}",
        family.slug(),
        kind,
        descriptor.protocol_mode,
        url
    );

    Ok(ProviderRequestPlan {
        family,
        kind,
        protocol_mode: descriptor.protocol_mode,
        url,
        headers,
        body,
        response_text_path: descriptor.response_text_path.to_string(),
        usage_path: descriptor.usage_path.map(str::to_string),
        safe_debug,
    })
}

fn provider_plan_url(
    descriptor: &ProviderProtocolDescriptor,
    input: &ProviderProtocolInput,
) -> String {
    let endpoint = descriptor
        .endpoint_template
        .replace("{model}", &input.model)
        .replace("{deployment}", &input.model)
        .replace("{api_version}", "2024-02-15-preview")
        .replace("{resource}", "resource")
        .replace("{region}", "us-ashburn-1");
    format!(
        "{}{}",
        endpoint.trim_end_matches('/'),
        descriptor.request_path
    )
    .replace("{model}", &input.model)
    .replace("{deployment}", &input.model)
    .replace("{api_version}", "2024-02-15-preview")
}

fn provider_plan_headers(descriptor: &ProviderProtocolDescriptor) -> BTreeMap<String, String> {
    let mut headers = BTreeMap::new();
    match descriptor.auth_scheme {
        ProviderAuthScheme::BearerApiKey => {
            headers.insert("Authorization".to_string(), "Bearer <redacted>".to_string());
        }
        ProviderAuthScheme::HeaderApiKey => {
            headers.insert("api-key".to_string(), "<redacted>".to_string());
        }
        ProviderAuthScheme::QueryApiKey => {
            headers.insert("x-goog-api-key".to_string(), "<redacted>".to_string());
        }
        ProviderAuthScheme::DelegatedWrapper => {
            headers.insert(
                "x-ragas-provider-boundary".to_string(),
                "delegated-wrapper".to_string(),
            );
        }
        ProviderAuthScheme::OciConfig => {
            headers.insert(
                "Authorization".to_string(),
                "Signature <redacted>".to_string(),
            );
        }
    }
    headers
}

fn provider_plan_body(
    descriptor: &ProviderProtocolDescriptor,
    input: &ProviderProtocolInput,
) -> serde_json::Value {
    match descriptor.kind {
        ProviderKind::Llm => serde_json::json!({
            "model": input.model,
            "messages": provider_plan_messages(input),
        }),
        ProviderKind::Embedding => serde_json::json!({
            "model": input.model,
            "input": input.embedding_input,
        }),
        ProviderKind::StructuredLlm => serde_json::json!({
            "model": input.model,
            "messages": provider_plan_messages(input),
            "response_model": input.response_schema,
        }),
    }
}

fn provider_plan_messages(input: &ProviderProtocolInput) -> Vec<serde_json::Value> {
    let mut messages = Vec::new();
    if let Some(system_prompt) = &input.system_prompt {
        messages.push(serde_json::json!({
            "role": "system",
            "content": system_prompt,
        }));
    }
    messages.extend(input.messages.iter().map(|message| {
        serde_json::json!({
            "role": message.role,
            "content": message.content,
        })
    }));
    messages
}

#[derive(Clone, Default)]
pub struct ProviderRegistry {
    llms: HashMap<String, Arc<dyn LlmProvider>>,
    embeddings: HashMap<String, Arc<dyn EmbeddingProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
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

    use crate::ChatMessage;
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
    fn test_26_1_1_provider_protocol_descriptors_cover_tracked_upstream_families() {
        // SCEN-26.1.1 / AC1 / TEST-26.1.1
        use std::collections::BTreeSet;
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
            !descriptor.endpoint_template.is_empty() && !descriptor.request_path.is_empty()
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
}
