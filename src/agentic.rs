//! Agentic multi-turn metrics.
//!
//! Two families live here, both on the [`MultiTurnMetric`] trait (the core `Metric` trait is
//! single-turn):
//!
//! - **Deterministic tool-call metrics** ([`ToolCallAccuracyMetric`], [`ToolCallF1Metric`]) —
//!   compare the tool calls a conversation actually made (extracted from the assistant messages,
//!   in order) against the sample's `reference_tool_calls`, matching on tool name + arguments. No
//!   provider needed.
//! - **LLM agentic metrics** ([`AgentGoalAccuracyWithReferenceMetric`],
//!   [`AgentGoalAccuracyWithoutReferenceMetric`], [`TopicAdherenceMetric`]) — render the
//!   conversation to a transcript and judge it with an LLM. These need an [`LlmProvider`].

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use crate::metric::generate_and_parse;
use crate::{
    ChatMessage, LlmProvider, LlmRequest, Message, MessageRole, MetricMetadata,
    MetricProviderRequirement, MetricResult, MetricSampleKind, MetricValue, MultiTurnMetric,
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

// ---------------------------------------------------------------------------
// LLM agentic metrics
// ---------------------------------------------------------------------------

/// Render a multi-turn conversation into the plain-text transcript the LLM judges read.
///
/// Mirrors ragas' `pretty_repr` (each message's lines, joined by `\n`): a user turn becomes
/// `Human: <content>`, a tool result `ToolOutput: <content>`, and an assistant turn an
/// `AI: <content>` line (only when it has content) followed, when it issued tool calls, by a
/// `Tools:` line and one indented `  <name>: <args>` line per call. `System` turns have no ragas
/// equivalent and are skipped. Tool arguments render as JSON (an accepted divergence from Python's
/// dict `repr` — only the judge reads them, and byte-exact prompt parity is a project non-goal).
fn render_transcript(messages: &[Message]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for message in messages {
        match message.role {
            MessageRole::User => lines.push(format!("Human: {}", message.content)),
            MessageRole::Tool => lines.push(format!("ToolOutput: {}", message.content)),
            MessageRole::Assistant => {
                if !message.content.is_empty() {
                    lines.push(format!("AI: {}", message.content));
                }
                if !message.tool_calls.is_empty() {
                    lines.push("Tools:".to_string());
                    for call in &message.tool_calls {
                        lines.push(format!("  {}: {}", call.name, call.arguments));
                    }
                }
            }
            MessageRole::System => {}
        }
    }
    lines.join("\n")
}

#[derive(Debug, Deserialize)]
struct WorkflowOutput {
    #[serde(default)]
    user_goal: String,
    #[serde(default)]
    end_state: String,
}

#[derive(Debug, Deserialize)]
struct CompareOutcomeOutput {
    #[serde(default)]
    reason: String,
    #[serde(default)]
    verdict: Value,
}

/// Coerce a JSON `0`/`1` (string, number, or bool) verdict into `1.0`/`0.0`. Errors on anything
/// else rather than silently scoring (matches ragas' Pydantic `Literal["0", "1"]` constraint).
fn parse_binary_verdict(value: &Value) -> Result<f64, RagasError> {
    let truthy = match value {
        Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" => Some(true),
            "0" | "false" | "no" => Some(false),
            _ => None,
        },
        Value::Number(n) => n.as_f64().map(|f| f != 0.0),
        Value::Bool(b) => Some(*b),
        _ => None,
    };
    match truthy {
        Some(true) => Ok(1.0),
        Some(false) => Ok(0.0),
        None => Err(RagasError::Parse {
            message: format!("agent_goal_accuracy: unexpected verdict {value}"),
        }),
    }
}

/// Infer `{user_goal, end_state}` from the rendered conversation (call 1, shared by both variants).
async fn infer_goal_and_end_state(
    llm: &Arc<dyn LlmProvider>,
    transcript: &str,
) -> Result<WorkflowOutput, RagasError> {
    let prompt = format!(
        "Given an agentic workflow comprised of Human, AI and Tools, identify the user_goal (the \
task or objective the user wants to achieve) and the end_state (the final outcome or result of \
the workflow). Return only JSON of the form {{\"user_goal\": \"...\", \"end_state\": \"...\"}}.\n\n\
WORKFLOW:\n{transcript}"
    );
    generate_and_parse(
        llm.as_ref(),
        LlmRequest {
            messages: vec![ChatMessage::user(prompt)],
            temperature: Some(0.0),
        },
        "agent goal/end-state inference",
    )
    .await
}

