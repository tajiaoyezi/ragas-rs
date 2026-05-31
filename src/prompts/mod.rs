use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::RagasError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairStrategy {
    None,
    ExtractJsonObject,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedJudgeOutput {
    pub score: f64,
    pub reason: Option<String>,
    pub raw: serde_json::Value,
    pub repaired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputParseDiagnostic {
    pub message: String,
    pub raw_excerpt: String,
    pub repair_strategy: RepairStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultimodalPromptPart {
    Text { text: String },
    ImageUrl { url: String, mime_type: String },
}

impl MultimodalPromptPart {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    pub fn image_url(url: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self::ImageUrl {
            url: url.into(),
            mime_type: mime_type.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultimodalPromptMessage {
    pub role: String,
    pub parts: Vec<MultimodalPromptPart>,
}

impl MultimodalPromptMessage {
    pub fn new(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            parts: Vec::new(),
        }
    }

    pub fn push_text(mut self, text: impl Into<String>) -> Self {
        self.parts.push(MultimodalPromptPart::text(text));
        self
    }

    pub fn push_image_url(
        mut self,
        url: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Self {
        self.parts
            .push(MultimodalPromptPart::image_url(url, mime_type));
        self
    }

    pub fn render_text_scaffold(&self) -> Result<String, RagasError> {
        let mut rendered = Vec::with_capacity(self.parts.len());
        for (index, part) in self.parts.iter().enumerate() {
            match part {
                MultimodalPromptPart::Text { text } => rendered.push(format!("[text] {text}")),
                MultimodalPromptPart::ImageUrl { url, mime_type } => {
                    if !is_supported_image_mime_type(mime_type) {
                        return Err(RagasError::Prompt {
                            message: format!(
                                "unsupported media type '{mime_type}' at part_index={index}"
                            ),
                        });
                    }
                    rendered.push(format!("[image {mime_type}] {url}"));
                }
            }
        }
        Ok(rendered.join("\n"))
    }
}

fn is_supported_image_mime_type(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/png" | "image/jpeg" | "image/webp" | "image/gif"
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JudgeOutputParser {
    repair_strategy: RepairStrategy,
}

impl JudgeOutputParser {
    pub fn new() -> Self {
        Self {
            repair_strategy: RepairStrategy::None,
        }
    }

    pub fn with_repair_strategy(mut self, repair_strategy: RepairStrategy) -> Self {
        self.repair_strategy = repair_strategy;
        self
    }

    pub fn parse(&self, output: &str) -> Result<ParsedJudgeOutput, OutputParseDiagnostic> {
        let (candidate, repaired) = self.repair_output(output);
        let raw: serde_json::Value = serde_json::from_str(candidate).map_err(|error| {
            self.diagnostic(format!("judge output JSON parse failed: {error}"), output)
        })?;

        let score = raw
            .get("score")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| self.diagnostic("judge output JSON missing numeric score", output))?;

        let reason = match raw.get("reason") {
            Some(value) if value.is_null() => None,
            Some(value) => Some(value.as_str().ok_or_else(|| {
                self.diagnostic("judge output JSON reason must be a string", output)
            })?.to_string()),
            None => None,
        };

        Ok(ParsedJudgeOutput {
            score,
            reason,
            raw,
            repaired,
        })
    }

    fn repair_output<'a>(&self, output: &'a str) -> (&'a str, bool) {
        match self.repair_strategy {
            RepairStrategy::None => (output, false),
            RepairStrategy::ExtractJsonObject => {
                let Some(start) = output.find('{') else {
                    return (output, false);
                };
                let Some(end) = output.rfind('}') else {
                    return (output, false);
                };
                if end < start {
                    return (output, false);
                }
                let repaired = &output[start..=end];
                (repaired, repaired != output)
            }
        }
    }

    fn diagnostic(
        &self,
        message: impl Into<String>,
        output: &str,
    ) -> OutputParseDiagnostic {
        OutputParseDiagnostic {
            message: message.into(),
            raw_excerpt: output.chars().take(80).collect(),
            repair_strategy: self.repair_strategy,
        }
    }
}

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

    fn render(&self) -> String {
        match self {
            PromptValue::Text(value) => value.clone(),
            PromptValue::Number(value) => value.to_string(),
            PromptValue::Boolean(value) => value.to_string(),
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

    fn adapt(&self, text: &str) -> String {
        if self.instruction.is_empty() {
            text.to_string()
        } else {
            format!("{}\n\n{text}", self.instruction)
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

    pub fn with_few_shot(mut self, example: FewShotExample) -> Self {
        self.few_shot_examples.push(example);
        self
    }

    pub fn few_shot_examples(&self) -> &[FewShotExample] {
        &self.few_shot_examples
    }

    pub fn with_language_adapter(mut self, adapter: LanguageAdapterRule) -> Self {
        self.language_adapter = Some(adapter);
        self
    }

    pub fn render(&self, variables: &PromptVariables) -> Result<RenderedPrompt, RagasError> {
        let mut text = self.template.clone();
        for (name, expected_kind) in &self.variables {
            let value = variables.values.get(name).ok_or_else(|| RagasError::Prompt {
                message: format!("missing prompt variable '{name}'"),
            })?;
            if value.kind() != *expected_kind {
                return Err(RagasError::Prompt {
                    message: format!(
                        "prompt variable type mismatch for '{name}': expected {expected_kind:?}, got {:?}",
                        value.kind()
                    ),
                });
            }
            text = text.replace(&format!("{{{{{name}}}}}"), &value.render());
        }

        if let Some(adapter) = &self.language_adapter {
            text = adapter.adapt(&text);
        }

        Ok(RenderedPrompt {
            text,
            few_shot_examples: self.few_shot_examples.clone(),
        })
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

    #[test]
    fn test_8_2_1_parser_extracts_typed_json_scores_and_reasons() {
        // SCEN-8.2.1 / AC1 / TEST-8.2.1
        let parsed = JudgeOutputParser::new()
            .parse(r#"{"score":0.75,"reason":"grounded in context"}"#)
            .expect("parsed judge output");

        assert_eq!(parsed.score, 0.75);
        assert_eq!(parsed.reason.as_deref(), Some("grounded in context"));
        assert!(!parsed.repaired);
        assert_eq!(parsed.raw["score"], 0.75);
    }

    #[test]
    fn test_8_2_2_malformed_judge_output_returns_diagnostics_with_raw_excerpt() {
        // SCEN-8.2.2 / AC2 / TEST-8.2.2
        let diagnostic = JudgeOutputParser::new()
            .parse("not json; the model answered with prose instead of a score")
            .expect_err("malformed output");

        assert!(diagnostic.message.contains("judge output JSON"));
        assert!(diagnostic.raw_excerpt.contains("not json"));
        assert!(diagnostic.raw_excerpt.len() <= 80);
        assert_eq!(diagnostic.repair_strategy, RepairStrategy::None);
    }

    #[test]
    fn test_8_2_3_repair_strategy_is_explicit_and_testable() {
        // SCEN-8.2.3 / AC3 / TEST-8.2.3
        let strict = JudgeOutputParser::new();
        assert!(strict.parse("Answer: {\"score\":0.5,\"reason\":\"partial\"}").is_err());

        let repaired = JudgeOutputParser::new()
            .with_repair_strategy(RepairStrategy::ExtractJsonObject)
            .parse("Answer: ```json\n{\"score\":0.5,\"reason\":\"partial\"}\n```")
            .expect("repaired output");

        assert_eq!(repaired.score, 0.5);
        assert_eq!(repaired.reason.as_deref(), Some("partial"));
        assert!(repaired.repaired);
    }

    #[test]
    fn test_8_3_1_multimodal_message_supports_text_and_image_parts() {
        // SCEN-8.3.1 / AC1 / TEST-8.3.1
        let message = MultimodalPromptMessage::new("user")
            .push_text("Describe this image")
            .push_image_url("https://example.test/cat.png", "image/png");

        assert_eq!(message.role, "user");
        assert_eq!(
            message.parts,
            vec![
                MultimodalPromptPart::Text {
                    text: "Describe this image".to_string()
                },
                MultimodalPromptPart::ImageUrl {
                    url: "https://example.test/cat.png".to_string(),
                    mime_type: "image/png".to_string()
                },
            ]
        );
    }

    #[test]
    fn test_8_3_2_prompt_rendering_preserves_part_order() {
        // SCEN-8.3.2 / AC2 / TEST-8.3.2
        let message = MultimodalPromptMessage::new("user")
            .push_text("first")
            .push_image_url("https://example.test/one.jpg", "image/jpeg")
            .push_text("second");

        let rendered = message.render_text_scaffold().expect("rendered multimodal scaffold");

        assert_eq!(
            rendered,
            "[text] first\n[image image/jpeg] https://example.test/one.jpg\n[text] second"
        );
    }

    #[test]
    fn test_8_3_3_unsupported_media_returns_structured_error() {
        // SCEN-8.3.3 / AC3 / TEST-8.3.3
        let message = MultimodalPromptMessage::new("user")
            .push_text("inspect")
            .push_image_url("https://example.test/file.svg", "image/svg+xml");

        let error = message
            .render_text_scaffold()
            .expect_err("unsupported media");
        let message = error.to_string();

        assert!(message.contains("unsupported media type"));
        assert!(message.contains("image/svg+xml"));
        assert!(message.contains("part_index=1"));
    }
}
