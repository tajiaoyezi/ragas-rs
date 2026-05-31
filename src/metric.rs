use std::{future::Future, pin::Pin};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{RagasError, SingleTurnSample};

pub type BoxMetricFuture = Pin<Box<dyn Future<Output = Result<MetricResult, RagasError>> + Send>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricValue {
    Discrete(String),
    Numeric(f64),
    Ranking(Vec<RankingItem>),
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
}
