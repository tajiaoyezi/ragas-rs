use std::collections::BTreeMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::RagasError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmRequest {
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub input: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub usage: Option<TokenUsage>,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, RagasError>;
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError>;
}

#[derive(Debug, Clone)]
pub struct EmbeddingAdapter<P> {
    provider: P,
    batch_size: usize,
    normalize: bool,
}

impl<P> EmbeddingAdapter<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            batch_size: usize::MAX,
            normalize: false,
        }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.max(1);
        self
    }

    pub fn with_normalization(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }
}

pub fn normalize_embedding_vector(vector: &mut [f32]) {
    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm == 0.0 {
        return;
    }
    for value in vector {
        *value /= norm;
    }
}

#[async_trait]
impl<P> EmbeddingProvider for EmbeddingAdapter<P>
where
    P: EmbeddingProvider,
{
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
        let mut embeddings = Vec::with_capacity(request.input.len());
        let mut usage = None;

        for (batch_index, batch) in request.input.chunks(self.batch_size).enumerate() {
            let batch_start = batch_index * self.batch_size;
            let batch_end = batch_start + batch.len();
            let response = self
                .provider
                .embed(EmbeddingRequest {
                    input: batch.to_vec(),
                })
                .await
                .map_err(|error| embedding_batch_error(error, batch_start, batch_end))?;

            merge_token_usage(&mut usage, response.usage);
            for mut embedding in response.embeddings {
                if self.normalize {
                    normalize_embedding_vector(&mut embedding);
                }
                embeddings.push(embedding);
            }
        }

        Ok(EmbeddingResponse { embeddings, usage })
    }
}

fn embedding_batch_error(error: RagasError, batch_start: usize, batch_end: usize) -> RagasError {
    RagasError::Provider {
        message: format!(
            "embedding batch failed batch_start={batch_start} batch_end={batch_end}: {error}"
        ),
    }
}

fn merge_token_usage(total: &mut Option<TokenUsage>, usage: Option<TokenUsage>) {
    let Some(usage) = usage else {
        return;
    };
    let total = total.get_or_insert(TokenUsage {
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: None,
    });
    add_tokens(&mut total.prompt_tokens, usage.prompt_tokens);
    add_tokens(&mut total.completion_tokens, usage.completion_tokens);
    add_tokens(&mut total.total_tokens, usage.total_tokens);
}

