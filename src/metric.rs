use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::validation::{MetricRequirements, SampleField};
use crate::{
    ChatMessage, EmbeddingProvider, EmbeddingRequest, LlmProvider, LlmRequest, RagasError,
    SingleTurnSample,
};

pub type BoxMetricFuture = Pin<Box<dyn Future<Output = Result<MetricResult, RagasError>> + Send>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricValue {
    Discrete(String),
    Numeric(f64),
    Ranking(Vec<RankingItem>),
}

pub struct FaithfulnessMetric {
    llm: Arc<dyn LlmProvider>,
}

impl FaithfulnessMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Metric for FaithfulnessMetric {
    fn name(&self) -> &str {
        "faithfulness"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![
                SampleField::UserInput,
                SampleField::Response,
                SampleField::RetrievedContexts,
            ],
        )
    }

    /// Faithful port of ragas' two-step Faithfulness:
    ///   1. decompose the response into atomic statements;
    ///   2. verify each statement against the retrieved contexts (NLI);
    ///   score = supported_statements / total_statements.
    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let statements = self.generate_statements(sample).await?;
        if statements.is_empty() {
            // ragas returns NaN when no statements can be extracted (0/0).
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(f64::NAN))
                    .with_reason("no statements could be extracted from the response"),
            );
        }

        let verdicts = self.verify_statements(sample, &statements).await?;
        let supported = verdicts.iter().filter(|verdict| verdict.verdict == 1).count();
        let total = statements.len();
        let score = supported as f64 / total as f64;
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "{supported}/{total} statements supported by the retrieved contexts"
            )),
        )
    }
}

impl FaithfulnessMetric {
    /// Step 1 — ask the LLM to break the response into standalone atomic statements.
    async fn generate_statements(
        &self,
        sample: &SingleTurnSample,
    ) -> Result<Vec<String>, RagasError> {
        let prompt = format!(
            "Break the ANSWER into a list of standalone, atomic factual statements. \
Each statement must be fully self-contained, resolving any pronouns using the QUESTION. \
Return only JSON of the form {{\"statements\": [\"...\"]}}.\n\n\
QUESTION: {}\nANSWER: {}",
            sample.user_input, sample.response,
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: StatementGenerationOutput =
            parse_json(&response.content, "faithfulness statement generation")?;
        Ok(parsed
            .statements
            .into_iter()
            .map(|statement| statement.trim().to_string())
            .filter(|statement| !statement.is_empty())
            .collect())
    }

    /// Step 2 — ask the LLM to verify each statement against the retrieved contexts.
    async fn verify_statements(
        &self,
        sample: &SingleTurnSample,
        statements: &[String],
    ) -> Result<Vec<StatementVerdict>, RagasError> {
        let numbered = statements
            .iter()
            .enumerate()
            .map(|(index, statement)| format!("{}. {statement}", index + 1))
            .collect::<Vec<_>>()
            .join("\n");
        let prompt = format!(
            "For each STATEMENT decide whether it can be directly inferred from the CONTEXT. \
Use verdict 1 when the statement is supported by the context, 0 otherwise. \
Return only JSON of the form \
{{\"verdicts\": [{{\"statement\": \"...\", \"verdict\": 0, \"reason\": \"...\"}}]}} \
with exactly one entry per statement, in the same order.\n\n\
CONTEXT:\n{}\n\nSTATEMENTS:\n{numbered}",
            sample.retrieved_contexts.join("\n"),
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: NliOutput =
            parse_json(&response.content, "faithfulness statement verification")?;
        Ok(parsed.verdicts)
    }
}

#[derive(Debug, Deserialize)]
struct StatementGenerationOutput {
    #[serde(default)]
    statements: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct NliOutput {
    #[serde(default)]
    verdicts: Vec<StatementVerdict>,
}

#[derive(Debug, Deserialize)]
struct StatementVerdict {
    verdict: i64,
}

/// Deserialize a JSON object from an LLM response, tolerating markdown fences or
/// surrounding prose by extracting the outermost `{ .. }` block (repair path).
fn parse_json<T: serde::de::DeserializeOwned>(
    content: &str,
    context: &str,
) -> Result<T, RagasError> {
    let block = extract_json_block(content);
    serde_json::from_str(block).map_err(|error| RagasError::Parse {
        message: format!("{context}: {error}"),
    })
}

fn extract_json_block(content: &str) -> &str {
    match (content.find('{'), content.rfind('}')) {
        (Some(start), Some(end)) if end >= start => &content[start..=end],
        _ => content.trim(),
    }
}

pub struct ResponseRelevancyMetric {
    llm: Arc<dyn LlmProvider>,
    embedding: Arc<dyn EmbeddingProvider>,
    strictness: usize,
}

impl ResponseRelevancyMetric {
    pub fn new(llm: Arc<dyn LlmProvider>, embedding: Arc<dyn EmbeddingProvider>) -> Self {
        Self {
            llm,
            embedding,
            strictness: 3,
        }
    }

    pub fn with_strictness(mut self, strictness: usize) -> Self {
        self.strictness = strictness.max(1);
        self
    }
}

#[async_trait]
impl Metric for ResponseRelevancyMetric {
    fn name(&self) -> &str {
        "response_relevancy"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::UserInput, SampleField::Response],
        )
    }

    /// Faithful port of ragas' ResponseRelevancy / AnswerRelevancy:
    ///   1. ask the LLM to generate `strictness` questions the RESPONSE answers,
    ///      each tagged with a `noncommittal` flag (1 when the response is evasive);
    ///   2. embed the original user_input and every generated question, then
    ///      score = mean cosine(user_input, generated_question_i).
    /// If any generated item is noncommittal the score collapses to 0.0, and when
    /// no questions are generated the score is NaN (matching ragas' 0/0 behaviour).
    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let questions = self.generate_questions(sample).await?;
        if questions.is_empty() {
            // No artificial questions -> undefined relevancy (ragas returns NaN).
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(f64::NAN))
                    .with_reason("no questions could be generated from the response"),
            );
        }

        if questions.iter().any(|question| question.noncommittal == 1) {
            // An evasive / non-committal answer is not relevant regardless of similarity.
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(0.0))
                    .with_reason("response is noncommittal"),
            );
        }

        let mut input = Vec::with_capacity(questions.len() + 1);
        input.push(sample.user_input.clone());
        input.extend(questions.iter().map(|question| question.question.clone()));
        let response = self.embedding.embed(EmbeddingRequest { input }).await?;
        if response.embeddings.len() != questions.len() + 1 {
            return Err(RagasError::Parse {
                message: "response relevancy embedding count mismatch".to_string(),
            });
        }

        let user_input_embedding = &response.embeddings[0];
        let total = questions.len();
        let similarity_sum: f64 = response
            .embeddings
            .iter()
            .skip(1)
            .map(|question_embedding| cosine_similarity(user_input_embedding, question_embedding))
            .sum();
        let score = similarity_sum / total as f64;

        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "mean cosine similarity between user_input and {total} generated question(s)"
            )),
        )
    }
}