/// Compare a desired outcome against the achieved outcome and emit a binary verdict (call 2).
async fn compare_outcome(
    llm: &Arc<dyn LlmProvider>,
    desired_outcome: &str,
    arrived_outcome: &str,
) -> Result<CompareOutcomeOutput, RagasError> {
    let prompt = format!(
        "Given a desired outcome and an achieved outcome, compare them and identify if they are \
the same (1) or different (0). Return only JSON of the form {{\"reason\": \"...\", \"verdict\": \
\"0\"}}.\n\nDESIRED OUTCOME: {desired_outcome}\nACHIEVED OUTCOME: {arrived_outcome}"
    );
    generate_and_parse(
        llm.as_ref(),
        LlmRequest {
            messages: vec![ChatMessage::user(prompt)],
            temperature: Some(0.0),
        },
        "agent goal outcome comparison",
    )
    .await
}

/// AgentGoalAccuracy (with reference) — infer the workflow's `end_state`, then compare it against
/// the sample's `reference` (the desired outcome) for a binary 0.0/1.0 verdict. Two LLM calls.
pub struct AgentGoalAccuracyWithReferenceMetric {
    llm: Arc<dyn LlmProvider>,
}

impl AgentGoalAccuracyWithReferenceMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl MultiTurnMetric for AgentGoalAccuracyWithReferenceMetric {
    fn metadata(&self) -> MetricMetadata {
        MetricMetadata::new("agent_goal_accuracy", MetricSampleKind::MultiTurn)
            .with_provider_requirements(vec![MetricProviderRequirement::Llm])
    }

    async fn score_multi_turn(&self, sample: &MultiTurnSample) -> Result<MetricResult, RagasError> {
        let reference = match sample.reference.as_deref() {
            Some(reference) if !reference.trim().is_empty() => reference,
            _ => {
                return Err(RagasError::Parse {
                    message: "agent_goal_accuracy (with reference) requires a reference outcome"
                        .to_string(),
                });
            }
        };
        let transcript = render_transcript(&sample.messages);
        let workflow = infer_goal_and_end_state(&self.llm, &transcript).await?;
        let comparison = compare_outcome(&self.llm, reference, &workflow.end_state).await?;
        let score = parse_binary_verdict(&comparison.verdict)?;
        Ok(
            MetricResult::success("agent_goal_accuracy", MetricValue::numeric(score))
                .with_reason(comparison.reason),
        )
    }
}

/// AgentGoalAccuracy (without reference) — infer both the `user_goal` and the `end_state`, then
/// compare them to each other (did the workflow achieve the goal it set out to?). Two LLM calls;
/// `reference` is ignored.
pub struct AgentGoalAccuracyWithoutReferenceMetric {
    llm: Arc<dyn LlmProvider>,
}

impl AgentGoalAccuracyWithoutReferenceMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl MultiTurnMetric for AgentGoalAccuracyWithoutReferenceMetric {
    fn metadata(&self) -> MetricMetadata {
        MetricMetadata::new("agent_goal_accuracy", MetricSampleKind::MultiTurn)
            .with_provider_requirements(vec![MetricProviderRequirement::Llm])
    }

    async fn score_multi_turn(&self, sample: &MultiTurnSample) -> Result<MetricResult, RagasError> {
        let transcript = render_transcript(&sample.messages);
        let workflow = infer_goal_and_end_state(&self.llm, &transcript).await?;
        let comparison =
            compare_outcome(&self.llm, &workflow.user_goal, &workflow.end_state).await?;
        let score = parse_binary_verdict(&comparison.verdict)?;
        Ok(
            MetricResult::success("agent_goal_accuracy", MetricValue::numeric(score))
                .with_reason(comparison.reason),
        )
    }
}

/// Aggregation mode for [`TopicAdherenceMetric`]. Default [`TopicAdherenceMode::F1`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TopicAdherenceMode {
    Precision,
    Recall,
    #[default]
    F1,
}

