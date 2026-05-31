use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricValueType {
    Numeric,
    Discrete,
    Ranking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreNormalizationPolicy {
    Clamp,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricErrorKind {
    Provider,
    Parse,
    Validation,
    Metric,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricError {
    pub kind: MetricErrorKind,
    pub message: String,
}

impl MetricError {
    pub fn provider(_message: impl Into<String>) -> Self {
        unimplemented!("task 9.2 RED skeleton")
    }

    pub fn parse(_message: impl Into<String>) -> Self {
        unimplemented!("task 9.2 RED skeleton")
    }

    pub fn validation(_message: impl Into<String>) -> Self {
        unimplemented!("task 9.2 RED skeleton")
    }

    pub fn metric(_message: impl Into<String>) -> Self {
        unimplemented!("task 9.2 RED skeleton")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricEvidence {
    pub source: String,
    pub text: String,
}

impl MetricEvidence {
    pub fn new(_source: impl Into<String>, _text: impl Into<String>) -> Self {
        unimplemented!("task 9.2 RED skeleton")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailedMetricResult {
    pub metric_name: String,
    pub value_type: MetricValueType,
    pub score: Option<f64>,
    pub reason: Option<String>,
    pub evidence: Vec<MetricEvidence>,
    pub error: Option<MetricError>,
}

impl DetailedMetricResult {
    pub fn new(_metric_name: impl Into<String>, _value_type: MetricValueType) -> Self {
        unimplemented!("task 9.2 RED skeleton")
    }

    pub fn with_score(
        self,
        _score: f64,
        _policy: ScoreNormalizationPolicy,
    ) -> Result<Self, MetricError> {
        unimplemented!("task 9.2 RED skeleton")
    }

    pub fn with_reason(self, _reason: impl Into<String>) -> Self {
        unimplemented!("task 9.2 RED skeleton")
    }

    pub fn with_evidence(self, _evidence: MetricEvidence) -> Self {
        unimplemented!("task 9.2 RED skeleton")
    }

    pub fn with_error(self, _error: MetricError) -> Self {
        unimplemented!("task 9.2 RED skeleton")
    }
}

pub fn normalize_score(
    _score: f64,
    _policy: ScoreNormalizationPolicy,
) -> Result<f64, MetricError> {
    unimplemented!("task 9.2 RED skeleton")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_9_2_1_metric_result_stores_score_value_type_reason_evidence_and_error() {
        // SCEN-9.2.1 / AC1 / TEST-9.2.1
        let result = DetailedMetricResult::new("faithfulness", MetricValueType::Numeric)
            .with_score(0.82, ScoreNormalizationPolicy::Reject)
            .expect("score")
            .with_reason("grounded in context")
            .with_evidence(MetricEvidence::new("context[0]", "Ragas evaluates LLM apps"))
            .with_error(MetricError::metric("judge uncertainty recorded"));

        assert_eq!(result.metric_name, "faithfulness");
        assert_eq!(result.value_type, MetricValueType::Numeric);
        assert_eq!(result.score, Some(0.82));
        assert_eq!(result.reason.as_deref(), Some("grounded in context"));
        assert_eq!(result.evidence[0].source, "context[0]");
        assert_eq!(
            result.error.as_ref().map(|error| error.kind),
            Some(MetricErrorKind::Metric)
        );
    }

    #[test]
    fn test_9_2_2_score_normalization_clamps_or_rejects_invalid_scores_by_policy() {
        // SCEN-9.2.2 / AC2 / TEST-9.2.2
        assert_eq!(
            normalize_score(-0.25, ScoreNormalizationPolicy::Clamp).expect("clamped low"),
            0.0
        );
        assert_eq!(
            normalize_score(1.25, ScoreNormalizationPolicy::Clamp).expect("clamped high"),
            1.0
        );

        let error = normalize_score(1.25, ScoreNormalizationPolicy::Reject)
            .expect_err("reject out of range");
        assert_eq!(error.kind, MetricErrorKind::Validation);
        assert!(error.message.contains("score out of range"));
    }

    #[test]
    fn test_9_2_3_error_taxonomy_distinguishes_provider_parse_validation_and_metric_failures() {
        // SCEN-9.2.3 / AC3 / TEST-9.2.3
        let provider = MetricError::provider("rate limited");
        let parse = MetricError::parse("bad json");
        let validation = MetricError::validation("missing response");
        let metric = MetricError::metric("division by zero");

        assert_eq!(provider.kind, MetricErrorKind::Provider);
        assert_eq!(parse.kind, MetricErrorKind::Parse);
        assert_eq!(validation.kind, MetricErrorKind::Validation);
        assert_eq!(metric.kind, MetricErrorKind::Metric);
    }
}