impl ResponseRelevancyMetric {
    /// Step 1 — ask the LLM to reverse-engineer the questions the response answers.
    async fn generate_questions(
        &self,
        sample: &SingleTurnSample,
    ) -> Result<Vec<GeneratedQuestion>, RagasError> {
        let context_block = if sample.retrieved_contexts.is_empty() {
            String::new()
        } else {
            format!("\nCONTEXT:\n{}", sample.retrieved_contexts.join("\n"))
        };
        let prompt = format!(
            "Generate {n} question(s) that the RESPONSE below answers. \
For each question set \"noncommittal\" to 1 when the response is evasive, vague, \
or non-committal (for example 'I don't know' or 'I am not sure'), otherwise 0. \
Return only JSON of the form \
{{\"questions\": [{{\"question\": \"...\", \"noncommittal\": 0}}]}} \
with exactly {n} entries.\n\nRESPONSE: {response}{context_block}",
            n = self.strictness,
            response = sample.response,
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: QuestionGenerationOutput =
            parse_json(&response.content, "response relevancy question generation")?;
        Ok(parsed
            .questions
            .into_iter()
            .filter(|question| !question.question.trim().is_empty())
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct QuestionGenerationOutput {
    #[serde(default)]
    questions: Vec<GeneratedQuestion>,
}

#[derive(Debug, Deserialize)]
struct GeneratedQuestion {
    question: String,
    #[serde(default)]
    noncommittal: i64,
}

pub struct ContextPrecisionMetric {
    llm: Arc<dyn LlmProvider>,
}

impl ContextPrecisionMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Metric for ContextPrecisionMetric {
    fn name(&self) -> &str {
        "context_precision"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![
                SampleField::UserInput,
                SampleField::Reference,
                SampleField::RetrievedContexts,
            ],
        )
    }

    /// Faithful port of ragas' LLMContextPrecisionWithReference:
    ///   for each retrieved context, ask the LLM whether that context was useful to
    ///   arrive at the REFERENCE answer for the user's question (verdict 1 = useful);
    ///   then compute Average Precision@k over the contexts in their retrieval order:
    ///   precision@k = (relevant among the first k) / k, and
    ///   AP = sum_k (precision@k * verdict_k) / (total relevant contexts).
    /// Returns 0.0 when no context is relevant (or there are no contexts).
    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = match sample.reference.as_deref() {
            Some(reference) if !reference.trim().is_empty() => reference,
            _ => {
                return Err(RagasError::Parse {
                    message: "context precision requires a non-empty reference".to_string(),
                });
            }
        };

        if sample.retrieved_contexts.is_empty() {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(0.0))
                    .with_reason("sample has no retrieved contexts"),
            );
        }

        let verdicts = self.verify_contexts(sample, reference).await?;

        let total_relevant = verdicts.iter().filter(|verdict| **verdict == 1).count();
        if total_relevant == 0 {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(0.0))
                    .with_reason("no retrieved context was useful to reach the reference"),
            );
        }

        let mut relevant_seen = 0usize;
        let mut precision_sum = 0.0f64;
        for (index, verdict) in verdicts.iter().enumerate() {
            if *verdict == 1 {
                relevant_seen += 1;
                let rank = index + 1;
                // precision@k weighted by verdict_k (only relevant ranks contribute).
                precision_sum += relevant_seen as f64 / rank as f64;
            }
        }
        let score = precision_sum / total_relevant as f64;

        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "average precision@k over {total_relevant}/{} useful context(s)",
                verdicts.len()
            )),
        )
    }
}

