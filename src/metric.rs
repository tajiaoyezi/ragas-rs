use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    ChatMessage, EmbeddingProvider, EmbeddingRequest, LlmProvider, LlmRequest, RagasError,
    SingleTurnSample,
};

pub type BoxMetricFuture = Pin<Box<dyn Future<Output = Result<MetricResult, RagasError>> + Send>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricValue {
    Discrete(String),
    Numeric(f64),
    Ranking(Vec<RankingItem>),
}

pub struct FaithfulnessMetric {
    llm: Arc<dyn LlmProvider>,
}

impl FaithfulnessMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Metric for FaithfulnessMetric {
    fn name(&self) -> &str {
        "faithfulness"
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let prompt = format!(
            "Score whether the response is faithful to the provided contexts. Return JSON with score between 0 and 1 and reason.\nQuestion: {}\nResponse: {}\nContexts:\n{}",
            sample.user_input,
            sample.response,
            sample.retrieved_contexts.join("\n")
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;

        parse_judge_score(&response.content, self.name())
    }
}

pub struct ResponseRelevancyMetric {
    embedding: Arc<dyn EmbeddingProvider>,
}

impl ResponseRelevancyMetric {
    pub fn new(embedding: Arc<dyn EmbeddingProvider>) -> Self {
        Self { embedding }
    }
}

#[async_trait]
impl Metric for ResponseRelevancyMetric {
    fn name(&self) -> &str {
        "response_relevancy"
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let response = self
            .embedding
            .embed(EmbeddingRequest {
                input: vec![sample.user_input.clone(), sample.response.clone()],
            })
            .await?;
        if response.embeddings.len() < 2 {
            return Err(RagasError::Parse {
                message: "response relevancy requires two embeddings".to_string(),
            });
        }

        let score = cosine_similarity(&response.embeddings[0], &response.embeddings[1]);
        Ok(MetricResult::success(self.name(), MetricValue::numeric(score))
            .with_reason("cosine similarity between question and response embeddings"))
    }
}

pub struct ContextPrecisionMetric {
    embedding: Arc<dyn EmbeddingProvider>,
}

impl ContextPrecisionMetric {
    pub fn new(embedding: Arc<dyn EmbeddingProvider>) -> Self {
        Self { embedding }
    }
}

#[async_trait]
impl Metric for ContextPrecisionMetric {
    fn name(&self) -> &str {
        "context_precision"
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        if sample.retrieved_contexts.is_empty() {
            return Ok(MetricResult::success(self.name(), MetricValue::numeric(0.0))
                .with_reason("sample has no retrieved contexts"));
        }

        let mut input = Vec::with_capacity(sample.retrieved_contexts.len() + 1);
        input.push(sample.user_input.clone());
        input.extend(sample.retrieved_contexts.iter().cloned());
        let response = self.embedding.embed(EmbeddingRequest { input }).await?;
        if response.embeddings.len() != sample.retrieved_contexts.len() + 1 {
            return Err(RagasError::Parse {
                message: "context precision embedding count mismatch".to_string(),
            });
        }

        let query = &response.embeddings[0];
        let mut relevant_count = 0usize;
        let mut precision_sum = 0.0f64;
        for (rank, context_embedding) in response.embeddings.iter().skip(1).enumerate() {
            let similarity = cosine_similarity(query, context_embedding);
            if similarity >= 0.5 {
                relevant_count += 1;
                precision_sum += relevant_count as f64 / (rank + 1) as f64;
            }
        }

        let score = if relevant_count == 0 {
            0.0
        } else {
            precision_sum / relevant_count as f64
        };

        Ok(MetricResult::success(self.name(), MetricValue::numeric(score))
            .with_reason("average precision over contexts with embedding similarity >= 0.5"))
    }
}

fn parse_judge_score(content: &str, metric_name: &str) -> Result<MetricResult, RagasError> {
    let parsed: Value = serde_json::from_str(content).map_err(|error| RagasError::Parse {
        message: format!("judge JSON: {error}"),
    })?;
    let score = parsed
        .get("score")
        .and_then(Value::as_f64)
        .ok_or_else(|| RagasError::Parse {
            message: "judge JSON missing numeric score".to_string(),
        })?;
    let mut result = MetricResult::success(metric_name, MetricValue::numeric(score));
    if let Some(reason) = parsed.get("reason").and_then(Value::as_str) {
        result = result.with_reason(reason);
    }
    Ok(result)
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> f64 {
    let len = left.len().min(right.len());
    if len == 0 {
        return 0.0;
    }

    let mut dot = 0.0f64;
    let mut left_norm = 0.0f64;
    let mut right_norm = 0.0f64;
    for index in 0..len {
        let l = left[index] as f64;
        let r = right[index] as f64;
        dot += l * r;
        left_norm += l * l;
        right_norm += r * r;
    }

    if left_norm == 0.0 || right_norm == 0.0 {
        return 0.0;
    }

    dot / (left_norm.sqrt() * right_norm.sqrt())
}

impl MetricValue {
    pub fn numeric(value: f64) -> Self {
        Self::Numeric(value)
    }

    pub fn discrete(value: impl Into<String>) -> Self {
        Self::Discrete(value.into())
    }

    pub fn ranking(items: Vec<RankingItem>) -> Self {
        Self::Ranking(items)
    }

    pub fn as_numeric(&self) -> Option<f64> {
        match self {
            Self::Numeric(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_discrete(&self) -> Option<&str> {
        match self {
            Self::Discrete(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_ranking(&self) -> Option<&[RankingItem]> {
        match self {
            Self::Ranking(items) => Some(items),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankingItem {
    pub item: String,
    pub score: f64,
}

impl RankingItem {
    pub fn new(item: impl Into<String>, score: f64) -> Self {
        Self {
            item: item.into(),
            score,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricResult {
    pub metric_name: String,
    pub value: Option<MetricValue>,
    pub reason: Option<String>,
    pub error: Option<String>,
}

impl MetricResult {
    pub fn success(metric_name: impl Into<String>, value: MetricValue) -> Self {
        Self {
            metric_name: metric_name.into(),
            value: Some(value),
            reason: None,
            error: None,
        }
    }

    pub fn failure(metric_name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            metric_name: metric_name.into(),
            value: None,
            reason: None,
            error: Some(error.into()),
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

#[async_trait]
pub trait Metric: Send + Sync {
    fn name(&self) -> &str;

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError>;
}

pub struct FnMetric<F>
where
    F: Fn(&SingleTurnSample) -> BoxMetricFuture + Send + Sync,
{
    name: String,
    scorer: F,
}

impl<F> FnMetric<F>
where
    F: Fn(&SingleTurnSample) -> BoxMetricFuture + Send + Sync,
{
    pub fn new(name: impl Into<String>, scorer: F) -> Self {
        Self {
            name: name.into(),
            scorer,
        }
    }
}

#[async_trait]
impl<F> Metric for FnMetric<F>
where
    F: Fn(&SingleTurnSample) -> BoxMetricFuture + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        (self.scorer)(sample).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::{EmbeddingResponse, LlmResponse};

    #[test]
    fn test_2_1_1_metric_values_expose_typed_accessors() {
        // SCEN-2.1.1 / AC1 / TEST-2.1.1
        let numeric = MetricValue::numeric(0.75);
        assert_eq!(numeric.as_numeric(), Some(0.75));
        assert_eq!(numeric.as_discrete(), None);

        let discrete = MetricValue::discrete("pass");
        assert_eq!(discrete.as_discrete(), Some("pass"));
        assert_eq!(discrete.as_numeric(), None);

        let ranking = MetricValue::ranking(vec![
            RankingItem::new("ctx-a", 0.91),
            RankingItem::new("ctx-b", 0.33),
        ]);
        let items = ranking.as_ranking().expect("ranking items");
        assert_eq!(items[0].item, "ctx-a");
        assert_eq!(items[0].score, 0.91);
    }

    #[test]
    fn test_2_1_2_metric_results_preserve_success_and_failure_details() {
        // SCEN-2.1.2 / AC2 / TEST-2.1.2
        let success =
            MetricResult::success("faithfulness", MetricValue::numeric(0.8)).with_reason("grounded");

        assert_eq!(success.metric_name, "faithfulness");
        assert_eq!(success.value.and_then(|value| value.as_numeric()), Some(0.8));
        assert_eq!(success.reason.as_deref(), Some("grounded"));
        assert!(success.error.is_none());

        let failure = MetricResult::failure("faithfulness", "provider failed");
        assert_eq!(failure.metric_name, "faithfulness");
        assert!(failure.value.is_none());
        assert_eq!(failure.error.as_deref(), Some("provider failed"));
    }

    #[tokio::test]
    async fn test_2_1_3_custom_metric_scores_asynchronously() {
        // SCEN-2.1.3 / AC3 / TEST-2.1.3
        let metric = FnMetric::new("answer_length", |sample: &SingleTurnSample| {
            let len = sample.response.len() as f64;
            Box::pin(async move {
                Ok(MetricResult::success(
                    "answer_length",
                    MetricValue::numeric(len),
                ))
            })
        });
        let sample = SingleTurnSample::new("Question", "Answer", vec!["Context".to_string()]);

        let result = metric.score(&sample).await.expect("metric result");

        assert_eq!(metric.name(), "answer_length");
        assert_eq!(result.value.and_then(|value| value.as_numeric()), Some(6.0));
    }

    struct StaticLlm {
        content: String,
    }

    #[async_trait]
    impl LlmProvider for StaticLlm {
        async fn generate(&self, _request: LlmRequest) -> Result<LlmResponse, RagasError> {
            Ok(LlmResponse {
                content: self.content.clone(),
                usage: None,
            })
        }
    }

    struct MapEmbeddingProvider {
        vectors: HashMap<String, Vec<f32>>,
    }

    #[async_trait]
    impl EmbeddingProvider for MapEmbeddingProvider {
        async fn embed(
            &self,
            request: EmbeddingRequest,
        ) -> Result<EmbeddingResponse, RagasError> {
            let embeddings = request
                .input
                .iter()
                .map(|input| self.vectors.get(input).cloned().unwrap_or_default())
                .collect();
            Ok(EmbeddingResponse {
                embeddings,
                usage: None,
            })
        }
    }

    #[tokio::test]
    async fn test_4_1_2_faithfulness_parses_llm_judgement() {
        // SCEN-4.1.2 / AC2 / TEST-4.1.2
        let metric = FaithfulnessMetric::new(Arc::new(StaticLlm {
            content: r#"{"score":0.7,"reason":"supported by context"}"#.to_string(),
        }));
        let sample = SingleTurnSample::new(
            "What is Ragas?",
            "Ragas evaluates LLM apps.",
            vec!["Ragas evaluates LLM applications.".to_string()],
        );

        let result = metric.score(&sample).await.expect("faithfulness");

        assert_eq!(result.metric_name, "faithfulness");
        assert_eq!(result.value.and_then(|value| value.as_numeric()), Some(0.7));
        assert_eq!(result.reason.as_deref(), Some("supported by context"));
    }

    #[tokio::test]
    async fn test_4_1_3_response_relevancy_uses_cosine_similarity() {
        // SCEN-4.1.3 / AC3 / TEST-4.1.3
        let metric = ResponseRelevancyMetric::new(Arc::new(MapEmbeddingProvider {
            vectors: HashMap::from([
                ("question".to_string(), vec![1.0, 0.0]),
                ("answer".to_string(), vec![0.5, 0.5]),
            ]),
        }));
        let sample = SingleTurnSample::new("question", "answer", vec!["context".to_string()]);

        let result = metric.score(&sample).await.expect("relevancy");
        let score = result.value.and_then(|value| value.as_numeric()).unwrap();

        assert!((score - 0.70710678).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_4_1_4_context_precision_computes_average_precision() {
        // SCEN-4.1.4 / AC4 / TEST-4.1.4
        let metric = ContextPrecisionMetric::new(Arc::new(MapEmbeddingProvider {
            vectors: HashMap::from([
                ("question".to_string(), vec![1.0, 0.0]),
                ("ctx-a".to_string(), vec![1.0, 0.0]),
                ("ctx-b".to_string(), vec![0.0, 1.0]),
                ("ctx-c".to_string(), vec![0.8, 0.2]),
            ]),
        }));
        let sample = SingleTurnSample::new(
            "question",
            "answer",
            vec!["ctx-a".to_string(), "ctx-b".to_string(), "ctx-c".to_string()],
        );

        let result = metric.score(&sample).await.expect("context precision");
        let score = result.value.and_then(|value| value.as_numeric()).unwrap();

        assert!((score - 0.83333333).abs() < 0.0001);
    }
}