#[derive(Debug, Deserialize)]
struct TopicExtractionOutput {
    #[serde(default)]
    topics: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TopicRefusedOutput {
    // Parsed as a `Value` (not `bool`) so a stringified/numeric verdict from a real LLM
    // (`"false"`, `0`) is coerced rather than failing the whole metric — symmetric with the
    // `classifications` path below. Missing -> Null -> `coerce_bool` -> false (not refused).
    #[serde(default)]
    refused_to_answer: Value,
}

#[derive(Debug, Deserialize)]
struct TopicClassificationOutput {
    #[serde(default)]
    classifications: Vec<Value>,
}

/// Coerce a JSON bool/number/string into a boolean (mirrors ragas' `_safe_bool_conversion`).
fn coerce_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => matches!(s.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes"),
        _ => false,
    }
}

/// TopicAdherenceScore — a 3-stage LLM pipeline over a conversation: extract the topics the user
/// raised, detect (per topic) whether the AI refused to answer, and classify each topic as
/// in-scope w.r.t. the sample's `reference_topics`. The answered/in-scope booleans form a
/// confusion matrix (TP = answered & in-scope, FP = answered & off-scope, FN = refused & in-scope)
/// reduced to precision / recall / F1 per [`TopicAdherenceMode`]. `N + 2` LLM calls for `N` topics.
pub struct TopicAdherenceMetric {
    llm: Arc<dyn LlmProvider>,
    mode: TopicAdherenceMode,
}

impl TopicAdherenceMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            llm,
            mode: TopicAdherenceMode::F1,
        }
    }

    pub fn with_mode(mut self, mode: TopicAdherenceMode) -> Self {
        self.mode = mode;
        self
    }

    async fn extract_topics(&self, transcript: &str) -> Result<Vec<String>, RagasError> {
        let prompt = format!(
            "Given an interaction between Human, Tool and AI, extract the distinct topics raised \
in the Human's inputs. Return only JSON of the form {{\"topics\": [\"...\"]}}.\n\n\
CONVERSATION:\n{transcript}"
        );
        let parsed: TopicExtractionOutput = generate_and_parse(
            self.llm.as_ref(),
            LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            },
            "topic extraction",
        )
        .await?;
        Ok(parsed.topics)
    }

    async fn refused(&self, transcript: &str, topic: &str) -> Result<bool, RagasError> {
        let prompt = format!(
            "Given a topic, classify whether the AI refused to answer the question about the \
topic. Return only JSON of the form {{\"refused_to_answer\": false}}.\n\n\
CONVERSATION:\n{transcript}\n\nTOPIC: {topic}"
        );
        let parsed: TopicRefusedOutput = generate_and_parse(
            self.llm.as_ref(),
            LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            },
            "topic refusal detection",
        )
        .await?;
        Ok(coerce_bool(&parsed.refused_to_answer))
    }

    async fn classify(
        &self,
        reference_topics: &[String],
        topics: &[String],
    ) -> Result<Vec<bool>, RagasError> {
        let reference_block = reference_topics.join("\n");
        let topics_block = topics
            .iter()
            .enumerate()
            .map(|(index, topic)| format!("{}. {topic}", index + 1))
            .collect::<Vec<_>>()
            .join("\n");
        let prompt = format!(
            "Given a set of reference topics, classify whether each of the given topics falls into \
any of the reference topics. Return only JSON of the form {{\"classifications\": [true, false]}} \
with exactly one boolean per topic, in the same order.\n\n\
REFERENCE TOPICS:\n{reference_block}\n\nTOPICS:\n{topics_block}"
        );
        let parsed: TopicClassificationOutput = generate_and_parse(
            self.llm.as_ref(),
            LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            },
            "topic in-scope classification",
        )
        .await?;
        let mut classified: Vec<bool> = parsed.classifications.iter().map(coerce_bool).collect();
        // Force length to match the topic count: pad with `false`, or truncate the overflow.
        classified.resize(topics.len(), false);
        Ok(classified)
    }
}

#[async_trait]
impl MultiTurnMetric for TopicAdherenceMetric {
    fn metadata(&self) -> MetricMetadata {
        MetricMetadata::new("topic_adherence", MetricSampleKind::MultiTurn)
            .with_provider_requirements(vec![MetricProviderRequirement::Llm])
    }