impl ContextPrecisionMetric {
    /// Ask the LLM, once per retrieved context, whether the context helped arrive at the
    /// reference answer. Verdicts are returned in retrieval order (one per context).
    async fn verify_contexts(
        &self,
        sample: &SingleTurnSample,
        reference: &str,
    ) -> Result<Vec<i64>, RagasError> {
        let mut verdicts = Vec::with_capacity(sample.retrieved_contexts.len());
        for context in &sample.retrieved_contexts {
            let prompt = format!(
                "Given a QUESTION, an ANSWER, and a CONTEXT, decide whether the context was \
useful to arrive at the answer. Use verdict 1 when the context is useful, 0 otherwise. \
Return only JSON of the form {{\"verdict\": 0, \"reason\": \"...\"}}.\n\n\
QUESTION: {}\nANSWER: {reference}\nCONTEXT: {context}",
                sample.user_input,
            );
            let response = self
                .llm
                .generate(LlmRequest {
                    messages: vec![ChatMessage::user(prompt)],
                    temperature: Some(0.0),
                })
                .await?;
            let parsed: ContextPrecisionVerdict =
                parse_json(&response.content, "context precision verdict")?;
            verdicts.push(parsed.verdict);
        }
        Ok(verdicts)
    }
}

#[derive(Debug, Deserialize)]
struct ContextPrecisionVerdict {
    #[serde(default)]
    verdict: i64,
}

pub struct LlmContextRecallMetric {
    llm: Arc<dyn LlmProvider>,
}

impl LlmContextRecallMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Metric for LlmContextRecallMetric {
    fn name(&self) -> &str {
        "context_recall"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![
                SampleField::UserInput,
                SampleField::Reference,
                SampleField::RetrievedContexts,
            ],
        )
    }

    /// Faithful port of ragas' LLMContextRecall:
    ///   1. split the REFERENCE answer into ordered sentences;
    ///   2. ask the LLM, in one call, to classify each sentence as attributable to the
    ///      retrieved contexts (verdict 1) or not (0), one classification per sentence in
    ///      order;
    ///   score = attributed_count / total_sentences.
    /// An empty reference (zero sentences) yields NaN (ragas' 0/0 behaviour).
    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = match sample.reference.as_deref() {
            Some(reference) if !reference.trim().is_empty() => reference,
            _ => {
                return Err(RagasError::Parse {
                    message: "context recall requires a non-empty reference".to_string(),
                });
            }
        };

        let sentences = split_into_sentences(reference);
        if sentences.is_empty() {
            // No sentences -> undefined recall (ragas returns NaN for 0/0).
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(f64::NAN))
                    .with_reason("reference contained no sentences to classify"),
            );
        }

        let classifications = self.classify_sentences(sample, &sentences).await?;
        let attributed = classifications
            .iter()
            .filter(|classification| classification.verdict == 1)
            .count();
        let total = sentences.len();
        let score = attributed as f64 / total as f64;

        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "{attributed}/{total} reference sentence(s) attributable to the retrieved contexts"
            )),
        )
    }
}

impl LlmContextRecallMetric {
    /// Ask the LLM, in a single call, to classify every reference sentence against the
    /// retrieved contexts. Classifications are returned in the same order as the sentences.
    async fn classify_sentences(
        &self,
        sample: &SingleTurnSample,
        sentences: &[String],
    ) -> Result<Vec<ContextRecallClassification>, RagasError> {
        let numbered = sentences
            .iter()
            .enumerate()
            .map(|(index, sentence)| format!("{}. {sentence}", index + 1))
            .collect::<Vec<_>>()
            .join("\n");
        let prompt = format!(
            "Given a QUESTION, the retrieved CONTEXT, and an ANSWER split into numbered \
sentences, decide for each sentence whether it can be attributed to the context. \
Use verdict 1 when the sentence is supported by the context, 0 otherwise. \
Return only JSON of the form \
{{\"classifications\": [{{\"verdict\": 0, \"reason\": \"...\"}}]}} \
with exactly one entry per sentence, in the same order.\n\n\
QUESTION: {}\n\nCONTEXT:\n{}\n\nANSWER SENTENCES:\n{numbered}",
            sample.user_input,
            sample.retrieved_contexts.join("\n"),
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: ContextRecallOutput =
            parse_json(&response.content, "context recall classification")?;
        Ok(parsed.classifications)
    }
}

/// Split a block of text into sentences on sentence-ending punctuation (`.`, `!`, `?`).
/// The terminator stays with its sentence; runs of whitespace and empty fragments are
/// dropped. This is a deliberately simple heuristic (it does not handle abbreviations).
fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_string());
            }
            current.clear();
        }
    }
    // Trailing text with no terminating punctuation still counts as a sentence.
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        sentences.push(trimmed.to_string());
    }
    sentences
}

#[derive(Debug, Deserialize)]
struct ContextRecallOutput {
    #[serde(default)]
    classifications: Vec<ContextRecallClassification>,
}

