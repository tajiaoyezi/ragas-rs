use crate::{
    DetailedMetricResult, MetricEvidence, MetricValueType, ScoreNormalizationPolicy,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPrecisionVariant {
    RankedRelevance,
    IdOverlap,
}

impl ContextPrecisionVariant {
    pub fn formula(&self) -> &'static str {
        match self {
            Self::RankedRelevance => {
                "sum(precision@k for relevant context at k) / total relevant contexts"
            }
            Self::IdOverlap => "retrieved reference id overlap / retrieved ids",
        }
    }
}

pub fn context_precision_from_relevance(_relevance: &[bool]) -> DetailedMetricResult {
    zero_result("context_precision")
}

pub fn id_based_context_precision<R, G>(
    _retrieved_context_ids: &[R],
    _reference_context_ids: &[G],
) -> DetailedMetricResult
where
    R: AsRef<str>,
    G: AsRef<str>,
{
    zero_result("id_based_context_precision")
}

pub fn context_recall(_reference: &str, _contexts: &[String]) -> DetailedMetricResult {
    zero_result("context_recall")
}

pub fn context_entity_recall(_reference: &str, _contexts: &[String]) -> DetailedMetricResult {
    zero_result("context_entity_recall")
}

pub fn context_relevance(_user_input: &str, _contexts: &[String]) -> DetailedMetricResult {
    zero_result("context_relevance")
}

fn zero_result(metric_name: &str) -> DetailedMetricResult {
    DetailedMetricResult::new(metric_name, MetricValueType::Numeric)
        .with_score(0.0, ScoreNormalizationPolicy::Reject)
        .expect("zero score is valid")
        .with_reason("not implemented")
}

#[allow(dead_code)]
fn unused_evidence(source: impl Into<String>, text: impl Into<String>) -> MetricEvidence {
    MetricEvidence::new(source, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_score_close(result: &DetailedMetricResult, expected: f64) {
        let actual = result.score.expect("score");
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn test_10_1_1_context_precision_variants_match_declared_formulas() {
        // SCEN-10.1.1 / AC1 / TEST-10.1.1
        assert_eq!(
            ContextPrecisionVariant::RankedRelevance.formula(),
            "sum(precision@k for relevant context at k) / total relevant contexts"
        );
        assert_eq!(
            ContextPrecisionVariant::IdOverlap.formula(),
            "retrieved reference id overlap / retrieved ids"
        );

        let ranked = context_precision_from_relevance(&[true, false, true, true]);
        assert_score_close(&ranked, (1.0 + 2.0 / 3.0 + 3.0 / 4.0) / 3.0);
        assert_eq!(ranked.evidence.len(), 3);
        assert!(ranked.reason.as_deref().unwrap_or("").contains("precision@k"));

        let id_based = id_based_context_precision(
            &["doc-a", "doc-b", "doc-c"],
            &["doc-b", "doc-c", "doc-z"],
        );
        assert_score_close(&id_based, 2.0 / 3.0);
        assert!(id_based.reason.as_deref().unwrap_or("").contains("id overlap"));
    }

    #[test]
    fn test_10_1_2_context_recall_and_entity_recall_use_reference_and_contexts() {
        // SCEN-10.1.2 / AC2 / TEST-10.1.2
        let contexts = vec![
            "Ragas evaluates LLM apps with metrics.".to_string(),
            "OpenAI embeddings score relevance for retrieval.".to_string(),
        ];
        let reference = "Ragas evaluates LLM apps. OpenAI embeddings score relevance. Rust services run fast.";

        let recall = context_recall(reference, &contexts);
        assert_score_close(&recall, 2.0 / 3.0);
        assert_eq!(recall.evidence.len(), 2);
        assert!(recall.reason.as_deref().unwrap_or("").contains("reference claims"));

        let entity_recall = context_entity_recall("Ragas uses OpenAI with Rust.", &contexts);
        assert_score_close(&entity_recall, 2.0 / 3.0);
        assert!(entity_recall.reason.as_deref().unwrap_or("").contains("entities"));
    }

    #[test]
    fn test_10_1_3_context_relevance_returns_score_with_evidence() {
        // SCEN-10.1.3 / AC3 / TEST-10.1.3
        let contexts = vec![
            "Ragas evaluates LLM applications with metrics.".to_string(),
            "Bananas are yellow.".to_string(),
        ];

        let relevance = context_relevance("How does Ragas evaluate LLM applications?", &contexts);

        assert_score_close(&relevance, 0.5);
        assert_eq!(relevance.metric_name, "context_relevance");
        assert_eq!(relevance.evidence[0].source, "context[0]");
        assert!(relevance.evidence[0].text.contains("Ragas evaluates"));
        assert!(relevance.reason.as_deref().unwrap_or("").contains("lexical overlap"));
    }
}