    async fn score_multi_turn(&self, sample: &MultiTurnSample) -> Result<MetricResult, RagasError> {
        if sample.reference_topics.is_empty() {
            return Err(RagasError::Parse {
                message: "topic_adherence requires reference_topics".to_string(),
            });
        }
        let transcript = render_transcript(&sample.messages);
        let topics = self.extract_topics(&transcript).await?;
        if topics.is_empty() {
            return Ok(
                MetricResult::success("topic_adherence", MetricValue::numeric(0.0))
                    .with_reason("no topics extracted from the conversation"),
            );
        }

        let mut answered = Vec::with_capacity(topics.len());
        for topic in &topics {
            answered.push(!self.refused(&transcript, topic).await?);
        }
        let classified = self.classify(&sample.reference_topics, &topics).await?;

        let mut tp = 0.0f64;
        let mut fp = 0.0f64;
        let mut fneg = 0.0f64;
        for (answered, in_scope) in answered.iter().zip(classified.iter()) {
            match (*answered, *in_scope) {
                (true, true) => tp += 1.0,
                (true, false) => fp += 1.0,
                (false, true) => fneg += 1.0,
                (false, false) => {}
            }
        }

        // `eps` matches ragas: it keeps every denominator non-zero so 0/0 yields 0.0, not NaN.
        let eps = 1e-10;
        let precision = tp / (tp + fp + eps);
        let recall = tp / (tp + fneg + eps);
        let score = match self.mode {
            TopicAdherenceMode::Precision => precision,
            TopicAdherenceMode::Recall => recall,
            TopicAdherenceMode::F1 => 2.0 * precision * recall / (precision + recall + eps),
        };
        Ok(
            MetricResult::success("topic_adherence", MetricValue::numeric(score)).with_reason(
                format!("tp={tp} fp={fp} fn={fneg} over {} topics", topics.len()),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LlmResponse, Message, OpenAiCompatibleClient};
    use serde_json::json;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    /// Replays a fixed list of responses in order, recording how many times it was called.
    struct ScriptedLlm {
        responses: Mutex<VecDeque<String>>,
        calls: Mutex<usize>,
    }

    impl ScriptedLlm {
        fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: Mutex::new(responses.into_iter().map(str::to_string).collect()),
                calls: Mutex::new(0),
            }
        }

        fn call_count(&self) -> usize {
            *self.calls.lock().expect("calls")
        }
    }

    #[async_trait]
    impl LlmProvider for ScriptedLlm {
        async fn generate(&self, _request: LlmRequest) -> Result<LlmResponse, RagasError> {
            *self.calls.lock().expect("calls") += 1;
            let content = self
                .responses
                .lock()
                .expect("responses")
                .pop_front()
                .ok_or_else(|| RagasError::Provider {
                    message: "scripted LLM ran out of responses".to_string(),
                })?;
            Ok(LlmResponse {
                content,
                usage: None,
            })
        }
    }

    fn live_client() -> Option<Arc<OpenAiCompatibleClient>> {
        crate::ProviderConfig::from_env()
            .chat_client()
            .map(Arc::new)
    }

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

    // --- LLM agentic metrics -------------------------------------------------

    #[test]
    fn render_transcript_formats_human_ai_tools_and_tool_output() {
        let messages = vec![
            Message::user("book a table"),
            Message::assistant("sure").with_tool_call(ToolCall::new(
                "c1",
                "book",
                json!({ "time": "8pm" }),
            )),
            Message::tool("c1", "booked"),
            Message::system("be polite"),
        ];
        assert_eq!(
            render_transcript(&messages),
            "Human: book a table\nAI: sure\nTools:\n  book: {\"time\":\"8pm\"}\nToolOutput: booked"
        );
    }

    #[test]
    fn parse_binary_verdict_accepts_strings_numbers_and_bools() {
        assert_eq!(parse_binary_verdict(&json!("1")).unwrap(), 1.0);
        assert_eq!(parse_binary_verdict(&json!("0")).unwrap(), 0.0);
        assert_eq!(parse_binary_verdict(&json!(1)).unwrap(), 1.0);
        assert_eq!(parse_binary_verdict(&json!(true)).unwrap(), 1.0);
        assert_eq!(parse_binary_verdict(&json!("yes")).unwrap(), 1.0);
        assert!(parse_binary_verdict(&json!("maybe")).is_err());
        assert!(parse_binary_verdict(&json!(null)).is_err());
    }

    #[test]
    fn coerce_bool_handles_bool_number_and_string() {
        assert!(coerce_bool(&json!(true)));
        assert!(coerce_bool(&json!(1)));
        assert!(coerce_bool(&json!("Yes")));
        assert!(!coerce_bool(&json!(false)));
        assert!(!coerce_bool(&json!(0)));
        assert!(!coerce_bool(&json!("false")));
        assert!(!coerce_bool(&json!(null)));
    }