#[derive(Debug, Deserialize)]
struct ContextRecallClassification {
    #[serde(default)]
    verdict: i64,
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> f64 {
    let len = left.len().min(right.len());
    if len == 0 {
        return 0.0;
    }

    let mut dot = 0.0f64;
    let mut left_norm = 0.0f64;
    let mut right_norm = 0.0f64;
    for index in 0..len {
        let l = left[index] as f64;
        let r = right[index] as f64;
        dot += l * r;
        left_norm += l * l;
        right_norm += r * r;
    }

    if left_norm == 0.0 || right_norm == 0.0 {
        return 0.0;
    }

    dot / (left_norm.sqrt() * right_norm.sqrt())
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

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(self.name(), Vec::new())
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError>;
}

pub struct FnMetric<F>
where
    F: Fn(&SingleTurnSample) -> BoxMetricFuture + Send + Sync,
{
    name: String,
    scorer: F,
    required_fields: Vec<SampleField>,
}

impl<F> FnMetric<F>
where
    F: Fn(&SingleTurnSample) -> BoxMetricFuture + Send + Sync,
{
    pub fn new(name: impl Into<String>, scorer: F) -> Self {
        Self {
            name: name.into(),
            scorer,
            required_fields: Vec::new(),
        }
    }

    pub fn with_required_fields(mut self, fields: Vec<SampleField>) -> Self {
        self.required_fields = fields;
        self
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

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(&self.name, self.required_fields.clone())
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        (self.scorer)(sample).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, VecDeque};
    use std::sync::Mutex;

    use crate::{EmbeddingResponse, LlmResponse};

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
        let success = MetricResult::success("faithfulness", MetricValue::numeric(0.8))
            .with_reason("grounded");

        assert_eq!(success.metric_name, "faithfulness");
        assert_eq!(
            success.value.and_then(|value| value.as_numeric()),
            Some(0.8)
        );
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

    /// Mock LLM that replays scripted responses in order and records the prompts it saw,
    /// so multi-step pipelines can be driven deterministically without a network call.
    struct ScriptedLlm {
        responses: Mutex<VecDeque<String>>,
        prompts: Mutex<Vec<String>>,
    }

    impl ScriptedLlm {
        fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: Mutex::new(responses.into_iter().map(str::to_string).collect()),
                prompts: Mutex::new(Vec::new()),
            }
        }

