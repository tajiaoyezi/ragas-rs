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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiCompatibleConfig {
    base_url: String,
    api_key: String,
    model: String,
    embedding_model: Option<String>,
    headers: BTreeMap<String, String>,
    query_params: BTreeMap<String, String>,
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
        }
    }

    pub fn with_header(self, _name: impl Into<String>, _value: impl Into<String>) -> Self {
        unimplemented!("task 7.2 RED skeleton")
    }

    pub fn with_embedding_model(mut self, model: impl Into<String>) -> Self {
        self.embedding_model = Some(model.into());
        self
    }

    pub fn with_query_param(self, _name: impl Into<String>, _value: impl Into<String>) -> Self {
        unimplemented!("task 7.2 RED skeleton")
    }
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
        unimplemented!("task 7.2 RED skeleton")
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
            client: reqwest::Client::new(),
        }
    }

    pub fn from_config(_config: OpenAiCompatibleConfig) -> Self {
        unimplemented!("task 7.2 RED skeleton")
    }

    pub fn with_embedding_model(mut self, model: impl Into<String>) -> Self {
        self.embedding_model = model.into();
        self
    }

    pub fn with_header(self, _name: impl Into<String>, _value: impl Into<String>) -> Self {
        unimplemented!("task 7.2 RED skeleton")
    }

    pub fn headers(&self) -> &BTreeMap<String, String> {
        unimplemented!("task 7.2 RED skeleton")
    }

    pub fn chat_url(&self) -> String {
        unimplemented!("task 7.2 RED skeleton")
    }

    pub fn embedding_url(&self) -> String {
        unimplemented!("task 7.2 RED skeleton")
    }

    pub fn provider_http_error(&self, _status: u16, _body: impl AsRef<str>) -> RagasError {
        unimplemented!("task 7.2 RED skeleton")
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
        let message = message.as_ref();
        if self.api_key.is_empty() {
            return message.to_string();
        }
        message.replace(&self.api_key, "[redacted-api-key]")
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), path)
    }

    fn provider_error(&self, message: impl AsRef<str>) -> RagasError {
        RagasError::Provider {
            message: self.sanitize_provider_error(message),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleClient {
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, RagasError> {
        let response = self
            .client
            .post(self.url("chat/completions"))
            .bearer_auth(&self.api_key)
            .json(&self.chat_payload(&request))
            .send()
            .await
            .map_err(|error| self.provider_error(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| self.provider_error(error.to_string()))?;
        if !status.is_success() {
            return Err(self.provider_error(format!("HTTP {status}: {body}")));
        }
        parse_chat_response(&body)
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAiCompatibleClient {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
        let response = self
            .client
            .post(self.url("embeddings"))
            .bearer_auth(&self.api_key)
            .json(&self.embedding_payload(&request))
            .send()
            .await
            .map_err(|error| self.provider_error(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| self.provider_error(error.to_string()))?;
        if !status.is_success() {
            return Err(self.provider_error(format!("HTTP {status}: {body}")));
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

    let parsed: ChatApiResponse = serde_json::from_str(body).map_err(|error| {
        RagasError::Parse {
            message: format!("chat response JSON: {error}"),
        }
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

    let mut parsed: EmbeddingApiResponse = serde_json::from_str(body).map_err(|error| {
        RagasError::Parse {
            message: format!("embedding response JSON: {error}"),
        }
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
        let client = OpenAiCompatibleClient::new(
            "https://example.test/v1",
            "sk-secret-value",
            "gpt-test",
        )
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
        let client = OpenAiCompatibleClient::new(
            "https://provider.test/v1",
            "sk-secret-value",
            "gpt-test",
        )
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
}
