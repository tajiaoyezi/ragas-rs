use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

impl ToolCall {
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub tool_call_id: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
            tool_calls: Vec::new(),
        }
    }

    pub fn with_tool_call(mut self, tool_call: ToolCall) -> Self {
        self.tool_calls.push(tool_call);
        self
    }

    fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            tool_call_id: None,
            tool_calls: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rubric {
    pub name: String,
    pub criteria: String,
    pub weight: Option<f64>,
}

impl Rubric {
    pub fn new(name: impl Into<String>, criteria: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            criteria: criteria.into(),
            weight: None,
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiTurnSample {
    pub messages: Vec<Message>,
    pub reference: Option<String>,
    pub rubrics: Vec<Rubric>,
    /// Expected/ground-truth tool calls for the tool-call metrics. Optional; omitted from JSON
    /// when empty (backward compatible).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_tool_calls: Vec<ToolCall>,
    /// Allowed/in-scope reference topics for the topic-adherence metric. Optional; omitted from
    /// JSON when empty (backward compatible).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_topics: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl MultiTurnSample {
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            messages,
            reference: None,
            rubrics: Vec::new(),
            reference_tool_calls: Vec::new(),
            reference_topics: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    pub fn with_rubric(mut self, rubric: Rubric) -> Self {
        self.rubrics.push(rubric);
        self
    }

    pub fn with_reference_tool_calls(mut self, reference_tool_calls: Vec<ToolCall>) -> Self {
        self.reference_tool_calls = reference_tool_calls;
        self
    }

    pub fn with_reference_topics(mut self, reference_topics: Vec<String>) -> Self {
        self.reference_topics = reference_topics;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_5_1_1_messages_and_tool_calls_preserve_roles_and_ids() {
        // SCEN-5.1.1 / AC1 / TEST-5.1.1
        let tool_call = ToolCall::new("call-1", "lookup", json!({"query":"ragas"}));
        let assistant = Message::assistant("I will call a tool").with_tool_call(tool_call.clone());
        let tool = Message::tool("call-1", "tool result");

        assert_eq!(Message::system("policy").role, MessageRole::System);
        assert_eq!(Message::user("question").role, MessageRole::User);
        assert_eq!(assistant.role, MessageRole::Assistant);
        assert_eq!(assistant.tool_calls, vec![tool_call]);
        assert_eq!(tool.role, MessageRole::Tool);
        assert_eq!(tool.tool_call_id.as_deref(), Some("call-1"));
    }

    #[test]
    fn test_5_1_2_multi_turn_sample_preserves_order_reference_rubrics_and_metadata() {
        // SCEN-5.1.2 / AC2 / TEST-5.1.2
        let sample = MultiTurnSample::new(vec![
            Message::user("What changed?"),
            Message::assistant("The schema changed."),
        ])
        .with_reference("The schema should include messages.")
        .with_rubric(Rubric::new("grounding", "Answer must be grounded").with_weight(0.7))
        .with_metadata("conversation_id", "conv-1");

        assert_eq!(sample.messages[0].role, MessageRole::User);
        assert_eq!(sample.messages[1].role, MessageRole::Assistant);
        assert_eq!(
            sample.reference.as_deref(),
            Some("The schema should include messages.")
        );
        assert_eq!(sample.rubrics[0].name, "grounding");
        assert_eq!(sample.rubrics[0].weight, Some(0.7));
        assert_eq!(
            sample.metadata.get("conversation_id").map(String::as_str),
            Some("conv-1")
        );
    }

    #[test]
    fn test_5_1_3_schema_types_roundtrip_optional_fields() {
        // SCEN-5.1.3 / AC3 / TEST-5.1.3
        let sample = MultiTurnSample::new(vec![Message::assistant("No tool call")])
            .with_rubric(Rubric::new("clarity", "Answer is clear"));

        let json = serde_json::to_string(&sample).expect("serialize sample");
        let roundtrip: MultiTurnSample = serde_json::from_str(&json).expect("deserialize sample");

        assert_eq!(roundtrip.messages[0].tool_call_id, None);
        assert!(roundtrip.messages[0].tool_calls.is_empty());
        assert_eq!(roundtrip.reference, None);
        assert_eq!(roundtrip.rubrics[0].weight, None);
    }

    #[test]
    fn reference_topics_round_trip_and_omit_when_empty() {
        let with_topics = MultiTurnSample::new(vec![Message::user("hi")])
            .with_reference_topics(vec!["billing".to_string(), "refunds".to_string()]);
        let json = serde_json::to_string(&with_topics).expect("serialize");
        assert!(json.contains("reference_topics"));
        let restored: MultiTurnSample = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.reference_topics, vec!["billing", "refunds"]);

        // Empty -> omitted from JSON, so legacy datasets are unaffected.
        let without = MultiTurnSample::new(vec![Message::user("hi")]);
        let json = serde_json::to_string(&without).expect("serialize");
        assert!(!json.contains("reference_topics"));
        let restored: MultiTurnSample = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.reference_topics.is_empty());
    }
}
