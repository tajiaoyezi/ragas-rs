use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::RagasError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptValueKind {
    Text,
    Number,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PromptValue {
    Text(String),
    Number(f64),
    Boolean(bool),
}

impl PromptValue {
    pub fn kind(&self) -> PromptValueKind {
        match self {
            PromptValue::Text(_) => PromptValueKind::Text,
            PromptValue::Number(_) => PromptValueKind::Number,
            PromptValue::Boolean(_) => PromptValueKind::Boolean,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PromptVariables {
    values: BTreeMap<String, PromptValue>,
}

impl PromptVariables {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_text(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.values
            .insert(name.into(), PromptValue::Text(value.into()));
        self
    }

    pub fn with_number(mut self, name: impl Into<String>, value: f64) -> Self {
        self.values.insert(name.into(), PromptValue::Number(value));
        self
    }

    pub fn with_boolean(mut self, name: impl Into<String>, value: bool) -> Self {
        self.values.insert(name.into(), PromptValue::Boolean(value));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FewShotExample {
    input: PromptVariables,
    output: String,
}

impl FewShotExample {
    pub fn new(input: PromptVariables, output: impl Into<String>) -> Self {
        Self {
            input,
            output: output.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageAdapterRule {
    target_language: String,
    instruction: String,
}

impl LanguageAdapterRule {
    pub fn new(target_language: impl Into<String>, instruction: impl Into<String>) -> Self {
        Self {
            target_language: target_language.into(),
            instruction: instruction.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderedPrompt {
    pub text: String,
    pub few_shot_examples: Vec<FewShotExample>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptTemplate {
    name: String,
    template: String,
    variables: BTreeMap<String, PromptValueKind>,
    few_shot_examples: Vec<FewShotExample>,
    language_adapter: Option<LanguageAdapterRule>,
}

impl PromptTemplate {
    pub fn new(name: impl Into<String>, template: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            template: template.into(),
            variables: BTreeMap::new(),
            few_shot_examples: Vec::new(),
            language_adapter: None,
        }
    }

    pub fn require_variable(mut self, name: impl Into<String>, kind: PromptValueKind) -> Self {
        self.variables.insert(name.into(), kind);
        self
    }

    pub fn with_few_shot(self, _example: FewShotExample) -> Self {
        unimplemented!("task 8.1 RED skeleton")
    }

    pub fn few_shot_examples(&self) -> &[FewShotExample] {
        &self.few_shot_examples
    }

    pub fn with_language_adapter(self, _adapter: LanguageAdapterRule) -> Self {
        unimplemented!("task 8.1 RED skeleton")
    }

    pub fn render(&self, _variables: &PromptVariables) -> Result<RenderedPrompt, RagasError> {
        unimplemented!("task 8.1 RED skeleton")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_8_1_1_prompt_template_renders_typed_variables_with_missing_diagnostics() {
        // SCEN-8.1.1 / AC1 / TEST-8.1.1
        let template = PromptTemplate::new(
            "faithfulness",
            "Question: {{question}}\nScore threshold: {{threshold}}",
        )
        .require_variable("question", PromptValueKind::Text)
        .require_variable("threshold", PromptValueKind::Number);

        let rendered = template
            .render(
                &PromptVariables::new()
                    .with_text("question", "What is RAG?")
                    .with_number("threshold", 0.8),
            )
            .expect("rendered prompt");

        assert!(rendered.text.contains("Question: What is RAG?"));
        assert!(rendered.text.contains("Score threshold: 0.8"));

        let missing = template
            .render(&PromptVariables::new().with_text("question", "What is RAG?"))
            .expect_err("missing variable");
        let message = missing.to_string();
        assert!(message.contains("missing prompt variable"));
        assert!(message.contains("threshold"));

        let wrong_type = template
            .render(
                &PromptVariables::new()
                    .with_text("question", "What is RAG?")
                    .with_text("threshold", "high"),
            )
            .expect_err("wrong variable type");
        let message = wrong_type.to_string();
        assert!(message.contains("prompt variable type mismatch"));
        assert!(message.contains("threshold"));
    }

    #[test]
    fn test_8_1_2_few_shot_examples_can_be_attached_and_serialized() {
        // SCEN-8.1.2 / AC2 / TEST-8.1.2
        let example = FewShotExample::new(
            PromptVariables::new().with_text("question", "What is Rust?"),
            "A systems programming language.",
        );
        let template = PromptTemplate::new("answer", "Question: {{question}}")
            .require_variable("question", PromptValueKind::Text)
            .with_few_shot(example.clone());

        let json = serde_json::to_string(&template).expect("serialize prompt template");
        assert!(json.contains("few_shot_examples"));
        assert!(json.contains("What is Rust?"));

        let roundtrip: PromptTemplate =
            serde_json::from_str(&json).expect("deserialize prompt template");
        assert_eq!(roundtrip.few_shot_examples(), &[example]);
    }

    #[test]
    fn test_8_1_3_language_adaptation_hook_rewrites_prompt_deterministically() {
        // SCEN-8.1.3 / AC3 / TEST-8.1.3
        let template = PromptTemplate::new("localized", "Answer: {{answer}}")
            .require_variable("answer", PromptValueKind::Text)
            .with_language_adapter(LanguageAdapterRule::new("zh-CN", "请用中文回答"));
        let variables = PromptVariables::new().with_text("answer", "Paris");

        let first = template.render(&variables).expect("first render");
        let second = template.render(&variables).expect("second render");

        assert_eq!(first, second);
        assert_eq!(first.text, "请用中文回答\n\nAnswer: Paris");
    }
}
