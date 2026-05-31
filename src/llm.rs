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
pub struct OpenAiCompatibleClient {
    base_url: String,
    api_key: String,
    model: String,
    embedding_model: String,
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
            client: reqwest::Client::new(),
        }
    }

    pub fn with_embedding_model(mut self, model: impl Into<String>) -> Self {
        self.embedding_model = model.into();
        self
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
}