        fn prompts(&self) -> Vec<String> {
            self.prompts.lock().expect("prompts").clone()
        }
    }

    #[async_trait]
    impl LlmProvider for ScriptedLlm {
        async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, RagasError> {
            let prompt = request
                .messages
                .iter()
                .map(|message| message.content.clone())
                .collect::<Vec<_>>()
                .join("\n");
            self.prompts.lock().expect("prompts").push(prompt);
            let content =
                self.responses
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

    fn faithfulness_sample() -> SingleTurnSample {
        SingleTurnSample::new(
            "What is Ragas and who maintains it?",
            "Ragas evaluates LLM applications. It is maintained by Exploding Gradients.",
            vec!["Ragas is a framework to evaluate LLM applications.".to_string()],
        )
    }

    fn numeric(result: &MetricResult) -> f64 {
        result
            .value
            .clone()
            .and_then(|value| value.as_numeric())
            .expect("numeric metric value")
    }

    struct MapEmbeddingProvider {
        vectors: HashMap<String, Vec<f32>>,
    }

    #[async_trait]
    impl EmbeddingProvider for MapEmbeddingProvider {
        async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
            let embeddings = request
                .input
                .iter()
                .map(|input| self.vectors.get(input).cloned().unwrap_or_default())
                .collect();
            Ok(EmbeddingResponse {
                embeddings,
                usage: None,
            })
        }
    }

    /// Embedding mock that must never be called; used to prove a pipeline short-circuits
    /// before reaching the embedding step.
    struct PanicEmbeddingProvider;

    #[async_trait]
    impl EmbeddingProvider for PanicEmbeddingProvider {
        async fn embed(&self, _request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
            panic!("embedding provider must not be called");
        }
    }

    #[tokio::test]
    async fn faithfulness_runs_two_step_pipeline_and_scores_supported_ratio() {
        // Step 1 extracts 2 statements; step 2 supports both -> 2/2 = 1.0.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"statements":["Ragas evaluates LLM applications.","Ragas is maintained by Exploding Gradients."]}"#,
            r#"{"verdicts":[{"statement":"Ragas evaluates LLM applications.","verdict":1,"reason":"stated in context"},{"statement":"Ragas is maintained by Exploding Gradients.","verdict":1,"reason":"stated in context"}]}"#,
        ]));
        let metric = FaithfulnessMetric::new(llm.clone());

        let result = metric.score(&faithfulness_sample()).await.expect("faithfulness");

        assert_eq!(result.metric_name, "faithfulness");
        assert_eq!(numeric(&result), 1.0);

        // The pipeline made exactly two LLM calls and chained step 1's output into step 2's prompt.
        let prompts = llm.prompts();
        assert_eq!(prompts.len(), 2);
        assert!(prompts[0].contains("ANSWER"));
        assert!(prompts[1].contains("CONTEXT"));
        assert!(prompts[1].contains("Ragas is maintained by Exploding Gradients."));
    }

    #[tokio::test]
    async fn faithfulness_discriminates_partial_and_unsupported_responses() {
        // 1 of 2 statements supported -> 0.5.
        let partial = FaithfulnessMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"statements":["a","b"]}"#,
            r#"{"verdicts":[{"verdict":1},{"verdict":0}]}"#,
        ])));
        assert_eq!(
            numeric(&partial.score(&faithfulness_sample()).await.expect("partial")),
            0.5
        );

        // Nothing supported -> 0.0 (an unfaithful answer must score low).
        let unsupported = FaithfulnessMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"statements":["a","b"]}"#,
            r#"{"verdicts":[{"verdict":0},{"verdict":0}]}"#,
        ])));
        assert_eq!(
            numeric(&unsupported.score(&faithfulness_sample()).await.expect("unsupported")),
            0.0
        );
    }

    #[tokio::test]
    async fn faithfulness_empty_statements_is_nan_and_skips_verification() {
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"statements":[]}"#]));
        let metric = FaithfulnessMetric::new(llm.clone());

        let result = metric.score(&faithfulness_sample()).await.expect("empty");

        assert!(numeric(&result).is_nan());
        // No statements -> the verification step must not run (only one LLM call).
        assert_eq!(llm.prompts().len(), 1);
    }

    #[tokio::test]
    async fn faithfulness_repairs_fenced_json_from_the_model() {
        // Both steps wrap their JSON in markdown fences and prose; the repair path must recover it.
        let metric = FaithfulnessMetric::new(Arc::new(ScriptedLlm::new(vec![
            "Sure! Here you go:\n```json\n{\"statements\":[\"a\"]}\n```",
            "```\n{\"verdicts\":[{\"verdict\":1}]}\n```",
        ])));

        let result = metric.score(&faithfulness_sample()).await.expect("repaired");

        assert_eq!(numeric(&result), 1.0);
    }

    #[tokio::test]
    async fn faithfulness_surfaces_unparseable_model_output_as_error() {
        let metric = FaithfulnessMetric::new(Arc::new(ScriptedLlm::new(vec![
            "I cannot break this into statements.",
        ])));

        let error = metric
            .score(&faithfulness_sample())
            .await
            .expect_err("malformed statement output");

        assert!(error.to_string().contains("statement generation"));
    }

    fn relevancy_sample() -> SingleTurnSample {
        SingleTurnSample::new(
            "user-question",
            "Albert Einstein was born in Germany.",
            vec!["Albert Einstein was a German-born theoretical physicist.".to_string()],
        )
    }

    #[tokio::test]
    async fn response_relevancy_scores_high_when_generated_questions_match_user_input() {
        // Step 1: LLM reverse-engineers 2 committal questions.
        // Step 2: both generated questions embed identically to the user_input -> mean cosine ~ 1.0.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"questions":[{"question":"q-match","noncommittal":0},{"question":"q-match","noncommittal":0}]}"#,
        ]));
        let metric = ResponseRelevancyMetric::new(
            llm.clone(),
            Arc::new(MapEmbeddingProvider {
                vectors: HashMap::from([
                    ("user-question".to_string(), vec![1.0, 0.0]),
                    ("q-match".to_string(), vec![1.0, 0.0]),
                ]),
            }),
        );

        let result = metric.score(&relevancy_sample()).await.expect("relevancy");

        assert!((numeric(&result) - 1.0).abs() < 1e-9);
        // The question-generation prompt is conditioned on the response (not a single 0-1 ask).
        let prompts = llm.prompts();
        assert_eq!(prompts.len(), 1);
        assert!(prompts[0].contains("Albert Einstein was born in Germany."));
    }

    #[tokio::test]
    async fn response_relevancy_discriminates_off_topic_questions() {
        // Generated questions are orthogonal to the user_input -> mean cosine collapses toward 0.
        let metric = ResponseRelevancyMetric::new(
            Arc::new(ScriptedLlm::new(vec![
                r#"{"questions":[{"question":"q-off","noncommittal":0},{"question":"q-off","noncommittal":0}]}"#,
            ])),
            Arc::new(MapEmbeddingProvider {
                vectors: HashMap::from([
                    ("user-question".to_string(), vec![1.0, 0.0]),
                    ("q-off".to_string(), vec![0.0, 1.0]),
                ]),
            }),
        );

        let result = metric.score(&relevancy_sample()).await.expect("relevancy");

        assert!(numeric(&result).abs() < 1e-9);
    }

    #[tokio::test]
    async fn response_relevancy_noncommittal_item_forces_zero() {
        // Even when the (single) generated question embeds perfectly, a noncommittal flag forces 0.0.
        let embedding = Arc::new(MapEmbeddingProvider {
            vectors: HashMap::from([
                ("user-question".to_string(), vec![1.0, 0.0]),
                ("q-evasive".to_string(), vec![1.0, 0.0]),
            ]),
        });
        let metric = ResponseRelevancyMetric::new(
            Arc::new(ScriptedLlm::new(vec![
                r#"{"questions":[{"question":"q-evasive","noncommittal":1}]}"#,
            ])),
            embedding,
        );

        let result = metric.score(&relevancy_sample()).await.expect("relevancy");

        assert_eq!(numeric(&result), 0.0);
        assert_eq!(result.reason.as_deref(), Some("response is noncommittal"));
    }

    #[tokio::test]
    async fn response_relevancy_empty_questions_is_nan_and_skips_embedding() {
        // The embedding provider would panic if asked to embed (its map is empty), proving
        // that zero generated questions short-circuits to NaN before any embedding call.
        let llm = Arc::new(ScriptedLlm::new(vec![r#"{"questions":[]}"#]));
        let metric = ResponseRelevancyMetric::new(
            llm.clone(),
            Arc::new(PanicEmbeddingProvider),
        );

        let result = metric.score(&relevancy_sample()).await.expect("relevancy");

        assert!(numeric(&result).is_nan());
        // Exactly one LLM call (generation) and no embedding call.
        assert_eq!(llm.prompts().len(), 1);
    }

    #[tokio::test]
    async fn response_relevancy_repairs_fenced_json_from_the_model() {
        // The model wraps its JSON in a markdown fence + prose; the repair path must recover it.
        let metric = ResponseRelevancyMetric::new(
            Arc::new(ScriptedLlm::new(vec![
                "Here are the questions:\n```json\n{\"questions\":[{\"question\":\"q-match\",\"noncommittal\":0}]}\n```",
            ])),
            Arc::new(MapEmbeddingProvider {
                vectors: HashMap::from([
                    ("user-question".to_string(), vec![1.0, 0.0]),
                    ("q-match".to_string(), vec![1.0, 0.0]),
                ]),
            }),
        );

        let result = metric.score(&relevancy_sample()).await.expect("relevancy");

        assert!((numeric(&result) - 1.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn response_relevancy_surfaces_unparseable_model_output_as_error() {
        let metric = ResponseRelevancyMetric::new(
            Arc::new(ScriptedLlm::new(vec![
                "I cannot turn this into questions.",
            ])),
            Arc::new(PanicEmbeddingProvider),
        );

        let error = metric
            .score(&relevancy_sample())
            .await
            .expect_err("malformed question output");

        assert!(error.to_string().contains("question generation"));
    }

    fn context_precision_sample(contexts: Vec<&str>) -> SingleTurnSample {
        SingleTurnSample::new(
            "What is Ragas and who maintains it?",
            "answer",
            contexts.into_iter().map(str::to_string).collect(),
        )
        .with_reference("Ragas evaluates LLM applications and is maintained by Exploding Gradients.")
    }

    #[tokio::test]
    async fn test_4_1_4_context_precision_computes_average_precision_at_k() {
        // SCEN-4.1.4 / AC4 / TEST-4.1.4
        // Per-context LLM verdicts in retrieval order: [1, 0, 1, 1].
        // precision@k contributes only at relevant ranks 1, 3, 4:
        //   AP = (1/1 + 2/3 + 3/4) / 3 (3 = total relevant contexts).
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"verdict":1,"reason":"directly answers"}"#,
            r#"{"verdict":0,"reason":"off topic"}"#,
            r#"{"verdict":1,"reason":"supports the answer"}"#,
            r#"{"verdict":1,"reason":"supports the answer"}"#,
        ]));
        let metric = ContextPrecisionMetric::new(llm.clone());
        let sample = context_precision_sample(vec!["ctx-a", "ctx-b", "ctx-c", "ctx-d"]);

        let result = metric.score(&sample).await.expect("context precision");

        assert_eq!(result.metric_name, "context_precision");
        assert!((numeric(&result) - (1.0 + 2.0 / 3.0 + 3.0 / 4.0) / 3.0).abs() < 1e-9);

        // One LLM call per context, each conditioned on the reference answer (not a single ask).
        let prompts = llm.prompts();
        assert_eq!(prompts.len(), 4);
        assert!(prompts[0].contains("ANSWER: Ragas evaluates LLM applications"));
        assert!(prompts[0].contains("CONTEXT: ctx-a"));
        assert!(prompts[2].contains("CONTEXT: ctx-c"));
    }

    #[tokio::test]
    async fn context_precision_all_irrelevant_contexts_score_zero() {
        // Every context judged useless -> no relevant contexts -> AP = 0.0 (discrimination: bad sample low).
        let metric = ContextPrecisionMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"verdict":0,"reason":"unrelated"}"#,
            r#"{"verdict":0,"reason":"unrelated"}"#,
        ])));
        let sample = context_precision_sample(vec!["ctx-a", "ctx-b"]);

        let result = metric.score(&sample).await.expect("context precision");

        assert_eq!(numeric(&result), 0.0);
    }

    #[tokio::test]
    async fn context_precision_empty_contexts_score_zero_without_calling_llm() {
        // No contexts -> 0.0, and the LLM must not be invoked.
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let metric = ContextPrecisionMetric::new(llm.clone());
        let sample = SingleTurnSample::new("question", "answer", Vec::new())
            .with_reference("the reference answer");

        let result = metric.score(&sample).await.expect("context precision");

        assert_eq!(numeric(&result), 0.0);
        assert!(llm.prompts().is_empty());
    }

    #[tokio::test]
    async fn context_precision_missing_reference_is_an_error_without_calling_llm() {
        // No reference -> validation error before any LLM call.
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let metric = ContextPrecisionMetric::new(llm.clone());
        // SingleTurnSample::new leaves `reference` as None.
        let sample = SingleTurnSample::new("question", "answer", vec!["ctx-a".to_string()]);

        let error = metric
            .score(&sample)
            .await
            .expect_err("missing reference");

        assert!(error.to_string().contains("reference"));
        assert!(llm.prompts().is_empty());
    }

    #[tokio::test]
    async fn context_precision_repairs_fenced_json_from_the_model() {
        // The model wraps its verdict JSON in a markdown fence + prose; the repair path recovers it.
        let metric = ContextPrecisionMetric::new(Arc::new(ScriptedLlm::new(vec![
            "Sure:\n```json\n{\"verdict\":1,\"reason\":\"useful\"}\n```",
        ])));
        let sample = context_precision_sample(vec!["ctx-a"]);

        let result = metric.score(&sample).await.expect("context precision");

        // Single relevant context at rank 1 -> AP = (1/1) / 1 = 1.0.
        assert_eq!(numeric(&result), 1.0);
    }

    #[tokio::test]
    async fn context_precision_surfaces_unparseable_model_output_as_error() {
        let metric = ContextPrecisionMetric::new(Arc::new(ScriptedLlm::new(vec![
            "This context is somewhat related I think.",
        ])));
        let sample = context_precision_sample(vec!["ctx-a"]);

        let error = metric
            .score(&sample)
            .await
            .expect_err("malformed verdict output");

        assert!(error.to_string().contains("context precision verdict"));
    }

    fn context_recall_sample() -> SingleTurnSample {
        SingleTurnSample::new(
            "What is Ragas and who maintains it?",
            "answer",
            vec![
                "Ragas is a framework to evaluate LLM applications.".to_string(),
                "Ragas is maintained by Exploding Gradients.".to_string(),
            ],
        )
        // Two sentences -> two classifications.
        .with_reference("Ragas evaluates LLM applications. Exploding Gradients maintains it.")
    }

    #[tokio::test]
    async fn context_recall_all_attributed_sentences_score_one() {
        // Both reference sentences attributable to the contexts -> 2/2 = 1.0.
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"classifications":[{"verdict":1,"reason":"stated in context"},{"verdict":1,"reason":"stated in context"}]}"#,
        ]));
        let metric = LlmContextRecallMetric::new(llm.clone());

        let result = metric.score(&context_recall_sample()).await.expect("recall");

        assert_eq!(result.metric_name, "context_recall");
        assert_eq!(numeric(&result), 1.0);

        // A single classification call conditioned on the contexts and the ordered sentences.
        let prompts = llm.prompts();
        assert_eq!(prompts.len(), 1);
        assert!(prompts[0].contains("CONTEXT"));
        assert!(prompts[0].contains("1. Ragas evaluates LLM applications."));
        assert!(prompts[0].contains("2. Exploding Gradients maintains it."));
    }

    #[tokio::test]
    async fn context_recall_discriminates_partial_and_unattributed_references() {
        // 1 of 2 sentences attributed -> 0.5.
        let half = LlmContextRecallMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"classifications":[{"verdict":1},{"verdict":0}]}"#,
        ])));
        assert_eq!(
            numeric(&half.score(&context_recall_sample()).await.expect("half")),
            0.5
        );

        // Nothing attributed -> 0.0 (a reference unsupported by context must score low).
        let none = LlmContextRecallMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"classifications":[{"verdict":0},{"verdict":0}]}"#,
        ])));
        assert_eq!(
            numeric(&none.score(&context_recall_sample()).await.expect("none")),
            0.0
        );
    }

    #[tokio::test]
    async fn context_recall_empty_reference_is_nan_without_calling_llm() {
        // A whitespace-only reference yields zero sentences -> NaN, and the LLM must not run.
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let metric = LlmContextRecallMetric::new(llm.clone());
        let sample =
            SingleTurnSample::new("question", "answer", vec!["ctx".to_string()]).with_reference("   ");

        let error = metric.score(&sample).await.expect_err("empty reference");

        assert!(error.to_string().contains("reference"));
        assert!(llm.prompts().is_empty());
    }

    #[tokio::test]
    async fn context_recall_repairs_fenced_json_from_the_model() {
        // The model wraps its JSON in a markdown fence + prose; the repair path recovers it.
        let metric = LlmContextRecallMetric::new(Arc::new(ScriptedLlm::new(vec![
            "Sure:\n```json\n{\"classifications\":[{\"verdict\":1},{\"verdict\":1}]}\n```",
        ])));

        let result = metric.score(&context_recall_sample()).await.expect("repaired");

        assert_eq!(numeric(&result), 1.0);
    }

    #[tokio::test]
    async fn context_recall_surfaces_unparseable_model_output_as_error() {
        let metric = LlmContextRecallMetric::new(Arc::new(ScriptedLlm::new(vec![
            "I think both sentences are kind of supported.",
        ])));

        let error = metric
            .score(&context_recall_sample())
            .await
            .expect_err("malformed classification output");

        assert!(error.to_string().contains("context recall classification"));
    }

    // ---------------------------------------------------------------------
    // ROUGE-L deterministic discrimination (NON-ignored: runs offline, no LLM).
    // This is the lexical baseline that does not need a model key, so it always
    // runs and proves the discrimination shape (faithful > adversarial).
    // ---------------------------------------------------------------------
    #[test]
    fn rouge_l_discriminates_overlapping_from_disjoint_candidate() {
        use crate::rouge_l_recall;

        let reference = "Ragas evaluates retrieval augmented generation applications.";
        // A candidate that closely tracks the reference shares a long subsequence.
        let faithful = "Ragas evaluates retrieval augmented generation applications.";
        // A candidate about an unrelated topic shares almost no subsequence.
        let adversarial = "The weather in Paris was cold and rainy yesterday.";

        let faithful_score = rouge_l_recall(faithful, reference)
            .score
            .expect("faithful rouge score");
        let adversarial_score = rouge_l_recall(adversarial, reference)
            .score
            .expect("adversarial rouge score");

        assert!(
            faithful_score > adversarial_score,
            "faithful={faithful_score} must exceed adversarial={adversarial_score}"
        );
        assert!((faithful_score - 1.0).abs() < 1e-9);
        assert_eq!(adversarial_score, 0.0);
    }

    // ---------------------------------------------------------------------
    // LIVE discrimination gate (IGNORED unless OPENAI_API_KEY is set).
    //
    // These are the project's real "done" gate for the LLM metrics: they call a
    // real provider and assert that a faithful/relevant sample scores STRICTLY
    // higher than an adversarial one. They are #[ignore]-attributed so they never
    // run in normal `cargo test`; run them with:
    //   OPENAI_API_KEY=... cargo test --lib -- --ignored
    // Until a real key has driven them, the honest status of the LLM metrics is
    // "math verified; real-LLM UNVERIFIED".
    // ---------------------------------------------------------------------
    use crate::OpenAiCompatibleClient;

    /// Build the live client from the environment, or `None` to skip when no key.
    fn live_client() -> Option<Arc<OpenAiCompatibleClient>> {
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        OpenAiCompatibleClient::from_env(model).map(Arc::new)
    }

    /// Build the live embedding client. Embeddings may live on a DIFFERENT provider
    /// than chat (e.g. DeepSeek chat + SiliconFlow embeddings): if
    /// `OPENAI_EMBEDDING_BASE_URL` and `OPENAI_EMBEDDING_API_KEY` are both set they are
    /// used, otherwise it falls back to the same endpoint/key as chat (`from_env`).
    fn live_embedding_client() -> Option<Arc<OpenAiCompatibleClient>> {
        let model =
            std::env::var("OPENAI_EMBEDDING_MODEL").unwrap_or_else(|_| "text-embedding-3-small".to_string());
        match (
            std::env::var("OPENAI_EMBEDDING_BASE_URL"),
            std::env::var("OPENAI_EMBEDDING_API_KEY"),
        ) {
            (Ok(base_url), Ok(api_key))
                if !base_url.trim().is_empty() && !api_key.trim().is_empty() =>
            {
                Some(Arc::new(OpenAiCompatibleClient::new(base_url, api_key, model)))
            }
            _ => OpenAiCompatibleClient::from_env(model).map(Arc::new),
        }
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_faithfulness_scores_faithful_above_unfaithful() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = FaithfulnessMetric::new(client);

        let contexts =
            vec!["Ragas is an open-source framework to evaluate LLM applications.".to_string()];
        let faithful = SingleTurnSample::new(
            "What is Ragas?",
            "Ragas is a framework for evaluating LLM applications.",
            contexts.clone(),
        );
        let unfaithful = SingleTurnSample::new(
            "What is Ragas?",
            "Ragas is a brand of running shoes founded in 1962 in Italy.",
            contexts,
        );

        let faithful_score = numeric(&metric.score(&faithful).await.expect("faithful"));
        let unfaithful_score = numeric(&metric.score(&unfaithful).await.expect("unfaithful"));

        assert!(
            faithful_score > unfaithful_score,
            "faithful={faithful_score} must exceed unfaithful={unfaithful_score}"
        );
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_answer_relevancy_scores_relevant_above_irrelevant() {
        let (Some(llm), Some(embedding)) = (live_client(), live_embedding_client()) else {
            return;
        };
        let metric = ResponseRelevancyMetric::new(llm, embedding);

        let relevant = SingleTurnSample::new(
            "Where was Albert Einstein born?",
            "Albert Einstein was born in Ulm, Germany.",
            Vec::new(),
        );
        let irrelevant = SingleTurnSample::new(
            "Where was Albert Einstein born?",
            "Photosynthesis converts sunlight into chemical energy in plants.",
            Vec::new(),
        );

        let relevant_score = numeric(&metric.score(&relevant).await.expect("relevant"));
        let irrelevant_score = numeric(&metric.score(&irrelevant).await.expect("irrelevant"));

        assert!(
            relevant_score > irrelevant_score,
            "relevant={relevant_score} must exceed irrelevant={irrelevant_score}"
        );
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_context_precision_ranks_useful_context_above_useless() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = ContextPrecisionMetric::new(client);

        let reference = "Ragas is a framework to evaluate LLM applications.";
        let useful_first = SingleTurnSample::new(
            "What is Ragas?",
            "answer",
            vec![
                "Ragas is a framework to evaluate LLM applications.".to_string(),
                "The Eiffel Tower is located in Paris, France.".to_string(),
            ],
        )
        .with_reference(reference);
        // Same contexts but with the useful one buried at the bottom -> lower AP@k.
        let useful_last = SingleTurnSample::new(
            "What is Ragas?",
            "answer",
            vec![
                "The Eiffel Tower is located in Paris, France.".to_string(),
                "Ragas is a framework to evaluate LLM applications.".to_string(),
            ],
        )
        .with_reference(reference);

        let first_score = numeric(&metric.score(&useful_first).await.expect("useful first"));
        let last_score = numeric(&metric.score(&useful_last).await.expect("useful last"));

        assert!(
            first_score > last_score,
            "useful-first={first_score} must exceed useful-last={last_score}"
        );
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_context_recall_scores_supported_above_unsupported() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = LlmContextRecallMetric::new(client);

        let contexts = vec![
            "Ragas is a framework to evaluate LLM applications.".to_string(),
            "Ragas is maintained by the Exploding Gradients team.".to_string(),
        ];
        let supported = SingleTurnSample::new("What is Ragas?", "answer", contexts.clone())
            .with_reference(
                "Ragas evaluates LLM applications. Exploding Gradients maintains it.",
            );
        let unsupported = SingleTurnSample::new("What is Ragas?", "answer", contexts)
            .with_reference("Mount Everest is the tallest mountain. The ocean is deep and blue.");

        let supported_score = numeric(&metric.score(&supported).await.expect("supported"));
        let unsupported_score = numeric(&metric.score(&unsupported).await.expect("unsupported"));

        assert!(
            supported_score > unsupported_score,
            "supported={supported_score} must exceed unsupported={unsupported_score}"
        );
    }
}