fn add_tokens(total: &mut Option<u32>, value: Option<u32>) {
    if let Some(value) = value {
        *total = Some(total.unwrap_or(0) + value);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiCompatibleConfig {
    base_url: String,
    api_key: String,
    model: String,
    embedding_model: Option<String>,
    headers: BTreeMap<String, String>,
    query_params: BTreeMap<String, String>,
    auth_strategy: AuthStrategy,
}

impl OpenAiCompatibleConfig {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            model: model.into(),
            embedding_model: None,
            headers: BTreeMap::new(),
            query_params: BTreeMap::new(),
            auth_strategy: AuthStrategy::Bearer,
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn with_embedding_model(mut self, model: impl Into<String>) -> Self {
        self.embedding_model = Some(model.into());
        self
    }

    pub fn with_query_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(name.into(), value.into());
        self
    }

    fn with_header_only_auth(mut self) -> Self {
        self.auth_strategy = AuthStrategy::HeaderOnly;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthStrategy {
    Bearer,
    HeaderOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AzureOpenAiConfig {
    endpoint: String,
    api_key: String,
    deployment: String,
    api_version: String,
}

impl AzureOpenAiConfig {
    pub fn new(
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
        deployment: impl Into<String>,
        api_version: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            deployment: deployment.into(),
            api_version: api_version.into(),
        }
    }

    pub fn into_openai_compatible_config(self) -> OpenAiCompatibleConfig {
        let base_url = format!(
            "{}/openai/deployments/{}",
            self.endpoint.trim_end_matches('/'),
            self.deployment
        );
        let api_key = self.api_key;

        OpenAiCompatibleConfig::new(base_url, api_key.clone(), self.deployment)
            .with_header("api-key", api_key)
            .with_query_param("api-version", self.api_version)
            .with_header_only_auth()
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiCompatibleClient {
    base_url: String,
    api_key: String,
    model: String,
    embedding_model: String,
    headers: BTreeMap<String, String>,
    query_params: BTreeMap<String, String>,
    auth_strategy: AuthStrategy,
    client: reqwest::Client,
}

impl OpenAiCompatibleClient {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        let model = model.into();
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            embedding_model: model.clone(),
            model,
            headers: BTreeMap::new(),
            query_params: BTreeMap::new(),
            auth_strategy: AuthStrategy::Bearer,
            client: reqwest::Client::new(),
        }
    }

    pub fn from_config(config: OpenAiCompatibleConfig) -> Self {
        let embedding_model = config
            .embedding_model
            .unwrap_or_else(|| config.model.clone());

        Self {
            base_url: config.base_url,
            api_key: config.api_key,
            model: config.model,
            embedding_model,
            headers: config.headers,
            query_params: config.query_params,
            auth_strategy: config.auth_strategy,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_embedding_model(mut self, model: impl Into<String>) -> Self {
        self.embedding_model = model.into();
        self
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn headers(&self) -> &BTreeMap<String, String> {
        &self.headers
    }

    pub fn chat_url(&self) -> String {
        self.url("chat/completions")
    }

    pub fn embedding_url(&self) -> String {
        self.url("embeddings")
    }

    pub fn provider_http_error(&self, status: u16, body: impl AsRef<str>) -> RagasError {
        let body = summarize_body(body.as_ref(), 2048);
        self.provider_error(format!("HTTP {status}: {body}"))
    }

    pub fn chat_payload(&self, request: &LlmRequest) -> serde_json::Value {
        json!({
            "model": self.model,
            "messages": request.messages,
            "temperature": request.temperature,
        })
    }

    pub fn embedding_payload(&self, request: &EmbeddingRequest) -> serde_json::Value {
        json!({
            "model": self.embedding_model,
            "input": request.input,
        })
    }

    pub fn sanitize_provider_error(&self, message: impl AsRef<str>) -> String {
        let mut sanitized = message.as_ref().to_string();
        if !self.api_key.is_empty() {
            sanitized = sanitized.replace(&self.api_key, "[redacted-api-key]");
        }
        for value in self.headers.values() {
            if !value.is_empty() {
                sanitized = sanitized.replace(value, "[redacted-header]");
            }
        }
        redact_bearer_tokens(&sanitized)
    }

    fn url(&self, path: &str) -> String {
        let mut url = format!("{}/{}", self.base_url.trim_end_matches('/'), path);
        if !self.query_params.is_empty() {
            let query = self
                .query_params
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect::<Vec<_>>()
                .join("&");
            url.push('?');
            url.push_str(&query);
        }
        url
    }

    fn provider_error(&self, message: impl AsRef<str>) -> RagasError {
        RagasError::Provider {
            message: self.sanitize_provider_error(message),
        }
    }

    fn apply_request_config(
        &self,
        mut request: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        if !self.api_key.is_empty() && matches!(self.auth_strategy, AuthStrategy::Bearer) {
            request = request.bearer_auth(&self.api_key);
        }
        for (name, value) in &self.headers {
            request = request.header(name.as_str(), value.as_str());
        }
        request
    }
}

fn summarize_body(body: &str, max_chars: usize) -> String {
    let mut chars = body.chars();
    let summary: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{summary}...")
    } else {
        summary
    }
}

fn redact_bearer_tokens(message: &str) -> String {
    const MARKER: &str = "Bearer ";

    let mut output = String::new();
    let mut cursor = 0;
    while let Some(relative_start) = message[cursor..].find(MARKER) {
        let marker_start = cursor + relative_start;
        output.push_str(&message[cursor..marker_start]);
        output.push_str("[redacted-bearer-token]");

        let token_start = marker_start + MARKER.len();
        let token_end = message[token_start..]
            .find(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\'' | ',' | ';' | '}' | ']'))
            .map_or(message.len(), |relative_end| token_start + relative_end);
        cursor = token_end;
    }
    output.push_str(&message[cursor..]);
    output
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleClient {
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, RagasError> {
        let response = self
            .client
            .post(self.chat_url())
            .json(&self.chat_payload(&request));
        let response = self
            .apply_request_config(response)
            .send()
            .await
            .map_err(|error| self.provider_error(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| self.provider_error(error.to_string()))?;
        if !status.is_success() {
            return Err(self.provider_http_error(status.as_u16(), body));
        }
        parse_chat_response(&body)
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAiCompatibleClient {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
        let response = self
            .client
            .post(self.embedding_url())
            .json(&self.embedding_payload(&request));
        let response = self
            .apply_request_config(response)
            .send()
            .await
            .map_err(|error| self.provider_error(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| self.provider_error(error.to_string()))?;
        if !status.is_success() {
            return Err(self.provider_http_error(status.as_u16(), body));
        }
        parse_embedding_response(&body)
    }
}

pub fn parse_chat_response(body: &str) -> Result<LlmResponse, RagasError> {
    #[derive(Deserialize)]
    struct ChatApiResponse {
        choices: Vec<Choice>,
        usage: Option<TokenUsage>,
    }

    #[derive(Deserialize)]
    struct Choice {
        message: ChatApiMessage,
    }

    #[derive(Deserialize)]
    struct ChatApiMessage {
        content: String,
    }

    let parsed: ChatApiResponse =
        serde_json::from_str(body).map_err(|error| RagasError::Parse {
            message: format!("chat response JSON: {error}"),
        })?;

    let content = parsed
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .ok_or_else(|| RagasError::Parse {
            message: "chat response contained no choices".to_string(),
        })?;

    Ok(LlmResponse {
        content,
        usage: parsed.usage,
    })
}

pub fn parse_embedding_response(body: &str) -> Result<EmbeddingResponse, RagasError> {
    #[derive(Deserialize)]
    struct EmbeddingApiResponse {
        data: Vec<EmbeddingDatum>,
        usage: Option<TokenUsage>,
    }

    #[derive(Deserialize)]
    struct EmbeddingDatum {
        index: usize,
        embedding: Vec<f32>,
    }

    let mut parsed: EmbeddingApiResponse =
        serde_json::from_str(body).map_err(|error| RagasError::Parse {
            message: format!("embedding response JSON: {error}"),
        })?;

    parsed.data.sort_by_key(|datum| datum.index);
    let embeddings = parsed
        .data
        .into_iter()
        .map(|datum| datum.embedding)
        .collect();

    Ok(EmbeddingResponse {
        embeddings,
        usage: parsed.usage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_3_1_1_chat_parser_extracts_assistant_content_and_usage() {
        // SCEN-3.1.1 / AC1 / TEST-3.1.1
        let body = r#"{
            "choices": [
                {"message": {"role": "assistant", "content": "0.82"}}
            ],
            "usage": {"prompt_tokens": 10, "completion_tokens": 3, "total_tokens": 13}
        }"#;

        let parsed = parse_chat_response(body).expect("chat response");

        assert_eq!(parsed.content, "0.82");
        assert_eq!(parsed.usage.expect("usage").total_tokens, Some(13));
    }

    #[test]
    fn test_3_1_2_embedding_parser_preserves_vector_order() {
        // SCEN-3.1.2 / AC2 / TEST-3.1.2
        let body = r#"{
            "data": [
                {"index": 0, "embedding": [1.0, 0.0]},
                {"index": 1, "embedding": [0.0, 1.0]}
            ],
            "usage": {"prompt_tokens": 4, "total_tokens": 4}
        }"#;

        let parsed = parse_embedding_response(body).expect("embedding response");

        assert_eq!(parsed.embeddings, vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
        assert_eq!(parsed.usage.expect("usage").prompt_tokens, Some(4));
    }

    #[test]
    fn test_3_1_3_client_payloads_and_errors_do_not_expose_api_key() {
        // SCEN-3.1.3 / AC3 / TEST-3.1.3
        let client =
            OpenAiCompatibleClient::new("https://example.test/v1", "sk-secret-value", "gpt-test")
                .with_embedding_model("embed-test");

        let chat_payload = client.chat_payload(&LlmRequest {
            messages: vec![ChatMessage::user("judge this")],
            temperature: Some(0.0),
        });
        assert_eq!(chat_payload["model"], "gpt-test");
        assert_eq!(chat_payload["messages"][0]["role"], "user");

        let embedding_payload = client.embedding_payload(&EmbeddingRequest {
            input: vec!["question".to_string(), "answer".to_string()],
        });
        assert_eq!(embedding_payload["model"], "embed-test");
        assert_eq!(embedding_payload["input"][1], "answer");

        let sanitized = client.sanitize_provider_error("401 sk-secret-value rejected");
        assert!(!sanitized.contains("sk-secret-value"));
        assert!(sanitized.contains("[redacted-api-key]"));
    }

    #[test]
    fn test_7_2_1_openai_client_supports_base_url_model_and_headers() {
        // SCEN-7.2.1 / AC1 / TEST-7.2.1
        let config =
            OpenAiCompatibleConfig::new("https://provider.test/v1/", "sk-test", "gpt-test")
                .with_header("X-Ragas-Trace", "trace-123")
                .with_embedding_model("embed-test");
        let client = OpenAiCompatibleClient::from_config(config);

        assert_eq!(
            client.chat_url(),
            "https://provider.test/v1/chat/completions"
        );
        assert_eq!(
            client.headers().get("X-Ragas-Trace").map(String::as_str),
            Some("trace-123")
        );

        let payload = client.chat_payload(&LlmRequest {
            messages: vec![ChatMessage::user("score this")],
            temperature: Some(0.2),
        });
        assert_eq!(payload["model"], "gpt-test");

        let embedding_payload = client.embedding_payload(&EmbeddingRequest {
            input: vec!["question".to_string()],
        });
        assert_eq!(embedding_payload["model"], "embed-test");
    }

    #[test]
    fn test_7_2_2_azure_config_maps_deployment_and_api_version() {
        // SCEN-7.2.2 / AC2 / TEST-7.2.2
        let config = AzureOpenAiConfig::new(
            "https://ragas-openai.azure.com/",
            "azure-secret",
            "ragas-deployment",
            "2024-02-15-preview",
        );
        let client = OpenAiCompatibleClient::from_config(config.into_openai_compatible_config());

        assert_eq!(
            client.chat_url(),
            "https://ragas-openai.azure.com/openai/deployments/ragas-deployment/chat/completions?api-version=2024-02-15-preview"
        );
        assert_eq!(
            client.embedding_url(),
            "https://ragas-openai.azure.com/openai/deployments/ragas-deployment/embeddings?api-version=2024-02-15-preview"
        );
        assert_eq!(
            client.headers().get("api-key").map(String::as_str),
            Some("azure-secret")
        );

        let payload = client.chat_payload(&LlmRequest {
            messages: vec![ChatMessage::user("judge")],
            temperature: None,
        });
        assert_eq!(payload["model"], "ragas-deployment");
    }

    #[test]
    fn test_7_2_3_http_errors_are_sanitized_and_preserve_status_body_summary() {
        // SCEN-7.2.3 / AC3 / TEST-7.2.3
        let client =
            OpenAiCompatibleClient::new("https://provider.test/v1", "sk-secret-value", "gpt-test")
                .with_header("X-Provider-Key", "provider-header-secret");

        let error = client.provider_http_error(
            401,
            r#"{"error":{"message":"bad key sk-secret-value provider-header-secret Bearer token-123"}}"#,
        );
        let message = error.to_string();

        assert!(message.contains("HTTP 401"));
        assert!(message.contains("bad key"));
        assert!(!message.contains("sk-secret-value"));
        assert!(!message.contains("provider-header-secret"));
        assert!(!message.contains("Bearer token-123"));
        assert!(message.contains("[redacted-api-key]"));
        assert!(message.contains("[redacted-header]"));
        assert!(message.contains("[redacted-bearer-token]"));
    }

    #[derive(Clone, Default)]
    struct RecordingEmbeddingProvider {
        calls: Arc<Mutex<Vec<Vec<String>>>>,
    }

    #[async_trait]
    impl EmbeddingProvider for RecordingEmbeddingProvider {
        async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
            self.calls
                .lock()
                .expect("calls")
                .push(request.input.clone());
            let embeddings = request
                .input
                .iter()
                .map(|value| vec![value.parse::<f32>().expect("numeric input")])
                .collect();
            Ok(EmbeddingResponse {
                embeddings,
                usage: None,
            })
        }
    }

    #[derive(Clone)]
    struct StaticEmbeddingProvider {
        embeddings: Vec<Vec<f32>>,
    }

    #[async_trait]
    impl EmbeddingProvider for StaticEmbeddingProvider {
        async fn embed(&self, _request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
            Ok(EmbeddingResponse {
                embeddings: self.embeddings.clone(),
                usage: None,
            })
        }
    }

    #[derive(Clone)]
    struct FailingEmbeddingProvider;

    #[async_trait]
    impl EmbeddingProvider for FailingEmbeddingProvider {
        async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
            if request.input.iter().any(|value| value == "bad") {
                return Err(RagasError::Provider {
                    message: "upstream embedding failed".to_string(),
                });
            }
            Ok(EmbeddingResponse {
                embeddings: request.input.iter().map(|_| vec![1.0]).collect(),
                usage: None,
            })
        }
    }

    #[tokio::test]
    async fn test_7_3_1_embedding_provider_batches_inputs_without_reordering_outputs() {
        // SCEN-7.3.1 / AC1 / TEST-7.3.1
        let provider = RecordingEmbeddingProvider::default();
        let calls = provider.calls.clone();
        let adapter = EmbeddingAdapter::new(provider).with_batch_size(2);

        let response = adapter
            .embed(EmbeddingRequest {
                input: vec![
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                    "4".to_string(),
                    "5".to_string(),
                ],
            })
            .await
            .expect("embedding response");

        assert_eq!(
            *calls.lock().expect("calls"),
            vec![
                vec!["1".to_string(), "2".to_string()],
                vec!["3".to_string(), "4".to_string()],
                vec!["5".to_string()],
            ]
        );
        assert_eq!(
            response.embeddings,
            vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0], vec![5.0]]
        );
    }

    #[tokio::test]
    async fn test_7_3_2_optional_vector_normalization_is_deterministic() {
        // SCEN-7.3.2 / AC2 / TEST-7.3.2
        let provider = StaticEmbeddingProvider {
            embeddings: vec![vec![3.0, 4.0], vec![0.0, 0.0]],
        };
        let adapter = EmbeddingAdapter::new(provider).with_normalization(true);

        let response = adapter
            .embed(EmbeddingRequest {
                input: vec!["a".to_string(), "b".to_string()],
            })
            .await
            .expect("embedding response");

        assert_eq!(response.embeddings, vec![vec![0.6, 0.8], vec![0.0, 0.0]]);
    }

    #[tokio::test]
    async fn test_7_3_3_embedding_errors_include_request_batch_position() {
        // SCEN-7.3.3 / AC3 / TEST-7.3.3
        let adapter = EmbeddingAdapter::new(FailingEmbeddingProvider).with_batch_size(2);

        let error = adapter
            .embed(EmbeddingRequest {
                input: vec![
                    "ok-1".to_string(),
                    "ok-2".to_string(),
                    "bad".to_string(),
                    "ok-4".to_string(),
                ],
            })
            .await
            .expect_err("batch failure");
        let message = error.to_string();

        assert!(message.contains("batch_start=2"));
        assert!(message.contains("batch_end=4"));
        assert!(message.contains("upstream embedding failed"));
    }
}
