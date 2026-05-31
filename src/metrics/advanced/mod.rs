use serde::{Deserialize, Serialize};

use crate::{
    DetailedMetricResult, MetricError, MetricEvidence, MetricValueType, MultiTurnSample,
    MultimodalPromptMessage, RagasError, ScoreNormalizationPolicy, ToolCall,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RubricCriterion {
    pub name: String,
    pub description: String,
    pub weight: f64,
}

impl RubricCriterion {
    pub fn new(name: impl Into<String>, description: impl Into<String>, weight: f64) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            weight,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RubricMetric {
    pub name: String,
    pub criteria: Vec<RubricCriterion>,
}

impl RubricMetric {
    pub fn new(name: impl Into<String>, criteria: Vec<RubricCriterion>) -> Self {
        Self {
            name: name.into(),
            criteria,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AspectCriticMode {
    Binary { threshold: f64 },
    Graded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AspectCriticConfig {
    pub mode: AspectCriticMode,
}

impl AspectCriticConfig {
    pub fn binary(threshold: f64) -> Self {
        Self {
            mode: AspectCriticMode::Binary { threshold },
        }
    }

    pub fn graded() -> Self {
        Self {
            mode: AspectCriticMode::Graded,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainRubric {
    pub domain: String,
    pub criteria: Vec<RubricCriterion>,
}

impl DomainRubric {
    pub fn new(domain: impl Into<String>, criteria: Vec<RubricCriterion>) -> Self {
        Self {
            domain: domain.into(),
            criteria,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstanceRubric {
    pub instance_id: String,
    pub criteria: Vec<RubricCriterion>,
    pub notes: Option<String>,
}

impl InstanceRubric {
    pub fn new(instance_id: impl Into<String>, criteria: Vec<RubricCriterion>) -> Self {
        Self {
            instance_id: instance_id.into(),
            criteria,
            notes: None,
        }
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolCallOrderPolicy {
    Strict,
    IgnoreOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGoalOutcome {
    pub goal: String,
    pub achieved: bool,
    pub evidence: Option<String>,
}

impl AgentGoalOutcome {
    pub fn achieved(goal: impl Into<String>, evidence: impl Into<String>) -> Self {
        Self {
            goal: goal.into(),
            achieved: true,
            evidence: Some(evidence.into()),
        }
    }

    pub fn missed(goal: impl Into<String>, evidence: impl Into<String>) -> Self {
        Self {
            goal: goal.into(),
            achieved: false,
            evidence: Some(evidence.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicAdherence {
    pub topic: String,
    pub adhered: bool,
    pub evidence: String,
}

impl TopicAdherence {
    pub fn new(topic: impl Into<String>, adhered: bool, evidence: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            adhered,
            evidence: evidence.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SqlJudgeVerdict {
    pub equivalent: bool,
    pub confidence: f64,
    pub reason: String,
}

impl SqlJudgeVerdict {
    pub fn new(equivalent: bool, confidence: f64, reason: impl Into<String>) -> Self {
        Self {
            equivalent,
            confidence,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultimodalMetricKind {
    Faithfulness,
    Relevance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummarizationSignals {
    pub coverage: f64,
    pub conciseness: f64,
    pub reason: Option<String>,
}

pub fn tool_call_accuracy(
    expected: &[ToolCall],
    actual: &[ToolCall],
    order_policy: ToolCallOrderPolicy,
) -> DetailedMetricResult {
    let denominator = expected.len().max(actual.len());
    let (matches, mut evidence) = match order_policy {
        ToolCallOrderPolicy::Strict => strict_tool_call_matches(expected, actual),
        ToolCallOrderPolicy::IgnoreOrder => unordered_tool_call_matches(expected, actual),
    };
    let score = ratio(matches, denominator);
    let order = match order_policy {
        ToolCallOrderPolicy::Strict => "strict",
        ToolCallOrderPolicy::IgnoreOrder => "ignore",
    };

    let mut result = numeric_result(
        "tool_call_accuracy",
        score,
        format!("order={order} matches={matches} denominator={denominator}"),
    );
    for item in evidence.drain(..) {
        result = result.with_evidence(item);
    }
    result
}

pub fn tool_call_f1(expected: &[ToolCall], actual: &[ToolCall]) -> DetailedMetricResult {
    let (matches, evidence) = unordered_tool_call_matches(expected, actual);
    let precision = ratio(matches, actual.len());
    let recall = ratio(matches, expected.len());
    let f1 = if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    };

    evidence.into_iter().fold(
        numeric_result(
            "tool_call_f1",
            f1,
            format!(
                "precision={precision:.3} recall={recall:.3} matches={matches} expected={} actual={}",
                expected.len(),
                actual.len()
            ),
        ),
        DetailedMetricResult::with_evidence,
    )
}

pub fn agent_goal_accuracy(
    sample: &MultiTurnSample,
    outcomes: &[AgentGoalOutcome],
) -> DetailedMetricResult {
    let achieved = outcomes.iter().filter(|outcome| outcome.achieved).count();
    let mut result = numeric_result(
        "agent_goal_accuracy",
        ratio(achieved, outcomes.len()),
        format!(
            "messages={} goals={} achieved={achieved}",
            sample.messages.len(),
            outcomes.len()
        ),
    );

    for outcome in outcomes {
        let status = if outcome.achieved {
            "achieved"
        } else {
            "missed"
        };
        let evidence = outcome.evidence.as_deref().unwrap_or("no evidence");
        result = result.with_evidence(MetricEvidence::new(
            format!("goal:{}", outcome.goal),
            format!("{status}: {evidence}"),
        ));
    }

    result
}

pub fn topic_adherence(topics: &[TopicAdherence]) -> DetailedMetricResult {
    let adhered = topics.iter().filter(|topic| topic.adhered).count();
    let failed = topics.len().saturating_sub(adhered);
    let result = numeric_result(
        "topic_adherence",
        ratio(adhered, topics.len()),
        format!("topics={} adhered={adhered} failed={failed}", topics.len()),
    );

    topics.iter().fold(result, |result, topic| {
        result.with_evidence(MetricEvidence::new(
            format!("topic:{}", topic.topic),
            topic.evidence.clone(),
        ))
    })
}

pub fn sql_semantic_equivalence(
    _expected_sql: &str,
    _actual_sql: &str,
    _judge: Option<SqlJudgeVerdict>,
) -> DetailedMetricResult {
    numeric_result("sql_semantic_equivalence", 0.0, "not implemented")
}

pub fn multimodal_metric_from_prompt(
    _kind: MultimodalMetricKind,
    _prompt: &MultimodalPromptMessage,
    _judge_score: f64,
    _judge_reason: impl Into<String>,
) -> Result<DetailedMetricResult, RagasError> {
    Ok(numeric_result("multimodal_metric", 0.0, "not implemented"))
}

pub fn summarization_score_from_judge_output(
    _output: &str,
) -> Result<DetailedMetricResult, MetricError> {
    Ok(numeric_result(
        "summarization_score",
        0.0,
        "not implemented",
    ))
}

fn strict_tool_call_matches(
    expected: &[ToolCall],
    actual: &[ToolCall],
) -> (usize, Vec<MetricEvidence>) {
    let mut matches = 0;
    let mut evidence = Vec::new();
    let max_len = expected.len().max(actual.len());

    for index in 0..max_len {
        let text = match (expected.get(index), actual.get(index)) {
            (Some(expected), Some(actual)) if same_tool_call(expected, actual) => {
                matches += 1;
                format!("matched {}", expected.name)
            }
            (Some(expected), Some(actual)) => format!(
                "mismatch expected {} {} actual {} {}",
                expected.name, expected.arguments, actual.name, actual.arguments
            ),
            (Some(expected), None) => format!("missing expected {}", expected.name),
            (None, Some(actual)) => format!("unexpected actual {}", actual.name),
            (None, None) => continue,
        };
        evidence.push(MetricEvidence::new(format!("tool_call[{index}]"), text));
    }

    (matches, evidence)
}

fn unordered_tool_call_matches(
    expected: &[ToolCall],
    actual: &[ToolCall],
) -> (usize, Vec<MetricEvidence>) {
    let mut used_actual = vec![false; actual.len()];
    let mut matches = 0;
    let mut evidence = Vec::new();

    for (expected_index, expected_call) in expected.iter().enumerate() {
        let actual_index = actual
            .iter()
            .enumerate()
            .find(|(actual_index, actual_call)| {
                !used_actual[*actual_index] && same_tool_call(expected_call, actual_call)
            })
            .map(|(actual_index, _)| actual_index);

        if let Some(actual_index) = actual_index {
            used_actual[actual_index] = true;
            matches += 1;
            evidence.push(MetricEvidence::new(
                format!("tool_call[{expected_index}]"),
                format!("matched {} with actual[{actual_index}]", expected_call.name),
            ));
        } else {
            evidence.push(MetricEvidence::new(
                format!("tool_call[{expected_index}]"),
                format!("missing expected {}", expected_call.name),
            ));
        }
    }

    for (actual_index, actual_call) in actual.iter().enumerate() {
        if !used_actual[actual_index] {
            evidence.push(MetricEvidence::new(
                format!("actual_tool_call[{actual_index}]"),
                format!("unexpected actual {}", actual_call.name),
            ));
        }
    }

    (matches, evidence)
}

fn same_tool_call(expected: &ToolCall, actual: &ToolCall) -> bool {
    expected.name == actual.name && expected.arguments == actual.arguments
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        1.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn numeric_result(
    metric_name: impl Into<String>,
    score: f64,
    reason: impl Into<String>,
) -> DetailedMetricResult {
    DetailedMetricResult::new(metric_name, MetricValueType::Numeric)
        .with_score(score, ScoreNormalizationPolicy::Reject)
        .expect("computed metric score is normalized")
        .with_reason(reason)
}

pub fn score_aspect_critic(raw_score: f64, config: AspectCriticConfig) -> DetailedMetricResult {
    let raw_score = raw_score.clamp(0.0, 1.0);
    let (score, reason) = match config.mode {
        AspectCriticMode::Binary { threshold } => {
            let passed = raw_score >= threshold;
            (
                if passed { 1.0 } else { 0.0 },
                format!("binary aspect critic: raw_score={raw_score:.3} threshold={threshold:.3}"),
            )
        }
        AspectCriticMode::Graded => (
            raw_score,
            format!("graded aspect critic: raw_score={raw_score:.3}"),
        ),
    };

    DetailedMetricResult::new("aspect_critic", MetricValueType::Numeric)
        .with_score(score, ScoreNormalizationPolicy::Reject)
        .expect("aspect critic score is normalized")
        .with_reason(reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;
    use serde_json::json;

    fn assert_score_close(result: &DetailedMetricResult, expected: f64) {
        let actual = result.score.expect("score");
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn test_12_1_1_rubric_metrics_accept_typed_criteria() {
        // SCEN-12.1.1 / AC1 / TEST-12.1.1
        let criterion = RubricCriterion::new("grounding", "Answer must cite context", 0.7);
        let metric = RubricMetric::new("domain_quality", vec![criterion.clone()]);

        assert_eq!(metric.name, "domain_quality");
        assert_eq!(metric.criteria, vec![criterion]);
        assert_eq!(metric.criteria[0].weight, 0.7);
    }

    #[test]
    fn test_12_1_2_aspect_critic_returns_binary_or_graded_result_by_config() {
        // SCEN-12.1.2 / AC2 / TEST-12.1.2
        let binary = score_aspect_critic(0.82, AspectCriticConfig::binary(0.8));
        assert_score_close(&binary, 1.0);
        assert!(binary.reason.as_deref().unwrap_or("").contains("binary"));

        let graded = score_aspect_critic(0.72, AspectCriticConfig::graded());
        assert_score_close(&graded, 0.72);
        assert!(graded.reason.as_deref().unwrap_or("").contains("graded"));
    }

    #[test]
    fn test_12_1_3_domain_and_instance_rubrics_serialize_for_audit() {
        // SCEN-12.1.3 / AC3 / TEST-12.1.3
        let criterion = RubricCriterion::new("style", "Use concise wording", 0.4);
        let domain = DomainRubric::new("support", vec![criterion.clone()]);
        let instance =
            InstanceRubric::new("row-42", vec![criterion]).with_notes("customer tier: enterprise");

        let domain_json = serde_json::to_string(&domain).expect("domain rubric JSON");
        let instance_json = serde_json::to_string(&instance).expect("instance rubric JSON");

        assert!(domain_json.contains("\"domain\":\"support\""));
        assert!(domain_json.contains("\"style\""));
        assert!(instance_json.contains("\"instance_id\":\"row-42\""));
        assert!(instance_json.contains("customer tier"));

        let roundtrip: InstanceRubric =
            serde_json::from_str(&instance_json).expect("roundtrip instance rubric");
        assert_eq!(
            roundtrip.notes.as_deref(),
            Some("customer tier: enterprise")
        );
        assert_eq!(roundtrip.criteria.len(), 1);
    }

    #[test]
    fn test_12_2_1_tool_call_metrics_compare_names_args_and_order_policy() {
        // SCEN-12.2.1 / AC1 / TEST-12.2.1
        let expected = vec![
            ToolCall::new("expected-1", "search", json!({"query": "ragas"})),
            ToolCall::new("expected-2", "open", json!({"url": "docs"})),
        ];
        let actual_one_arg_mismatch = vec![
            ToolCall::new("actual-1", "search", json!({"query": "ragas"})),
            ToolCall::new("actual-2", "open", json!({"url": "wrong"})),
        ];
        let swapped_actual = vec![
            ToolCall::new("actual-2", "open", json!({"url": "docs"})),
            ToolCall::new("actual-1", "search", json!({"query": "ragas"})),
        ];

        let strict = tool_call_accuracy(
            &expected,
            &actual_one_arg_mismatch,
            ToolCallOrderPolicy::Strict,
        );
        assert_score_close(&strict, 0.5);
        assert!(
            strict
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("order=strict")
        );

        let strict_swapped =
            tool_call_accuracy(&expected, &swapped_actual, ToolCallOrderPolicy::Strict);
        assert_score_close(&strict_swapped, 0.0);

        let unordered =
            tool_call_accuracy(&expected, &swapped_actual, ToolCallOrderPolicy::IgnoreOrder);
        assert_score_close(&unordered, 1.0);
        assert!(
            unordered
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("order=ignore")
        );

        let with_extra_call = vec![
            expected[0].clone(),
            expected[1].clone(),
            ToolCall::new("extra", "lookup_price", json!({"ticker": "RAG"})),
        ];
        let f1 = tool_call_f1(&expected, &with_extra_call);
        assert_score_close(&f1, 0.8);
        assert!(
            f1.reason
                .as_deref()
                .unwrap_or("")
                .contains("precision=0.667")
        );
    }

    #[test]
    fn test_12_2_2_agent_goal_accuracy_supports_multi_turn_traces() {
        // SCEN-12.2.2 / AC2 / TEST-12.2.2
        let sample = MultiTurnSample::new(vec![
            Message::user("Find the refund policy."),
            Message::assistant("I will search the policy docs.").with_tool_call(ToolCall::new(
                "call-1",
                "search",
                json!({"query": "refund policy"}),
            )),
            Message::tool("call-1", "Refunds are available within 30 days."),
            Message::assistant("Refunds are available within 30 days."),
        ]);
        let outcomes = vec![
            AgentGoalOutcome::achieved("locate policy", "search tool returned refund policy"),
            AgentGoalOutcome::missed("ask clarifying question", "assistant answered directly"),
        ];

        let result = agent_goal_accuracy(&sample, &outcomes);

        assert_score_close(&result, 0.5);
        assert!(
            result
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("messages=4")
        );
        assert_eq!(result.evidence.len(), 2);
        assert_eq!(result.evidence[0].source, "goal:locate policy");
        assert!(
            result.evidence[1]
                .text
                .contains("assistant answered directly")
        );
    }

    #[test]
    fn test_12_2_3_topic_adherence_records_per_topic_evidence() {
        // SCEN-12.2.3 / AC3 / TEST-12.2.3
        let topics = vec![
            TopicAdherence::new("refunds", true, "answer states the 30 day refund window"),
            TopicAdherence::new(
                "medical advice",
                false,
                "answer never discusses medical advice",
            ),
        ];

        let result = topic_adherence(&topics);

        assert_score_close(&result, 0.5);
        assert!(result.reason.as_deref().unwrap_or("").contains("failed=1"));
        assert_eq!(result.evidence.len(), 2);
        assert_eq!(result.evidence[0].source, "topic:refunds");
        assert_eq!(result.evidence[1].source, "topic:medical advice");
    }

    #[test]
    fn test_12_3_1_sql_semantic_equivalence_compares_normalized_sql_or_judge_output() {
        // SCEN-12.3.1 / AC1 / TEST-12.3.1
        let normalized = sql_semantic_equivalence(
            "SELECT name FROM users WHERE id = 1;",
            " select name from users where id=1 ",
            None,
        );
        assert_score_close(&normalized, 1.0);
        assert!(
            normalized
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("normalized_sql")
        );

        let judged = sql_semantic_equivalence(
            "SELECT count(*) FROM orders;",
            "SELECT total_orders FROM daily_order_metrics;",
            Some(SqlJudgeVerdict::new(
                true,
                0.85,
                "both queries represent the same aggregate count",
            )),
        );
        assert_score_close(&judged, 0.85);
        assert!(judged.reason.as_deref().unwrap_or("").contains("judge"));
        assert!(
            judged
                .evidence
                .iter()
                .any(|item| item.text.contains("same aggregate count"))
        );
    }

    #[test]
    fn test_12_3_2_multimodal_metrics_route_through_multimodal_prompt_model() {
        // SCEN-12.3.2 / AC2 / TEST-12.3.2
        let prompt = MultimodalPromptMessage::new("user")
            .push_text("Judge whether the answer is grounded in the image.")
            .push_image_url("https://example.test/chart.png", "image/png");

        let faithfulness = multimodal_metric_from_prompt(
            MultimodalMetricKind::Faithfulness,
            &prompt,
            0.75,
            "image supports the answer",
        )
        .expect("supported multimodal prompt");
        assert_score_close(&faithfulness, 0.75);
        assert_eq!(faithfulness.metric_name, "multimodal_faithfulness");
        assert!(
            faithfulness.evidence[0]
                .text
                .contains("[image image/png] https://example.test/chart.png")
        );

        let relevance = multimodal_metric_from_prompt(
            MultimodalMetricKind::Relevance,
            &prompt,
            0.6,
            "chart is partially relevant",
        )
        .expect("supported multimodal prompt");
        assert_eq!(relevance.metric_name, "multimodal_relevance");
        assert!(
            relevance
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("partially")
        );
    }

    #[test]
    fn test_12_3_3_summarization_score_parses_coverage_and_conciseness_signals() {
        // SCEN-12.3.3 / AC3 / TEST-12.3.3
        let result = summarization_score_from_judge_output(
            r#"{"coverage":0.8,"conciseness":0.6,"reason":"covers the key points but is verbose"}"#,
        )
        .expect("valid summarization output");

        assert_score_close(&result, 0.7);
        assert!(
            result
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("coverage=0.800 conciseness=0.600")
        );
        assert_eq!(result.evidence[0].source, "coverage");
        assert_eq!(result.evidence[1].source, "conciseness");
        assert!(
            result
                .evidence
                .iter()
                .any(|item| item.text.contains("covers the key points"))
        );
    }
}
