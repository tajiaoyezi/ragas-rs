use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::{
    EmbeddingProvider, EmbeddingRequest, EmbeddingResponse, LlmProvider, LlmRequest, LlmResponse,
    RagasError, TokenUsage, UsageTracker,
};

#[derive(Default, Clone)]
pub struct ProviderRegistry {
    llms: HashMap<String, Arc<dyn LlmProvider>>,
    embeddings: HashMap<String, Arc<dyn EmbeddingProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_llm(
        mut self,
        name: impl Into<String>,
        provider: Arc<dyn LlmProvider>,
    ) -> Self {
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
}