    #[tokio::test]
    async fn agent_goal_accuracy_with_reference_uses_two_calls_and_maps_verdict() {
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"user_goal":"book a table","end_state":"a table is booked at Jade Palace"}"#,
            r#"{"reason":"matches the reference","verdict":"1"}"#,
        ]));
        let sample = MultiTurnSample::new(vec![
            Message::user("book a table"),
            Message::assistant("done, table booked"),
        ])
        .with_reference("a table is booked");
        let metric = AgentGoalAccuracyWithReferenceMetric::new(llm.clone());
        let result = metric.score_multi_turn(&sample).await.expect("score");
        assert_eq!(result.metric_name, "agent_goal_accuracy");
        assert_eq!(numeric(&result), 1.0);
        assert_eq!(llm.call_count(), 2);
    }

    #[tokio::test]
    async fn agent_goal_accuracy_repairs_malformed_inference_via_second_call() {
        // The first inference output is unparseable; the FixOutputFormat repair recovers it through
        // the {text: ...} wrapper, so the metric still scores. Proves the repair path (shared
        // `generate_and_parse`) reaches the agentic call sites, not just src/metric.rs.
        let llm = Arc::new(ScriptedLlm::new(vec![
            "Sorry, no JSON — but the goal was to book a table and it was booked.",
            r#"{"text":"{\"user_goal\":\"book a table\",\"end_state\":\"a table is booked\"}"}"#,
            r#"{"reason":"matches the reference","verdict":"1"}"#,
        ]));
        let sample = MultiTurnSample::new(vec![
            Message::user("book a table"),
            Message::assistant("done, table booked"),
        ])
        .with_reference("a table is booked");
        let metric = AgentGoalAccuracyWithReferenceMetric::new(llm.clone());
        let result = metric
            .score_multi_turn(&sample)
            .await
            .expect("score after repair");
        assert_eq!(numeric(&result), 1.0);
        // infer (malformed) + repair + compare = 3 calls.
        assert_eq!(llm.call_count(), 3);
    }

    #[tokio::test]
    async fn agent_goal_accuracy_with_reference_scores_zero_on_mismatch() {
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"user_goal":"book a table","end_state":"no table available"}"#,
            r#"{"reason":"the goal was not achieved","verdict":"0"}"#,
        ]));
        let sample = MultiTurnSample::new(vec![Message::user("book a table")])
            .with_reference("a table is booked");
        let metric = AgentGoalAccuracyWithReferenceMetric::new(llm);
        assert_eq!(
            numeric(&metric.score_multi_turn(&sample).await.expect("score")),
            0.0
        );
    }

    #[tokio::test]
    async fn agent_goal_accuracy_with_reference_requires_a_reference() {
        let metric =
            AgentGoalAccuracyWithReferenceMetric::new(Arc::new(ScriptedLlm::new(Vec::new())));
        let sample = MultiTurnSample::new(vec![Message::user("hi")]);
        assert!(metric.score_multi_turn(&sample).await.is_err());
    }

    #[tokio::test]
    async fn agent_goal_accuracy_without_reference_compares_goal_to_end_state() {
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"user_goal":"book a table","end_state":"a table is booked"}"#,
            r#"{"reason":"the end state achieves the goal","verdict":"1"}"#,
        ]));
        let sample = MultiTurnSample::new(vec![Message::user("book a table")]);
        let metric = AgentGoalAccuracyWithoutReferenceMetric::new(llm.clone());
        assert_eq!(
            numeric(&metric.score_multi_turn(&sample).await.expect("score")),
            1.0
        );
        assert_eq!(llm.call_count(), 2);
    }

    fn topic_sample() -> MultiTurnSample {
        MultiTurnSample::new(vec![
            Message::user("What is your refund policy?"),
            Message::assistant("You can request a refund within 30 days."),
            Message::user("Can you give me medical advice?"),
            Message::assistant("Sure, take some ibuprofen."),
        ])
        .with_reference_topics(vec!["customer support".to_string()])
    }

    #[tokio::test]
    async fn topic_adherence_computes_precision_recall_f1() {
        // topics [refunds, medical]; both answered (refused=false); classified [true,false]
        // => TP=1 (refunds answered+in-scope), FP=1 (medical answered+off-scope), FN=0.
        let script = vec![
            r#"{"topics":["refunds","medical advice"]}"#,
            r#"{"refused_to_answer":false}"#,
            r#"{"refused_to_answer":false}"#,
            r#"{"classifications":[true,false]}"#,
        ];

        let llm = Arc::new(ScriptedLlm::new(script.clone()));
        let precision = numeric(
            &TopicAdherenceMetric::new(llm.clone())
                .with_mode(TopicAdherenceMode::Precision)
                .score_multi_turn(&topic_sample())
                .await
                .expect("precision"),
        );
        assert!((precision - 0.5).abs() < 1e-6, "precision={precision}");
        assert_eq!(llm.call_count(), 4); // 1 extract + 2 refusal + 1 classify

        let recall = numeric(
            &TopicAdherenceMetric::new(Arc::new(ScriptedLlm::new(script.clone())))
                .with_mode(TopicAdherenceMode::Recall)
                .score_multi_turn(&topic_sample())
                .await
                .expect("recall"),
        );
        assert!((recall - 1.0).abs() < 1e-6, "recall={recall}");

        let f1 = numeric(
            &TopicAdherenceMetric::new(Arc::new(ScriptedLlm::new(script)))
                .with_mode(TopicAdherenceMode::F1)
                .score_multi_turn(&topic_sample())
                .await
                .expect("f1"),
        );
        assert!((f1 - (2.0 * 0.5 * 1.0 / 1.5)).abs() < 1e-6, "f1={f1}");
    }

    #[tokio::test]
    async fn topic_adherence_pads_short_classification_list() {
        // 2 topics, classifications has only 1 entry -> padded with false (off-scope).
        // refunds answered+in-scope=TP; medical answered+padded-false=FP => precision 0.5.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"topics":["refunds","medical advice"]}"#,
            r#"{"refused_to_answer":false}"#,
            r#"{"refused_to_answer":false}"#,
            r#"{"classifications":[true]}"#,
        ]));
        let precision = numeric(
            &TopicAdherenceMetric::new(llm)
                .with_mode(TopicAdherenceMode::Precision)
                .score_multi_turn(&topic_sample())
                .await
                .expect("precision"),
        );
        assert!((precision - 0.5).abs() < 1e-6, "precision={precision}");
    }

    #[tokio::test]
    async fn topic_adherence_coerces_non_bool_refusal_values() {
        // Refusal comes back as a number (0) and a string ("no") rather than a JSON bool. Both
        // must coerce to "not refused" -> answered=true. With classifications [true,false] that is
        // TP=1 (refunds), FP=1 (medical) -> precision 0.5. The strict-bool parse would have errored.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"topics":["refunds","medical advice"]}"#,
            r#"{"refused_to_answer":0}"#,
            r#"{"refused_to_answer":"no"}"#,
            r#"{"classifications":[true,false]}"#,
        ]));
        let precision = numeric(
            &TopicAdherenceMetric::new(llm)
                .with_mode(TopicAdherenceMode::Precision)
                .score_multi_turn(&topic_sample())
                .await
                .expect("precision"),
        );
        assert!((precision - 0.5).abs() < 1e-6, "precision={precision}");
    }

    #[tokio::test]
    async fn topic_adherence_zero_when_no_topics_extracted() {
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"topics":[]}"#]));
        let metric = TopicAdherenceMetric::new(llm.clone());
        assert_eq!(
            numeric(
                &metric
                    .score_multi_turn(&topic_sample())
                    .await
                    .expect("score")
            ),
            0.0
        );
        assert_eq!(llm.call_count(), 1); // only extraction; no refusal/classify
    }

    #[tokio::test]
    async fn topic_adherence_requires_reference_topics() {
        let no_topics = MultiTurnSample::new(vec![Message::user("hi")]);
        let metric = TopicAdherenceMetric::new(Arc::new(ScriptedLlm::new(Vec::new())));
        assert!(metric.score_multi_turn(&no_topics).await.is_err());
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_agent_goal_accuracy_discriminates_achieved_from_failed() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = AgentGoalAccuracyWithReferenceMetric::new(client);
        let reference = "A table for two at an Italian restaurant is booked for 8pm tonight.";
        let achieved = MultiTurnSample::new(vec![
            Message::user("Book a table for two at an Italian restaurant tonight at 8pm."),
            Message::assistant("").with_tool_call(ToolCall::new(
                "c1",
                "book_table",
                json!({ "cuisine": "Italian", "party": 2, "time": "8:00pm" }),
            )),
            Message::tool(
                "c1",
                "Reservation confirmed at Bella Italia for 2 at 8:00pm.",
            ),
            Message::assistant("Your table for two at Bella Italia is booked for 8pm tonight."),
        ])
        .with_reference(reference);
        let failed = MultiTurnSample::new(vec![
            Message::user("Book a table for two at an Italian restaurant tonight at 8pm."),
            Message::assistant("").with_tool_call(ToolCall::new(
                "c1",
                "book_table",
                json!({ "cuisine": "Italian", "party": 2, "time": "8:00pm" }),
            )),
            Message::tool("c1", "No availability tonight at any Italian restaurant."),
            Message::assistant("Sorry, I couldn't find any availability tonight."),
        ])
        .with_reference(reference);

        let achieved_score = numeric(&metric.score_multi_turn(&achieved).await.expect("achieved"));
        let failed_score = numeric(&metric.score_multi_turn(&failed).await.expect("failed"));
        assert!(
            achieved_score > failed_score,
            "achieved={achieved_score} failed={failed_score}"
        );
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_agent_goal_accuracy_without_reference_discriminates_achieved_from_failed() {
        let Some(client) = live_client() else {
            return;
        };
        // No reference: the goal is inferred from the conversation and compared to the inferred
        // end-state. Achieved (table booked) must score strictly higher than failed (no availability).
        let metric = AgentGoalAccuracyWithoutReferenceMetric::new(client);
        let achieved = MultiTurnSample::new(vec![
            Message::user("Book a table for two at an Italian restaurant tonight at 8pm."),
            Message::assistant("").with_tool_call(ToolCall::new(
                "c1",
                "book_table",
                json!({ "cuisine": "Italian", "party": 2, "time": "8:00pm" }),
            )),
            Message::tool(
                "c1",
                "Reservation confirmed at Bella Italia for 2 at 8:00pm.",
            ),
            Message::assistant("Your table for two at Bella Italia is booked for 8pm tonight."),
        ]);
        let failed = MultiTurnSample::new(vec![
            Message::user("Book a table for two at an Italian restaurant tonight at 8pm."),
            Message::assistant("").with_tool_call(ToolCall::new(
                "c1",
                "book_table",
                json!({ "cuisine": "Italian", "party": 2, "time": "8:00pm" }),
            )),
            Message::tool("c1", "No availability tonight at any Italian restaurant."),
            Message::assistant("Sorry, I couldn't find any availability tonight."),
        ]);

        let achieved_score = numeric(&metric.score_multi_turn(&achieved).await.expect("achieved"));
        let failed_score = numeric(&metric.score_multi_turn(&failed).await.expect("failed"));
        assert!(
            achieved_score > failed_score,
            "achieved={achieved_score} failed={failed_score}"
        );
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_topic_adherence_scores_adherent_above_non_adherent() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = TopicAdherenceMetric::new(client);
        let topics = vec!["customer support and refunds".to_string()];
        // Adherent: answers the in-scope refund question, refuses the off-scope medical question.
        let adherent = MultiTurnSample::new(vec![
            Message::user("What is your refund policy?"),
            Message::assistant("You can request a refund within 30 days of purchase."),
            Message::user("Can you give me medical advice about my headache?"),
            Message::assistant("I'm sorry, I can't help with medical advice. Please see a doctor."),
        ])
        .with_reference_topics(topics.clone());
        // Non-adherent: answers the off-scope medical question instead of refusing.
        let non_adherent = MultiTurnSample::new(vec![
            Message::user("What is your refund policy?"),
            Message::assistant("You can request a refund within 30 days of purchase."),
            Message::user("Can you give me medical advice about my headache?"),
            Message::assistant("Sure! Take 400mg of ibuprofen every six hours and rest."),
        ])
        .with_reference_topics(topics);

        let adherent_score = numeric(&metric.score_multi_turn(&adherent).await.expect("adherent"));
        let non_adherent_score = numeric(
            &metric
                .score_multi_turn(&non_adherent)
                .await
                .expect("non_adherent"),
        );
        assert!(
            adherent_score > non_adherent_score,
            "adherent={adherent_score} non_adherent={non_adherent_score}"
        );
    }
}
