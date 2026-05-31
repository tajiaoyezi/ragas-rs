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
    pub fn new(_id: impl Into<String>, _name: impl Into<String>, _arguments: Value) -> Self {
        unimplemented!("TEST-5.1.1: tool call construction is not implemented yet")
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
    pub fn system(_content: impl Into<String>) -> Self {
        unimplemented!("TEST-5.1.1: system message construction is not implemented yet")
    }

    pub fn user(_content: impl Into<String>) -> Self {
        unimplemented!("TEST-5.1.1: user message construction is not implemented yet")
    }

    pub fn assistant(_content: impl Into<String>) -> Self {
        unimplemented!("TEST-5.1.1: assistant message construction is not implemented yet")
    }

    pub fn tool(_tool_call_id: impl Into<String>, _content: impl Into<String>) -> Self {
        unimplemented!("TEST-5.1.1: tool message construction is not implemented yet")
    }

    pub fn with_tool_call(self, _tool_call: ToolCall) -> Self {
        unimplemented!("TEST-5.1.1: attaching tool calls is not implemented yet")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rubric {
    pub name: String,
    pub criteria: String,
    pub weight: Option<f64>,
}

impl Rubric {
    pub fn new(_name: impl Into<String>, _criteria: impl Into<String>) -> Self {
        unimplemented!("TEST-5.1.2: rubric construction is not implemented yet")
    }

    pub fn with_weight(self, _weight: f64) -> Self {
        unimplemented!("TEST-5.1.2: rubric weighting is not implemented yet")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiTurnSample {
    pub messages: Vec<Message>,
    pub reference: Option<String>,
    pub rubrics: Vec<Rubric>,
    pub metadata: HashMap<String, String>,
}

impl MultiTurnSample {
    pub fn new(_messages: Vec<Message>) -> Self {
        unimplemented!("TEST-5.1.2: multi-turn sample construction is not implemented yet")
    }

    pub fn with_reference(self, _reference: impl Into<String>) -> Self {
        unimplemented!("TEST-5.1.2: multi-turn reference setter is not implemented yet")
    }

    pub fn with_rubric(self, _rubric: Rubric) -> Self {
        unimplemented!("TEST-5.1.2: multi-turn rubric setter is not implemented yet")
    }

    pub fn with_metadata(self, _key: impl Into<String>, _value: impl Into<String>) -> Self {
        unimplemented!("TEST-5.1.2: multi-turn metadata setter is not implemented yet")
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
}
