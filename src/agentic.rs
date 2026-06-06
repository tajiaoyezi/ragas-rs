//! Agentic multi-turn metrics.
//!
//! Currently the deterministic tool-call metrics: they compare the tool calls a conversation
//! actually made (extracted from the assistant messages, in order) against the sample's
//! `reference_tool_calls`, matching on tool name + arguments. They implement [`MultiTurnMetric`]
//! (the core `Metric` trait is single-turn) and need no provider.

use async_trait::async_trait;

use crate::{
    MessageRole, MetricMetadata, MetricResult, MetricSampleKind, MetricValue, MultiTurnMetric,
    MultiTurnSample, RagasError, ToolCall, ToolCallOrderPolicy, tool_call_accuracy, tool_call_f1,
};

/// Flatten the tool calls actually issued across the conversation's assistant messages, in order.
fn extract_tool_calls(sample: &MultiTurnSample) -> Vec<ToolCall> {
    sample
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::Assistant)
        .flat_map(|message| message.tool_calls.iter().cloned())
        .collect()
}

/// ToolCallAccuracy — position-wise match of the conversation's tool calls against
/// `reference_tool_calls` (matching on name + arguments). Deterministic; `Strict` order by default.
pub struct ToolCallAccuracyMetric {
    order_policy: ToolCallOrderPolicy,
}

impl Default for ToolCallAccuracyMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolCallAccuracyMetric {
    pub fn new() -> Self {
        Self {
            order_policy: ToolCallOrderPolicy::Strict,
        }
    }

    pub fn with_order_policy(mut self, order_policy: ToolCallOrderPolicy) -> Self {
        self.order_policy = order_policy;
        self
    }
}

#[async_trait]
impl MultiTurnMetric for ToolCallAccuracyMetric {
    fn metadata(&self) -> MetricMetadata {
        MetricMetadata::new("tool_call_accuracy", MetricSampleKind::MultiTurn)
    }

    async fn score_multi_turn(&self, sample: &MultiTurnSample) -> Result<MetricResult, RagasError> {
        if sample.reference_tool_calls.is_empty() {
            return Err(RagasError::Parse {
                message: "tool_call_accuracy requires reference_tool_calls".to_string(),
            });
        }
        let actual = extract_tool_calls(sample);
        let score = tool_call_accuracy(&sample.reference_tool_calls, &actual, self.order_policy)
            .score
            .unwrap_or(0.0);
        Ok(MetricResult::success(
            "tool_call_accuracy",
            MetricValue::numeric(score),
        ))
    }
}

/// ToolCallF1 — set-based precision/recall/F1 of the conversation's tool calls against
/// `reference_tool_calls` (order-independent; name + arguments). Deterministic.
pub struct ToolCallF1Metric;

impl Default for ToolCallF1Metric {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolCallF1Metric {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MultiTurnMetric for ToolCallF1Metric {
    fn metadata(&self) -> MetricMetadata {
        MetricMetadata::new("tool_call_f1", MetricSampleKind::MultiTurn)
    }

    async fn score_multi_turn(&self, sample: &MultiTurnSample) -> Result<MetricResult, RagasError> {
        if sample.reference_tool_calls.is_empty() {
            return Err(RagasError::Parse {
                message: "tool_call_f1 requires reference_tool_calls".to_string(),
            });
        }
        let actual = extract_tool_calls(sample);
        let score = tool_call_f1(&sample.reference_tool_calls, &actual)
            .score
            .unwrap_or(0.0);
        Ok(MetricResult::success(
            "tool_call_f1",
            MetricValue::numeric(score),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;
    use serde_json::json;

    fn tc(name: &str, query: &str) -> ToolCall {
        ToolCall::new("id", name, json!({ "q": query }))
    }

    fn numeric(result: &MetricResult) -> f64 {
        result
            .value
            .clone()
            .and_then(|value| value.as_numeric())
            .expect("numeric")
    }

    fn sample_with(actual: Vec<ToolCall>, reference: Vec<ToolCall>) -> MultiTurnSample {
        let mut assistant = Message::assistant("calling tools");
        for call in actual {
            assistant = assistant.with_tool_call(call);
        }
        MultiTurnSample::new(vec![Message::user("do it"), assistant])
            .with_reference_tool_calls(reference)
    }

    #[tokio::test]
    async fn tool_call_accuracy_matches_in_order() {
        let exact = sample_with(
            vec![tc("search", "ragas"), tc("lookup", "ragas")],
            vec![tc("search", "ragas"), tc("lookup", "ragas")],
        );
        let result = ToolCallAccuracyMetric::new()
            .score_multi_turn(&exact)
            .await
            .expect("accuracy");
        assert_eq!(result.metric_name, "tool_call_accuracy");
        assert_eq!(numeric(&result), 1.0);

        // One of two positions matches in strict order -> 0.5.
        let partial = sample_with(
            vec![tc("search", "ragas"), tc("wrong", "x")],
            vec![tc("search", "ragas"), tc("lookup", "ragas")],
        );
        assert_eq!(
            numeric(
                &ToolCallAccuracyMetric::new()
                    .score_multi_turn(&partial)
                    .await
                    .expect("partial")
            ),
            0.5
        );
    }

    #[tokio::test]
    async fn tool_call_f1_is_order_independent() {
        // Same two calls, reversed order -> F1 = 1.0 (order-independent).
        let reversed = sample_with(
            vec![tc("lookup", "ragas"), tc("search", "ragas")],
            vec![tc("search", "ragas"), tc("lookup", "ragas")],
        );
        assert_eq!(
            numeric(
                &ToolCallF1Metric::new()
                    .score_multi_turn(&reversed)
                    .await
                    .expect("f1")
            ),
            1.0
        );

        // Actual missing one call -> precision 1/1, recall 1/2 -> F1 = 2*1*0.5/1.5.
        let partial = sample_with(
            vec![tc("search", "ragas")],
            vec![tc("search", "ragas"), tc("lookup", "ragas")],
        );
        let f1 = numeric(
            &ToolCallF1Metric::new()
                .score_multi_turn(&partial)
                .await
                .expect("f1"),
        );
        assert!((f1 - (2.0 * 1.0 * 0.5 / 1.5)).abs() < 1e-9);
    }

    #[tokio::test]
    async fn tool_call_metrics_require_reference_tool_calls() {
        let no_ref = MultiTurnSample::new(vec![Message::user("hi")]);
        assert!(
            ToolCallAccuracyMetric::new()
                .score_multi_turn(&no_ref)
                .await
                .is_err()
        );
        assert!(
            ToolCallF1Metric::new()
                .score_multi_turn(&no_ref)
                .await
                .is_err()
        );
    }

    #[test]
    fn reference_tool_calls_round_trip_through_json() {
        let sample = sample_with(Vec::new(), vec![tc("search", "ragas")]);
        let json = serde_json::to_string(&sample).expect("serialize");
        assert!(json.contains("reference_tool_calls"));
        let restored: MultiTurnSample = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.reference_tool_calls.len(), 1);
        assert_eq!(restored.reference_tool_calls[0].name, "search");
    }
}
