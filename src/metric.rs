use std::collections::{BTreeMap, BTreeSet};
use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::validation::{MetricRequirements, SampleField};
use crate::{
    AnswerCorrectnessWeights, ChatMessage, EmbeddingProvider, EmbeddingRequest,
    FactualCorrectnessCounts, LlmProvider, LlmRequest, RagasError, SemanticThresholdPolicy,
    SingleTurnSample, answer_correctness, factual_correctness, semantic_similarity_from_vectors,
    threshold_semantic_similarity,
};

pub type BoxMetricFuture = Pin<Box<dyn Future<Output = Result<MetricResult, RagasError>> + Send>>;

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Discrete(String),
    Numeric(f64),
    Ranking(Vec<RankingItem>),
}

// `MetricValue` uses serde's default externally-tagged enum shape
// (`{"Numeric": 0.5}`, `{"Discrete": "pass"}`, `{"Ranking": [...]}`).
//
// A non-finite `f64` (NaN / +-Inf) — e.g. Faithfulness over an empty statement
// set returns 0/0 = NaN — is rendered as JSON `null` (`{"Numeric": null}`),
// matching Python `json`/ragas. The in-memory value is left untouched; only the
// JSON output becomes null.
//
// Note on serialization: serde_json's *own* default already emits `null` for a
// non-finite float, so the derived `Serialize` did not error. This explicit impl
// makes that behavior intentional and serializer-independent (a stricter
// serializer that would reject NaN still receives `null` from us), and — more
// importantly — pairs with the custom `Deserialize` below so the value can be
// read back. With the derived `Deserialize`, `{"Numeric": null}` failed with
// "invalid type: null, expected f64", so a NaN report serialized but could not
// round-trip.
//
// Round-trip: finite numerics serialize and deserialize byte-for-byte as before.
// `{"Numeric": null}` deserializes back to `MetricValue::Numeric(f64::NAN)`, so a
// non-finite score survives a JSON round-trip as NaN (not as a finite 0).
impl Serialize for MetricValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            MetricValue::Discrete(value) => {
                serializer.serialize_newtype_variant("MetricValue", 0, "Discrete", value)
            }
            MetricValue::Numeric(value) => {
                if value.is_finite() {
                    serializer.serialize_newtype_variant("MetricValue", 1, "Numeric", value)
                } else {
                    // Non-finite -> JSON null (mirrors Python json / ragas), and
                    // guarantees it even under serializers that would reject NaN.
                    serializer.serialize_newtype_variant(
                        "MetricValue",
                        1,
                        "Numeric",
                        &Option::<f64>::None,
                    )
                }
            }
            MetricValue::Ranking(items) => {
                serializer.serialize_newtype_variant("MetricValue", 2, "Ranking", items)
            }
        }
    }
}

impl<'de> Deserialize<'de> for MetricValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum Repr {
            Discrete(String),
            // `Option<f64>` so that `{"Numeric": null}` (a non-finite score that
            // was rendered as null on the way out) deserializes back to NaN.
            Numeric(Option<f64>),
            Ranking(Vec<RankingItem>),
        }

        Ok(match Repr::deserialize(deserializer)? {
            Repr::Discrete(value) => MetricValue::Discrete(value),
            Repr::Numeric(Some(value)) => MetricValue::Numeric(value),
            Repr::Numeric(None) => MetricValue::Numeric(f64::NAN),
            Repr::Ranking(items) => MetricValue::Ranking(items),
        })
    }
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
        let supported = verdicts
            .iter()
            .filter(|verdict| verdict.verdict == 1)
            .count();
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
pub(crate) fn parse_json<T: serde::de::DeserializeOwned>(
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

/// SemanticSimilarity (ragas' `answer_similarity`): embedding cosine similarity between the
/// response and the reference. Embedding-only — no LLM. Optional `threshold` binarizes the score.
pub struct SemanticSimilarityMetric {
    embedding: Arc<dyn EmbeddingProvider>,
    threshold: Option<f64>,
}

impl SemanticSimilarityMetric {
    pub fn new(embedding: Arc<dyn EmbeddingProvider>) -> Self {
        Self {
            embedding,
            threshold: None,
        }
    }

    /// Binarize the score: `1.0` when cosine `>= threshold` (inclusive), else `0.0`. Without a
    /// threshold the raw cosine (clamped to `[0, 1]`) is returned, matching ragas' default.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = Some(threshold);
        self
    }
}

#[async_trait]
impl Metric for SemanticSimilarityMetric {
    fn name(&self) -> &str {
        "semantic_similarity"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Response, SampleField::Reference],
        )
    }

    /// Faithful port of ragas' SemanticSimilarity: embed the response and the reference, take the
    /// cosine similarity of the two vectors (clamped to `[0, 1]` via
    /// [`semantic_similarity_from_vectors`]). With a positive `threshold`, the score is binarized;
    /// otherwise the raw cosine is returned. Requires a `reference`.
    ///
    /// Two ragas-faithful edge behaviors: an empty `response`/`reference` is coerced to a single
    /// space before embedding (ragas's `answer or " "`), and a `threshold` of `0.0` is treated as
    /// "no threshold" (ragas gates binarization on Python truthiness, `if self.threshold:`).
    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let Some(reference) = sample.reference.as_ref() else {
            return Err(RagasError::Parse {
                message: "semantic_similarity requires a reference".to_string(),
            });
        };

        // ragas coerces an empty string to " " before embedding (`answer or " "`), avoiding a
        // degenerate/zero embedding for "".
        let coerce = |text: &str| -> String {
            if text.is_empty() {
                " ".to_string()
            } else {
                text.to_string()
            }
        };
        let embedded = self
            .embedding
            .embed(EmbeddingRequest {
                input: vec![coerce(&sample.response), coerce(reference)],
            })
            .await?;
        if embedded.embeddings.len() != 2 {
            return Err(RagasError::Parse {
                message: "semantic_similarity expected two embeddings".to_string(),
            });
        }

        let raw =
            semantic_similarity_from_vectors(&embedded.embeddings[0], &embedded.embeddings[1])
                .score
                .ok_or_else(|| RagasError::Parse {
                    message: "semantic_similarity produced no score".to_string(),
                })?;

        // A non-positive threshold is treated as "no threshold" (ragas: `if self.threshold:`).
        let (score, reason) = match self.threshold {
            Some(threshold) if threshold > 0.0 => {
                let passed = threshold_semantic_similarity(
                    raw,
                    SemanticThresholdPolicy::inclusive(threshold),
                )
                .score
                .unwrap_or(0.0);
                (
                    passed,
                    format!("cosine {raw:.6} vs inclusive threshold {threshold:.3} -> {passed}"),
                )
            }
            _ => (raw, format!("embedding cosine similarity {raw:.6}")),
        };

        Ok(MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(reason))
    }
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

// ============================================================================
// Phase 1 metrics (docs/parity-roadmap.md) — LLM-judged variants that reuse the
// scaffolding above. Each is a real `impl Metric`, usable in `evaluate()`.
// ============================================================================

/// Average Precision@k over per-context relevance verdicts in retrieval order:
/// `precision@k` is accumulated only at relevant ranks and divided by the total
/// number of relevant contexts. Returns 0.0 when nothing is relevant.
fn average_precision_at_k(verdicts: &[i64]) -> f64 {
    let total_relevant = verdicts.iter().filter(|verdict| **verdict == 1).count();
    if total_relevant == 0 {
        return 0.0;
    }
    let mut relevant_seen = 0usize;
    let mut precision_sum = 0.0f64;
    for (index, verdict) in verdicts.iter().enumerate() {
        if *verdict == 1 {
            relevant_seen += 1;
            precision_sum += relevant_seen as f64 / (index + 1) as f64;
        }
    }
    precision_sum / total_relevant as f64
}

/// Append the optional CONTEXT / REFERENCE fields to a judge prompt when present.
fn optional_fields_block(sample: &SingleTurnSample) -> String {
    let mut block = String::new();
    if !sample.retrieved_contexts.is_empty() {
        block.push_str(&format!(
            "\nCONTEXT:\n{}",
            sample.retrieved_contexts.join("\n")
        ));
    }
    if let Some(reference) = sample.reference.as_deref()
        && !reference.trim().is_empty()
    {
        block.push_str(&format!("\nREFERENCE: {reference}"));
    }
    block
}

#[derive(Debug, Deserialize)]
struct BinaryVerdict {
    #[serde(default)]
    verdict: i64,
}

/// LLMContextPrecisionWithoutReference / ContextUtilization.
///
/// Like [`ContextPrecisionMetric`] but needs no ground-truth reference: each retrieved
/// context is judged for usefulness against the RESPONSE, then scored with Average
/// Precision@k in retrieval order. This is the common production case (no labels).
pub struct ContextUtilizationMetric {
    llm: Arc<dyn LlmProvider>,
}

impl ContextUtilizationMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }

    /// Ask the LLM, once per retrieved context, whether it helped reach the response.
    async fn verify_contexts(&self, sample: &SingleTurnSample) -> Result<Vec<i64>, RagasError> {
        let mut verdicts = Vec::with_capacity(sample.retrieved_contexts.len());
        for context in &sample.retrieved_contexts {
            let prompt = format!(
                "Given a QUESTION, an ANSWER, and a CONTEXT, decide whether the context was \
useful to arrive at the answer. Use verdict 1 when the context is useful, 0 otherwise. \
Return only JSON of the form {{\"verdict\": 0, \"reason\": \"...\"}}.\n\n\
QUESTION: {}\nANSWER: {}\nCONTEXT: {context}",
                sample.user_input, sample.response,
            );
            let response = self
                .llm
                .generate(LlmRequest {
                    messages: vec![ChatMessage::user(prompt)],
                    temperature: Some(0.0),
                })
                .await?;
            let parsed: BinaryVerdict =
                parse_json(&response.content, "context utilization verdict")?;
            verdicts.push(parsed.verdict);
        }
        Ok(verdicts)
    }
}

#[async_trait]
impl Metric for ContextUtilizationMetric {
    fn name(&self) -> &str {
        "context_utilization"
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

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        if sample.retrieved_contexts.is_empty() {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(0.0))
                    .with_reason("sample has no retrieved contexts"),
            );
        }
        let verdicts = self.verify_contexts(sample).await?;
        let useful = verdicts.iter().filter(|verdict| **verdict == 1).count();
        Ok(MetricResult::success(
            self.name(),
            MetricValue::numeric(average_precision_at_k(&verdicts)),
        )
        .with_reason(format!(
            "average precision@k over {useful}/{} useful context(s)",
            verdicts.len()
        )))
    }
}

/// AspectCritic — a binary LLM judge over a user-defined CRITERION. With `strictness > 1`
/// the criterion is judged multiple times and the majority verdict is taken. Score is 1.0
/// when the submission meets the criterion, 0.0 otherwise.
pub struct AspectCriticMetric {
    llm: Arc<dyn LlmProvider>,
    name: String,
    definition: String,
    strictness: usize,
}

impl AspectCriticMetric {
    pub fn new(
        name: impl Into<String>,
        definition: impl Into<String>,
        llm: Arc<dyn LlmProvider>,
    ) -> Self {
        Self {
            llm,
            name: name.into(),
            definition: definition.into(),
            strictness: 1,
        }
    }

    pub fn with_strictness(mut self, strictness: usize) -> Self {
        self.strictness = strictness.max(1);
        self
    }

    async fn judge_once(&self, sample: &SingleTurnSample) -> Result<i64, RagasError> {
        let prompt = format!(
            "Evaluate whether the SUBMISSION meets the CRITERION. \
Use verdict 1 when it meets the criterion, 0 otherwise. \
Return only JSON of the form {{\"verdict\": 0, \"reason\": \"...\"}}.\n\n\
CRITERION: {}\n\nQUESTION: {}\nSUBMISSION: {}{}",
            self.definition,
            sample.user_input,
            sample.response,
            optional_fields_block(sample),
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: BinaryVerdict = parse_json(&response.content, "aspect critic verdict")?;
        Ok(parsed.verdict)
    }
}

#[async_trait]
impl Metric for AspectCriticMetric {
    fn name(&self) -> &str {
        &self.name
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::UserInput, SampleField::Response],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let mut positive = 0usize;
        for _ in 0..self.strictness {
            if self.judge_once(sample).await? == 1 {
                positive += 1;
            }
        }
        // Strict majority across `strictness` calls (ties resolve to 0).
        let verdict = if positive * 2 > self.strictness {
            1.0
        } else {
            0.0
        };
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(verdict)).with_reason(format!(
                "{positive}/{} judge call(s) returned a positive verdict",
                self.strictness
            )),
        )
    }
}

#[derive(Debug, Deserialize)]
struct SimpleCriteriaOutput {
    #[serde(default)]
    score: f64,
}

/// SimpleCriteriaScore — a single-call LLM judge that returns an integer score on a
/// user-defined scale (e.g. 0–5). Unlike AspectCritic the score is the raw rating, not a
/// normalized 0/1 (it is stored as-is).
pub struct SimpleCriteriaScoreMetric {
    llm: Arc<dyn LlmProvider>,
    name: String,
    definition: String,
}

impl SimpleCriteriaScoreMetric {
    pub fn new(
        name: impl Into<String>,
        definition: impl Into<String>,
        llm: Arc<dyn LlmProvider>,
    ) -> Self {
        Self {
            llm,
            name: name.into(),
            definition: definition.into(),
        }
    }
}

#[async_trait]
impl Metric for SimpleCriteriaScoreMetric {
    fn name(&self) -> &str {
        &self.name
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::UserInput, SampleField::Response],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let prompt = format!(
            "Score the SUBMISSION against the CRITERION. \
Return only JSON of the form {{\"score\": 0, \"reason\": \"...\"}} \
where score is an integer that follows the criterion's scale.\n\n\
CRITERION: {}\n\nQUESTION: {}\nSUBMISSION: {}{}",
            self.definition,
            sample.user_input,
            sample.response,
            optional_fields_block(sample),
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: SimpleCriteriaOutput = parse_json(&response.content, "simple criteria score")?;
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(parsed.score))
                .with_reason(format!("criterion '{}' scored {}", self.name, parsed.score)),
        )
    }
}

/// SQL semantic equivalence (LLM judge). The RESPONSE is the produced SQL and the
/// REFERENCE the expected SQL; any retrieved contexts are treated as the database schema.
/// Normalized exact-match short-circuits to 1.0 without an LLM call; otherwise the LLM
/// judges whether the two queries are semantically equivalent (verdict 1 / 0).
pub struct SqlSemanticEquivalenceMetric {
    llm: Arc<dyn LlmProvider>,
}

impl SqlSemanticEquivalenceMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

/// Cheap normalization for the exact-match pre-check: lowercase, collapse whitespace,
/// and drop a trailing semicolon. Not a full SQL parser — just a fast equivalence gate.
fn normalize_sql(sql: &str) -> String {
    let lowered = sql.to_lowercase();
    let collapsed = lowered.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.trim().trim_end_matches(';').trim().to_string()
}

#[async_trait]
impl Metric for SqlSemanticEquivalenceMetric {
    fn name(&self) -> &str {
        "sql_semantic_equivalence"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Response, SampleField::Reference],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = match sample.reference.as_deref() {
            Some(reference) if !reference.trim().is_empty() => reference,
            _ => {
                return Err(RagasError::Parse {
                    message: "sql semantic equivalence requires a non-empty reference".to_string(),
                });
            }
        };

        // Exact match on normalized SQL: equivalent without spending an LLM call.
        if normalize_sql(&sample.response) == normalize_sql(reference) {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(1.0))
                    .with_reason("normalized SQL exact match"),
            );
        }

        let schema = if sample.retrieved_contexts.is_empty() {
            "(no schema provided)".to_string()
        } else {
            sample.retrieved_contexts.join("\n")
        };
        let prompt = format!(
            "Decide whether the ACTUAL SQL query is semantically equivalent to the EXPECTED SQL \
query for the given DATABASE SCHEMA. They may differ in formatting, aliases, or column order \
but must return the same result set. Use verdict 1 when equivalent, 0 otherwise. \
Return only JSON of the form {{\"verdict\": 0, \"reason\": \"...\"}}.\n\n\
DATABASE SCHEMA:\n{schema}\n\nEXPECTED SQL: {reference}\nACTUAL SQL: {}",
            sample.response,
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: BinaryVerdict = parse_json(&response.content, "sql equivalence verdict")?;
        let score = if parsed.verdict == 1 { 1.0 } else { 0.0 };
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score))
                .with_reason("LLM judged SQL semantic equivalence"),
        )
    }
}

#[derive(Debug, Deserialize)]
struct FactualClassification {
    #[serde(rename = "TP", default)]
    true_positive: Vec<String>,
    #[serde(rename = "FP", default)]
    false_positive: Vec<String>,
    #[serde(rename = "FN", default)]
    false_negative: Vec<String>,
}

/// AnswerCorrectness — a weighted blend of factual F1 and semantic similarity against a
/// reference answer. Step 1: the LLM classifies statements into TP/FP/FN (claims shared,
/// claims only in the answer, claims only in the reference). Step 2: embeddings give the
/// cosine similarity between response and reference. The two are combined by the tested
/// [`answer_correctness`] formula (default weights: factual 0.75, semantic 0.25).
pub struct AnswerCorrectnessMetric {
    llm: Arc<dyn LlmProvider>,
    embedding: Arc<dyn EmbeddingProvider>,
}

impl AnswerCorrectnessMetric {
    pub fn new(llm: Arc<dyn LlmProvider>, embedding: Arc<dyn EmbeddingProvider>) -> Self {
        Self { llm, embedding }
    }

    async fn classify(
        &self,
        sample: &SingleTurnSample,
        reference: &str,
    ) -> Result<FactualClassification, RagasError> {
        let prompt = format!(
            "Compare the ANSWER to the GROUND TRUTH for the QUESTION. Decompose both into atomic \
statements and classify each statement into one list: \"TP\" (present in both the answer and the \
ground truth), \"FP\" (present in the answer but not supported by the ground truth), or \"FN\" \
(present in the ground truth but missing from the answer). Return only JSON of the form \
{{\"TP\": [\"...\"], \"FP\": [\"...\"], \"FN\": [\"...\"]}}.\n\n\
QUESTION: {}\nANSWER: {}\nGROUND TRUTH: {reference}",
            sample.user_input, sample.response,
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        parse_json(&response.content, "answer correctness classification")
    }
}

#[async_trait]
impl Metric for AnswerCorrectnessMetric {
    fn name(&self) -> &str {
        "answer_correctness"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![
                SampleField::UserInput,
                SampleField::Response,
                SampleField::Reference,
            ],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = match sample.reference.as_deref() {
            Some(reference) if !reference.trim().is_empty() => reference,
            _ => {
                return Err(RagasError::Parse {
                    message: "answer correctness requires a non-empty reference".to_string(),
                });
            }
        };

        // Step 1 — claim-level TP/FP/FN classification.
        let classification = self.classify(sample, reference).await?;
        let counts = FactualCorrectnessCounts::new(
            classification.true_positive.len(),
            classification.false_positive.len(),
            classification.false_negative.len(),
        );

        // Step 2 — semantic similarity between response and reference.
        let embedded = self
            .embedding
            .embed(EmbeddingRequest {
                input: vec![sample.response.clone(), reference.to_string()],
            })
            .await?;
        if embedded.embeddings.len() != 2 {
            return Err(RagasError::Parse {
                message: "answer correctness embedding count mismatch".to_string(),
            });
        }
        let semantic =
            cosine_similarity(&embedded.embeddings[0], &embedded.embeddings[1]).clamp(0.0, 1.0);

        // Combine via the tested weighting formula (factual 0.75 / semantic 0.25).
        let score = answer_correctness(
            semantic,
            counts.clone(),
            AnswerCorrectnessWeights::new(0.25, 0.75),
        )
        .score
        .unwrap_or(0.0);

        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "TP={} FP={} FN={}, semantic={semantic:.3}",
                counts.true_positive, counts.false_positive, counts.false_negative
            )),
        )
    }
}

#[derive(Debug, Deserialize)]
struct AnswerAccuracyRating {
    #[serde(default)]
    rating: i64,
}

/// AnswerAccuracy (Nvidia `nv_accuracy`) — two LLM rating passes with the answer and
/// reference swapped between them (to reduce position bias). Each pass rates 0/2/4, is
/// normalized to [0, 1] (rating / 4), and the two are averaged. Embedding-free.
pub struct AnswerAccuracyMetric {
    llm: Arc<dyn LlmProvider>,
}

impl AnswerAccuracyMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }

    /// One rating pass: how well does `answer` match `reference` for `question`?
    async fn rate(&self, question: &str, answer: &str, reference: &str) -> Result<f64, RagasError> {
        let prompt = format!(
            "Rate how well the ANSWER matches the REFERENCE for the QUESTION. \
Use 4 = fully correct/equivalent, 2 = partially correct, 0 = incorrect. \
Return only JSON of the form {{\"rating\": 0, \"reason\": \"...\"}}.\n\n\
QUESTION: {question}\nREFERENCE: {reference}\nANSWER: {answer}",
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: AnswerAccuracyRating = parse_json(&response.content, "answer accuracy rating")?;
        Ok(parsed.rating.clamp(0, 4) as f64 / 4.0)
    }
}

#[async_trait]
impl Metric for AnswerAccuracyMetric {
    fn name(&self) -> &str {
        "answer_accuracy"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![
                SampleField::UserInput,
                SampleField::Response,
                SampleField::Reference,
            ],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = match sample.reference.as_deref() {
            Some(reference) if !reference.trim().is_empty() => reference,
            _ => {
                return Err(RagasError::Parse {
                    message: "answer accuracy requires a non-empty reference".to_string(),
                });
            }
        };

        // Two passes with answer/reference swapped, then averaged.
        let forward = self
            .rate(&sample.user_input, &sample.response, reference)
            .await?;
        let swapped = self
            .rate(&sample.user_input, reference, &sample.response)
            .await?;
        let score = (forward + swapped) / 2.0;

        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "mean of two rating passes (forward={forward:.3}, swapped={swapped:.3})"
            )),
        )
    }
}

// ============================================================================
// Phase 3 metrics (docs/parity-roadmap.md) — LLM metrics replacing/extending the
// deterministic placeholders. Each is a real `impl Metric`.
// ============================================================================

/// One-call TP/FP/FN classification of response statements vs the reference, shared by the
/// factual-correctness style metrics.
async fn classify_tp_fp_fn(
    llm: &Arc<dyn LlmProvider>,
    user_input: &str,
    response: &str,
    reference: &str,
) -> Result<FactualClassification, RagasError> {
    let prompt = format!(
        "Compare the ANSWER to the GROUND TRUTH for the QUESTION. Decompose both into atomic \
statements and classify each into one list: \"TP\" (present in both), \"FP\" (in the answer but \
not supported by the ground truth), or \"FN\" (in the ground truth but missing from the answer). \
Return only JSON of the form {{\"TP\": [\"...\"], \"FP\": [\"...\"], \"FN\": [\"...\"]}}.\n\n\
QUESTION: {user_input}\nANSWER: {response}\nGROUND TRUTH: {reference}",
    );
    let response_msg = llm
        .generate(LlmRequest {
            messages: vec![ChatMessage::user(prompt)],
            temperature: Some(0.0),
        })
        .await?;
    parse_json(&response_msg.content, "factual correctness classification")
}

/// FactualCorrectness — claim-level F1 of the response against the reference. One LLM call
/// classifies statements into TP/FP/FN; score = 2·TP / (2·TP + FP + FN) via the tested
/// [`crate::factual_correctness`] helper.
pub struct FactualCorrectnessMetric {
    llm: Arc<dyn LlmProvider>,
}

impl FactualCorrectnessMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Metric for FactualCorrectnessMetric {
    fn name(&self) -> &str {
        "factual_correctness"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![
                SampleField::UserInput,
                SampleField::Response,
                SampleField::Reference,
            ],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = require_reference(sample, self.name())?;
        let classification =
            classify_tp_fp_fn(&self.llm, &sample.user_input, &sample.response, reference).await?;
        let counts = FactualCorrectnessCounts::new(
            classification.true_positive.len(),
            classification.false_positive.len(),
            classification.false_negative.len(),
        );
        let score = factual_correctness(counts.clone()).score.unwrap_or(0.0);
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "factual F1 from TP={} FP={} FN={}",
                counts.true_positive, counts.false_positive, counts.false_negative
            )),
        )
    }
}

#[derive(Debug, Deserialize)]
struct EntityExtractionOutput {
    #[serde(default)]
    entities: Vec<String>,
}

/// ContextEntityRecall — fraction of the reference's named entities that also appear in the
/// retrieved contexts. Two LLM extraction calls (reference, contexts); score = |ref ∩ ctx| /
/// |ref| over the lowercased entity sets. An empty reference-entity set yields NaN.
pub struct ContextEntityRecallMetric {
    llm: Arc<dyn LlmProvider>,
}

impl ContextEntityRecallMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }

    async fn extract_entities(
        &self,
        text: &str,
        context_label: &str,
    ) -> Result<BTreeSet<String>, RagasError> {
        let prompt = format!(
            "Extract the named entities (people, places, organizations, dates, products, \
numbers) mentioned in the TEXT. Return only JSON of the form {{\"entities\": [\"...\"]}}.\n\n\
TEXT: {text}",
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: EntityExtractionOutput = parse_json(&response.content, context_label)?;
        Ok(parsed
            .entities
            .into_iter()
            .map(|entity| entity.trim().to_lowercase())
            .filter(|entity| !entity.is_empty())
            .collect())
    }
}

#[async_trait]
impl Metric for ContextEntityRecallMetric {
    fn name(&self) -> &str {
        "context_entity_recall"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Reference, SampleField::RetrievedContexts],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = require_reference(sample, self.name())?;
        let reference_entities = self
            .extract_entities(reference, "context entity recall (reference)")
            .await?;
        if reference_entities.is_empty() {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(f64::NAN))
                    .with_reason("no entities found in the reference"),
            );
        }
        let context_entities = self
            .extract_entities(
                &sample.retrieved_contexts.join("\n"),
                "context entity recall (contexts)",
            )
            .await?;
        let recalled = reference_entities
            .iter()
            .filter(|entity| context_entities.contains(*entity))
            .count();
        let score = recalled as f64 / reference_entities.len() as f64;
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "{recalled}/{} reference entities present in contexts",
                reference_entities.len()
            )),
        )
    }
}

/// Average of two LLM rating passes (0/2/4 -> [0, 1]) for the Nvidia-style dual-judge metrics.
async fn dual_rating(
    llm: &Arc<dyn LlmProvider>,
    prompt: &str,
    label: &str,
) -> Result<f64, RagasError> {
    let mut total = 0.0f64;
    for _ in 0..2 {
        let response = llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt.to_string())],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: AnswerAccuracyRating = parse_json(&response.content, label)?;
        total += parsed.rating.clamp(0, 4) as f64 / 4.0;
    }
    Ok(total / 2.0)
}

/// ContextRelevance (Nvidia `nv_context_relevance`) — dual LLM rating (0/2/4 -> [0, 1],
/// averaged) of how relevant the retrieved contexts are to the question.
pub struct ContextRelevanceMetric {
    llm: Arc<dyn LlmProvider>,
}

impl ContextRelevanceMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Metric for ContextRelevanceMetric {
    fn name(&self) -> &str {
        "context_relevance"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::UserInput, SampleField::RetrievedContexts],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        if sample.retrieved_contexts.is_empty() {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(0.0))
                    .with_reason("sample has no retrieved contexts"),
            );
        }
        let prompt = format!(
            "Rate how relevant the CONTEXT is for answering the QUESTION. \
Use 4 = fully relevant, 2 = partially relevant, 0 = irrelevant. \
Return only JSON of the form {{\"rating\": 0, \"reason\": \"...\"}}.\n\n\
QUESTION: {}\nCONTEXT:\n{}",
            sample.user_input,
            sample.retrieved_contexts.join("\n"),
        );
        let score = dual_rating(&self.llm, &prompt, "context relevance rating").await?;
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score))
                .with_reason("mean of two context-relevance rating passes"),
        )
    }
}

/// ResponseGroundedness (Nvidia `nv_response_groundedness`) — dual LLM rating (0/2/4 ->
/// [0, 1], averaged) of how well the response is supported by the retrieved contexts.
pub struct ResponseGroundednessMetric {
    llm: Arc<dyn LlmProvider>,
}

impl ResponseGroundednessMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Metric for ResponseGroundednessMetric {
    fn name(&self) -> &str {
        "response_groundedness"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Response, SampleField::RetrievedContexts],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        if sample.retrieved_contexts.is_empty() {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(0.0))
                    .with_reason("sample has no retrieved contexts"),
            );
        }
        let prompt = format!(
            "Rate how well the RESPONSE is grounded in (directly supported by) the CONTEXT. \
Use 4 = fully grounded, 2 = partially grounded, 0 = not grounded. \
Return only JSON of the form {{\"rating\": 0, \"reason\": \"...\"}}.\n\n\
CONTEXT:\n{}\nRESPONSE: {}",
            sample.retrieved_contexts.join("\n"),
            sample.response,
        );
        let score = dual_rating(&self.llm, &prompt, "response groundedness rating").await?;
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score))
                .with_reason("mean of two groundedness rating passes"),
        )
    }
}

/// RubricsScore (DomainSpecificRubrics) — a single LLM call scores the submission against an
/// ordered rubric (score -> description). Returns the integer rubric score as-is.
pub struct RubricsScoreMetric {
    llm: Arc<dyn LlmProvider>,
    name: String,
    rubric: Vec<(i64, String)>,
}

impl RubricsScoreMetric {
    pub fn new(
        name: impl Into<String>,
        rubric: Vec<(i64, String)>,
        llm: Arc<dyn LlmProvider>,
    ) -> Self {
        Self {
            llm,
            name: name.into(),
            rubric,
        }
    }
}

#[async_trait]
impl Metric for RubricsScoreMetric {
    fn name(&self) -> &str {
        &self.name
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::UserInput, SampleField::Response],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let rubric_text = self
            .rubric
            .iter()
            .map(|(score, description)| format!("score {score}: {description}"))
            .collect::<Vec<_>>()
            .join("\n");
        let prompt = format!(
            "Score the SUBMISSION using the RUBRIC, returning the single best-matching integer \
score. Return only JSON of the form {{\"score\": 0, \"feedback\": \"...\"}}.\n\n\
RUBRIC:\n{rubric_text}\n\nQUESTION: {}\nSUBMISSION: {}{}",
            sample.user_input,
            sample.response,
            optional_fields_block(sample),
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: SimpleCriteriaOutput = parse_json(&response.content, "rubrics score")?;
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(parsed.score))
                .with_reason(format!("rubric '{}' scored {}", self.name, parsed.score)),
        )
    }
}

#[derive(Debug, Deserialize)]
struct InstanceRubricOutput {
    #[serde(default)]
    score: f64,
    #[serde(default)]
    feedback: String,
}

/// InstanceSpecificRubrics — like [`RubricsScoreMetric`] (DomainSpecificRubrics) but the rubric is
/// carried per-sample (`SingleTurnSample.rubrics`) instead of fixed on the metric, so every
/// instance can be judged against its own criteria. One LLM call returns `{score, feedback}`; the
/// raw integer score is returned as-is (no normalization/clamping) and the feedback becomes the
/// reason.
pub struct InstanceSpecificRubricsMetric {
    llm: Arc<dyn LlmProvider>,
}

impl InstanceSpecificRubricsMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl Metric for InstanceSpecificRubricsMetric {
    fn name(&self) -> &str {
        "instance_specific_rubrics"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::UserInput, SampleField::Response],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        if sample.rubrics.is_empty() {
            return Err(RagasError::Parse {
                message: "instance_specific_rubrics requires a per-sample rubric (sample.rubrics)"
                    .to_string(),
            });
        }
        let rubric_text = sample
            .rubrics
            .iter()
            .map(|(score, description)| format!("score {score}: {description}"))
            .collect::<Vec<_>>()
            .join("\n");
        let prompt = format!(
            "Assign an appropriate score and provide feedback to the inputs based solely on the \
scoring criteria (RUBRIC). Return only JSON of the form {{\"score\": 0, \"feedback\": \"...\"}}.\n\n\
RUBRIC:\n{rubric_text}\n\nQUESTION: {}\nSUBMISSION: {}{}",
            sample.user_input,
            sample.response,
            optional_fields_block(sample),
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: InstanceRubricOutput =
            parse_json(&response.content, "instance specific rubrics")?;
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(parsed.score))
                .with_reason(parsed.feedback),
        )
    }
}

#[derive(Debug, Deserialize)]
struct SummarizationQuestions {
    #[serde(default)]
    questions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SummarizationAnswers {
    #[serde(default)]
    answers: Vec<BinaryVerdict>,
}

/// SummarizationScore — coverage of the source's key facts by the summary. Two LLM calls:
/// (1) generate yes/no questions probing the SOURCE (retrieved contexts); (2) answer each using
/// ONLY the SUMMARY (response). score = fraction answered yes. An empty question set yields NaN.
/// (Functional port of ragas' keyphrase→QA→answer chain, collapsed to question→answer.)
pub struct SummarizationScoreMetric {
    llm: Arc<dyn LlmProvider>,
    question_count: usize,
}

impl SummarizationScoreMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            llm,
            question_count: 5,
        }
    }

    pub fn with_question_count(mut self, question_count: usize) -> Self {
        self.question_count = question_count.max(1);
        self
    }
}

#[async_trait]
impl Metric for SummarizationScoreMetric {
    fn name(&self) -> &str {
        "summarization_score"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Response, SampleField::RetrievedContexts],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        if sample.retrieved_contexts.is_empty() {
            return Err(RagasError::Parse {
                message: "summarization score requires retrieved_contexts as the source text"
                    .to_string(),
            });
        }
        let source = sample.retrieved_contexts.join("\n");

        // Step 1 — generate yes/no questions probing the source's key facts.
        let question_prompt = format!(
            "Generate {n} yes/no questions that test whether a summary captures the key facts of \
the SOURCE. Return only JSON of the form {{\"questions\": [\"...\"]}}.\n\nSOURCE:\n{source}",
            n = self.question_count,
        );
        let question_response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(question_prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let questions: SummarizationQuestions = parse_json(
            &question_response.content,
            "summarization question generation",
        )?;
        let questions: Vec<String> = questions
            .questions
            .into_iter()
            .filter(|question| !question.trim().is_empty())
            .collect();
        if questions.is_empty() {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(f64::NAN))
                    .with_reason("no questions could be generated from the source"),
            );
        }

        // Step 2 — answer each question using ONLY the summary (the response).
        let numbered = questions
            .iter()
            .enumerate()
            .map(|(index, question)| format!("{}. {question}", index + 1))
            .collect::<Vec<_>>()
            .join("\n");
        let answer_prompt = format!(
            "Using ONLY the SUMMARY, answer each QUESTION with verdict 1 when the summary \
supports a 'yes' and 0 otherwise. Return only JSON of the form \
{{\"answers\": [{{\"verdict\": 0}}]}} with exactly one entry per question, in order.\n\n\
SUMMARY: {}\n\nQUESTIONS:\n{numbered}",
            sample.response,
        );
        let answer_response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(answer_prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let answers: SummarizationAnswers =
            parse_json(&answer_response.content, "summarization answering")?;
        let covered = answers
            .answers
            .iter()
            .filter(|answer| answer.verdict == 1)
            .count();
        let score = covered as f64 / questions.len() as f64;

        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "{covered}/{} source questions answerable from the summary",
                questions.len()
            )),
        )
    }
}

/// Decompose a text into atomic statements (shared Faithfulness-style step).
async fn decompose_statements(
    llm: &Arc<dyn LlmProvider>,
    question: &str,
    answer: &str,
    label: &str,
) -> Result<Vec<String>, RagasError> {
    let prompt = format!(
        "Break the ANSWER into a list of standalone, atomic factual statements. \
Each statement must be fully self-contained, resolving any pronouns using the QUESTION. \
Return only JSON of the form {{\"statements\": [\"...\"]}}.\n\n\
QUESTION: {question}\nANSWER: {answer}",
    );
    let response = llm
        .generate(LlmRequest {
            messages: vec![ChatMessage::user(prompt)],
            temperature: Some(0.0),
        })
        .await?;
    let parsed: StatementGenerationOutput = parse_json(&response.content, label)?;
    Ok(parsed
        .statements
        .into_iter()
        .map(|statement| statement.trim().to_string())
        .filter(|statement| !statement.is_empty())
        .collect())
}

/// Verify each statement against a context block, returning a 1/0 verdict per statement, in order.
async fn verify_statements_against(
    llm: &Arc<dyn LlmProvider>,
    context: &str,
    statements: &[String],
    label: &str,
) -> Result<Vec<i64>, RagasError> {
    let numbered = statements
        .iter()
        .enumerate()
        .map(|(index, statement)| format!("{}. {statement}", index + 1))
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = format!(
        "For each STATEMENT decide whether it can be directly inferred from the CONTEXT. \
Use verdict 1 when the statement is supported by the context, 0 otherwise. \
Return only JSON of the form {{\"verdicts\": [{{\"verdict\": 0}}]}} with exactly one entry per \
statement, in order.\n\nCONTEXT:\n{context}\n\nSTATEMENTS:\n{numbered}",
    );
    let response = llm
        .generate(LlmRequest {
            messages: vec![ChatMessage::user(prompt)],
            temperature: Some(0.0),
        })
        .await?;
    let parsed: NliOutput = parse_json(&response.content, label)?;
    Ok(parsed
        .verdicts
        .iter()
        .map(|verdict| verdict.verdict)
        .collect())
}

/// Which retrieved contexts a [`NoiseSensitivityMetric`] attributes incorrect claims to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseSensitivityMode {
    /// Incorrect claims grounded in contexts that DO support the reference (relevant noise).
    Relevant,
    /// Incorrect claims grounded in contexts that do NOT support the reference (irrelevant noise).
    Irrelevant,
}

/// NoiseSensitivity — the fraction of response claims that are (a) incorrect (not attributable to
/// the reference) AND (b) grounded in the target subset of retrieved contexts. In `Relevant` mode
/// the target is the contexts that support the reference; in `Irrelevant` mode the others. Lower
/// is better. Faithful port of ragas' claim × context attribution pipeline.
pub struct NoiseSensitivityMetric {
    llm: Arc<dyn LlmProvider>,
    mode: NoiseSensitivityMode,
}

impl NoiseSensitivityMetric {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            llm,
            mode: NoiseSensitivityMode::Relevant,
        }
    }

    pub fn with_mode(mut self, mode: NoiseSensitivityMode) -> Self {
        self.mode = mode;
        self
    }

    /// Per-context verdict: is the reference (ground truth) attributable to that context?
    async fn classify_context_relevance(
        &self,
        reference: &str,
        contexts: &[String],
    ) -> Result<Vec<i64>, RagasError> {
        let numbered = contexts
            .iter()
            .enumerate()
            .map(|(index, context)| format!("{}. {context}", index + 1))
            .collect::<Vec<_>>()
            .join("\n");
        let prompt = format!(
            "For each CONTEXT decide whether the GROUND TRUTH answer can be attributed to \
(inferred from) it. Use verdict 1 when the ground truth is supported by that context, 0 \
otherwise. Return only JSON of the form {{\"verdicts\": [{{\"verdict\": 0}}]}} with exactly one \
entry per context, in order.\n\nGROUND TRUTH: {reference}\n\nCONTEXTS:\n{numbered}",
        );
        let response = self
            .llm
            .generate(LlmRequest {
                messages: vec![ChatMessage::user(prompt)],
                temperature: Some(0.0),
            })
            .await?;
        let parsed: NliOutput =
            parse_json(&response.content, "noise sensitivity context relevance")?;
        Ok(parsed
            .verdicts
            .iter()
            .map(|verdict| verdict.verdict)
            .collect())
    }
}

#[async_trait]
impl Metric for NoiseSensitivityMetric {
    fn name(&self) -> &str {
        match self.mode {
            NoiseSensitivityMode::Relevant => "noise_sensitivity_relevant",
            NoiseSensitivityMode::Irrelevant => "noise_sensitivity_irrelevant",
        }
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![
                SampleField::UserInput,
                SampleField::Response,
                SampleField::Reference,
                SampleField::RetrievedContexts,
            ],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = require_reference(sample, self.name())?;
        if sample.retrieved_contexts.is_empty() {
            // No retrieved contexts -> no retrieval noise.
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(0.0))
                    .with_reason("sample has no retrieved contexts"),
            );
        }

        // 1. Decompose the response into atomic claims.
        let statements = decompose_statements(
            &self.llm,
            &sample.user_input,
            &sample.response,
            "noise sensitivity statement generation",
        )
        .await?;
        if statements.is_empty() {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(f64::NAN))
                    .with_reason("no statements could be extracted from the response"),
            );
        }

        // 2. Correctness: is each response claim attributable to the reference?
        let correctness = verify_statements_against(
            &self.llm,
            reference,
            &statements,
            "noise sensitivity correctness",
        )
        .await?;

        // 3. Per-context relevance, then the target subset for this mode.
        let relevance = self
            .classify_context_relevance(reference, &sample.retrieved_contexts)
            .await?;
        let target_contexts: Vec<String> = sample
            .retrieved_contexts
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                let is_relevant = relevance.get(*index).copied().unwrap_or(0) == 1;
                match self.mode {
                    NoiseSensitivityMode::Relevant => is_relevant,
                    NoiseSensitivityMode::Irrelevant => !is_relevant,
                }
            })
            .map(|(_, context)| context.clone())
            .collect();

        if target_contexts.is_empty() {
            // No contexts in the target subset -> no claim can be grounded there -> 0 noise.
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(0.0)).with_reason(format!(
                    "no {} contexts to attribute incorrect claims to",
                    match self.mode {
                        NoiseSensitivityMode::Relevant => "relevant",
                        NoiseSensitivityMode::Irrelevant => "irrelevant",
                    }
                )),
            );
        }

        // 4. Is each claim grounded in the target context subset?
        let grounded = verify_statements_against(
            &self.llm,
            &target_contexts.join("\n"),
            &statements,
            "noise sensitivity grounding",
        )
        .await?;

        // 5. Noise = fraction of claims that are incorrect AND grounded in the target subset.
        let total = statements.len();
        let mut noisy = 0usize;
        for index in 0..total {
            let incorrect = correctness.get(index).copied().unwrap_or(0) == 0;
            let is_grounded = grounded.get(index).copied().unwrap_or(0) == 1;
            if incorrect && is_grounded {
                noisy += 1;
            }
        }
        let score = noisy as f64 / total as f64;

        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "{noisy}/{total} response claims are incorrect yet grounded in the target contexts"
            )),
        )
    }
}

// ============================================================================
// Phase 2 metrics (docs/parity-roadmap.md) — deterministic, no provider needed.
// Each is a real `impl Metric` wrapping a tested helper from `src/metrics`.
// ============================================================================

/// Extract a non-empty trimmed reference or return a clear error. Shared by the
/// deterministic reference-based metrics below.
fn require_reference<'a>(
    sample: &'a SingleTurnSample,
    metric: &str,
) -> Result<&'a str, RagasError> {
    match sample.reference.as_deref() {
        Some(reference) if !reference.trim().is_empty() => Ok(reference),
        _ => Err(RagasError::Parse {
            message: format!("{metric} requires a non-empty reference"),
        }),
    }
}

/// ExactMatch — 1.0 when the response equals the reference (after trimming), else 0.0.
/// Deterministic; reuses the tested [`crate::exact_match`] helper.
pub struct ExactMatchMetric;

impl Default for ExactMatchMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl ExactMatchMetric {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Metric for ExactMatchMetric {
    fn name(&self) -> &str {
        "exact_match"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Response, SampleField::Reference],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = require_reference(sample, self.name())?;
        let score = crate::exact_match(&sample.response, reference)
            .score
            .unwrap_or(0.0);
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score))
                .with_reason("deterministic exact match (trimmed)"),
        )
    }
}

/// StringPresence — 1.0 when the reference appears as a substring of the response, else 0.0.
/// Deterministic; no provider needed.
pub struct StringPresenceMetric;

impl Default for StringPresenceMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl StringPresenceMetric {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Metric for StringPresenceMetric {
    fn name(&self) -> &str {
        "string_presence"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Response, SampleField::Reference],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = require_reference(sample, self.name())?;
        let present = sample.response.contains(reference.trim());
        Ok(MetricResult::success(
            self.name(),
            MetricValue::numeric(if present { 1.0 } else { 0.0 }),
        )
        .with_reason("reference substring presence in response"))
    }
}

/// StringSimilarity — normalized string-distance similarity in [0, 1] between the response and
/// reference under a selectable [`crate::DistanceMeasure`] (Levenshtein by default). Deterministic;
/// mirrors Python ragas's `NonLLMStringSimilarity`. Reuses [`crate::string_distance_similarity_with`].
pub struct StringSimilarityMetric {
    measure: crate::DistanceMeasure,
}

impl Default for StringSimilarityMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl StringSimilarityMetric {
    pub fn new() -> Self {
        Self {
            measure: crate::DistanceMeasure::Levenshtein,
        }
    }

    /// Select the character-distance backend (Levenshtein / Hamming / Jaro / Jaro-Winkler).
    pub fn with_distance_measure(mut self, measure: crate::DistanceMeasure) -> Self {
        self.measure = measure;
        self
    }
}

#[async_trait]
impl Metric for StringSimilarityMetric {
    fn name(&self) -> &str {
        "string_similarity"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Response, SampleField::Reference],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = require_reference(sample, self.name())?;
        let score =
            crate::string_distance_similarity_with(&sample.response, reference, self.measure)
                .score
                .unwrap_or(0.0);
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score))
                .with_reason("normalized string-distance similarity"),
        )
    }
}

/// BleuScore — deterministic BLEU-4 between response and reference. Reuses [`crate::bleu_score`].
pub struct BleuScoreMetric;

impl Default for BleuScoreMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl BleuScoreMetric {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Metric for BleuScoreMetric {
    fn name(&self) -> &str {
        "bleu_score"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Response, SampleField::Reference],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = require_reference(sample, self.name())?;
        let score = crate::bleu_score(&sample.response, reference)
            .score
            .unwrap_or(0.0);
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score))
                .with_reason("BLEU-4 (deterministic, functional)"),
        )
    }
}

/// ChrfScore — deterministic chrF (character n-gram F-beta, beta=2) between response and
/// reference. Reuses [`crate::chrf`].
pub struct ChrfScoreMetric;

impl Default for ChrfScoreMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl ChrfScoreMetric {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Metric for ChrfScoreMetric {
    fn name(&self) -> &str {
        "chrf"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Response, SampleField::Reference],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = require_reference(sample, self.name())?;
        let score = crate::chrf(&sample.response, reference)
            .score
            .unwrap_or(0.0);
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score))
                .with_reason("chrF (deterministic, functional)"),
        )
    }
}

/// Maximum normalized string-distance similarity of `text` against any of `candidates`.
fn max_string_similarity(text: &str, candidates: &[String]) -> f64 {
    candidates
        .iter()
        .map(|candidate| {
            crate::string_distance_similarity(text, candidate)
                .score
                .unwrap_or(0.0)
        })
        .fold(0.0f64, f64::max)
}

const NON_LLM_CONTEXT_THRESHOLD: f64 = 0.5;

/// NonLLMContextPrecisionWithReference — deterministic Average Precision@k. A retrieved context
/// is "relevant" when its normalized string similarity to ANY `reference_contexts` entry is at
/// least `threshold` (default 0.5). No provider needed.
pub struct NonLlmContextPrecisionMetric {
    threshold: f64,
}

impl Default for NonLlmContextPrecisionMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl NonLlmContextPrecisionMetric {
    pub fn new() -> Self {
        Self {
            threshold: NON_LLM_CONTEXT_THRESHOLD,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }
}

#[async_trait]
impl Metric for NonLlmContextPrecisionMetric {
    fn name(&self) -> &str {
        "non_llm_context_precision"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(self.name(), vec![SampleField::RetrievedContexts])
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        if sample.reference_contexts.is_empty() {
            return Err(RagasError::Parse {
                message: "non_llm_context_precision requires reference_contexts".to_string(),
            });
        }
        if sample.retrieved_contexts.is_empty() {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(0.0))
                    .with_reason("sample has no retrieved contexts"),
            );
        }
        let verdicts: Vec<i64> = sample
            .retrieved_contexts
            .iter()
            .map(|retrieved| {
                let similar =
                    max_string_similarity(retrieved, &sample.reference_contexts) >= self.threshold;
                i64::from(similar)
            })
            .collect();
        let relevant = verdicts.iter().filter(|verdict| **verdict == 1).count();
        Ok(MetricResult::success(
            self.name(),
            MetricValue::numeric(average_precision_at_k(&verdicts)),
        )
        .with_reason(format!(
            "average precision@k over {relevant}/{} relevant context(s)",
            verdicts.len()
        )))
    }
}

/// NonLLMContextRecall — fraction of `reference_contexts` that are covered by some retrieved
/// context (normalized string similarity >= `threshold`, default 0.5). Deterministic.
pub struct NonLlmContextRecallMetric {
    threshold: f64,
}

impl Default for NonLlmContextRecallMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl NonLlmContextRecallMetric {
    pub fn new() -> Self {
        Self {
            threshold: NON_LLM_CONTEXT_THRESHOLD,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }
}

#[async_trait]
impl Metric for NonLlmContextRecallMetric {
    fn name(&self) -> &str {
        "non_llm_context_recall"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(self.name(), vec![SampleField::RetrievedContexts])
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        if sample.reference_contexts.is_empty() {
            return Err(RagasError::Parse {
                message: "non_llm_context_recall requires reference_contexts".to_string(),
            });
        }
        let covered = sample
            .reference_contexts
            .iter()
            .filter(|reference| {
                max_string_similarity(reference, &sample.retrieved_contexts) >= self.threshold
            })
            .count();
        let total = sample.reference_contexts.len();
        let score = covered as f64 / total as f64;
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "{covered}/{total} reference contexts covered by the retrieved contexts"
            )),
        )
    }
}

/// IDBasedContextPrecision — deterministic: fraction of `retrieved_context_ids` that are present
/// in the `reference_context_ids` set. Reuses the tested [`crate::id_based_context_precision`].
pub struct IdBasedContextPrecisionMetric;

impl Default for IdBasedContextPrecisionMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl IdBasedContextPrecisionMetric {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Metric for IdBasedContextPrecisionMetric {
    fn name(&self) -> &str {
        "id_based_context_precision"
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        if sample.reference_context_ids.is_empty() {
            return Err(RagasError::Parse {
                message: "id_based_context_precision requires reference_context_ids".to_string(),
            });
        }
        let score = crate::id_based_context_precision(
            &sample.retrieved_context_ids,
            &sample.reference_context_ids,
        )
        .score
        .unwrap_or(0.0);
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score))
                .with_reason("retrieved context-id overlap with the reference id set / retrieved"),
        )
    }
}

/// IDBasedContextRecall — deterministic: fraction of `reference_context_ids` that appear in the
/// `retrieved_context_ids` set.
pub struct IdBasedContextRecallMetric;

impl Default for IdBasedContextRecallMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl IdBasedContextRecallMetric {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Metric for IdBasedContextRecallMetric {
    fn name(&self) -> &str {
        "id_based_context_recall"
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        if sample.reference_context_ids.is_empty() {
            return Err(RagasError::Parse {
                message: "id_based_context_recall requires reference_context_ids".to_string(),
            });
        }
        let retrieved: BTreeSet<&str> = sample
            .retrieved_context_ids
            .iter()
            .map(|id| id.trim())
            .filter(|id| !id.is_empty())
            .collect();
        let reference: BTreeSet<&str> = sample
            .reference_context_ids
            .iter()
            .map(|id| id.trim())
            .filter(|id| !id.is_empty())
            .collect();
        if reference.is_empty() {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(f64::NAN))
                    .with_reason("reference_context_ids contained no non-empty ids"),
            );
        }
        let matched = reference
            .iter()
            .filter(|id| retrieved.contains(*id))
            .count();
        let score = matched as f64 / reference.len() as f64;
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "{matched}/{} reference context ids were retrieved",
                reference.len()
            )),
        )
    }
}

/// Which component a [`DataCompyScoreMetric`] reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataCompyMode {
    Precision,
    Recall,
    F1,
}

/// Parse `text` as CSV (headerless, flexible column counts); each row becomes one canonical
/// string (cells joined by the unit separator) for multiset comparison.
fn parse_csv_rows(text: &str) -> Result<Vec<String>, RagasError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|error| RagasError::Parse {
            message: format!("datacompy CSV parse failed: {error}"),
        })?;
        rows.push(record.iter().collect::<Vec<_>>().join("\u{1f}"));
    }
    Ok(rows)
}

/// Multiset overlap (sum of min counts) between two row collections.
fn row_multiset_overlap(left: &[String], right: &[String]) -> usize {
    let mut left_counts: BTreeMap<&String, usize> = BTreeMap::new();
    for row in left {
        *left_counts.entry(row).or_insert(0) += 1;
    }
    let mut right_counts: BTreeMap<&String, usize> = BTreeMap::new();
    for row in right {
        *right_counts.entry(row).or_insert(0) += 1;
    }
    left_counts
        .iter()
        .map(|(row, count)| (*count).min(*right_counts.get(*row).unwrap_or(&0)))
        .sum()
}

/// DataCompyScore — deterministic row-level comparison of the response and reference as CSV
/// tables. Rows are compared as a multiset: precision = matched / response rows, recall =
/// matched / reference rows, F1 = harmonic mean. Reports the [`DataCompyMode`] component
/// (default F1). NaN when the reference has no rows. No provider needed.
pub struct DataCompyScoreMetric {
    mode: DataCompyMode,
}

impl Default for DataCompyScoreMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl DataCompyScoreMetric {
    pub fn new() -> Self {
        Self {
            mode: DataCompyMode::F1,
        }
    }

    pub fn with_mode(mut self, mode: DataCompyMode) -> Self {
        self.mode = mode;
        self
    }
}

#[async_trait]
impl Metric for DataCompyScoreMetric {
    fn name(&self) -> &str {
        "datacompy_score"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Response, SampleField::Reference],
        )
    }

    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = require_reference(sample, self.name())?;
        let response_rows = parse_csv_rows(&sample.response)?;
        let reference_rows = parse_csv_rows(reference)?;
        if reference_rows.is_empty() {
            return Ok(
                MetricResult::success(self.name(), MetricValue::numeric(f64::NAN))
                    .with_reason("reference CSV has no rows"),
            );
        }
        let matched = row_multiset_overlap(&response_rows, &reference_rows);
        let precision = if response_rows.is_empty() {
            0.0
        } else {
            matched as f64 / response_rows.len() as f64
        };
        let recall = matched as f64 / reference_rows.len() as f64;
        let f1 = if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        };
        let score = match self.mode {
            DataCompyMode::Precision => precision,
            DataCompyMode::Recall => recall,
            DataCompyMode::F1 => f1,
        };
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "{matched} matching rows (precision={precision:.3}, recall={recall:.3})"
            )),
        )
    }
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

    #[test]
    fn test_metric_value_non_finite_score_serializes_as_json_null_and_round_trips() {
        // Edge case: Faithfulness over an empty statement set returns 0/0 = NaN
        // (see `FaithfulnessMetric::score`). A whole `EvaluationReport` carrying
        // that NaN must (1) serialize, rendering the score as JSON `null`, and
        // (2) round-trip back to a NaN numeric.
        use crate::{EvaluationReport, SampleEvaluation};

        let report = EvaluationReport {
            metric_names: vec!["faithfulness".to_string()],
            usage: Default::default(),
            results: vec![SampleEvaluation {
                sample_index: 0,
                results: vec![
                    MetricResult::success("faithfulness", MetricValue::numeric(f64::NAN))
                        .with_reason("no statements could be extracted from the response"),
                ],
            }],
        };

        // The whole report serializes successfully and the NaN score is `null`.
        let json = serde_json::to_string(&report).expect("report with NaN score must serialize");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

        let score = &value["results"][0]["results"][0]["value"]["Numeric"];
        assert!(
            score.is_null(),
            "non-finite numeric score must render as JSON null, got: {score}"
        );
        // The textual reason and surrounding structure are untouched.
        assert_eq!(value["metric_names"][0], "faithfulness");
        assert_eq!(
            value["results"][0]["results"][0]["reason"],
            "no statements could be extracted from the response"
        );

        // The concrete gap this fix closes: with the derived `Deserialize`,
        // reading `{"Numeric": null}` back failed with "invalid type: null,
        // expected f64". It must now restore to a NaN numeric so the report
        // survives a JSON round-trip.
        let restored: EvaluationReport = serde_json::from_str(&json).expect("round-trip");
        let restored_value = restored.results[0].results[0]
            .value
            .as_ref()
            .and_then(MetricValue::as_numeric)
            .expect("numeric value present");
        assert!(restored_value.is_nan(), "null must restore to NaN");
    }

    #[test]
    fn test_metric_value_null_numeric_deserializes_to_nan() {
        // Direct guard on the deserialize gap, independent of the serializer:
        // hand-authored `{"Numeric": null}` (as produced by Python/ragas for an
        // undefined score) must parse back to NaN rather than erroring.
        let restored: MetricValue =
            serde_json::from_str("{\"Numeric\":null}").expect("null numeric must deserialize");
        assert!(
            restored.as_numeric().expect("numeric variant").is_nan(),
            "null numeric must deserialize to NaN"
        );
    }

    #[test]
    fn test_metric_value_infinite_score_serializes_as_json_null() {
        // +Inf and -Inf are also non-finite and must follow the same null path.
        for value in [f64::INFINITY, f64::NEG_INFINITY] {
            let metric = MetricValue::numeric(value);
            let json = serde_json::to_string(&metric).expect("infinite value must serialize");
            assert_eq!(json, "{\"Numeric\":null}");
        }
    }

    #[test]
    fn test_metric_value_finite_score_round_trips_unchanged() {
        // Finite numbers must serialize exactly as the previous derived impl did
        // (`{"Numeric": <number>}`) and deserialize back to the same value, so the
        // NaN fix does not change the wire format for ordinary scores.
        let cases = [
            (MetricValue::numeric(0.0), "{\"Numeric\":0.0}"),
            (MetricValue::numeric(0.82), "{\"Numeric\":0.82}"),
            (MetricValue::discrete("pass"), "{\"Discrete\":\"pass\"}"),
        ];
        for (value, expected_json) in cases {
            let json = serde_json::to_string(&value).expect("serialize");
            assert_eq!(json, expected_json);
            let restored: MetricValue = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(restored, value);
        }

        // A ranking with finite scores also round-trips byte-for-byte.
        let ranking = MetricValue::ranking(vec![
            RankingItem::new("ctx-a", 0.91),
            RankingItem::new("ctx-b", 0.33),
        ]);
        let json = serde_json::to_string(&ranking).expect("serialize ranking");
        let restored: MetricValue = serde_json::from_str(&json).expect("deserialize ranking");
        assert_eq!(restored, ranking);
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

        let result = metric
            .score(&faithfulness_sample())
            .await
            .expect("faithfulness");

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
            numeric(
                &partial
                    .score(&faithfulness_sample())
                    .await
                    .expect("partial")
            ),
            0.5
        );

        // Nothing supported -> 0.0 (an unfaithful answer must score low).
        let unsupported = FaithfulnessMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"statements":["a","b"]}"#,
            r#"{"verdicts":[{"verdict":0},{"verdict":0}]}"#,
        ])));
        assert_eq!(
            numeric(
                &unsupported
                    .score(&faithfulness_sample())
                    .await
                    .expect("unsupported")
            ),
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

        let result = metric
            .score(&faithfulness_sample())
            .await
            .expect("repaired");

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
        let metric = ResponseRelevancyMetric::new(llm.clone(), Arc::new(PanicEmbeddingProvider));

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
            Arc::new(ScriptedLlm::new(vec!["I cannot turn this into questions."])),
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
        .with_reference(
            "Ragas evaluates LLM applications and is maintained by Exploding Gradients.",
        )
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

        let error = metric.score(&sample).await.expect_err("missing reference");

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

        let result = metric
            .score(&context_recall_sample())
            .await
            .expect("recall");

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
        let sample = SingleTurnSample::new("question", "answer", vec!["ctx".to_string()])
            .with_reference("   ");

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

        let result = metric
            .score(&context_recall_sample())
            .await
            .expect("repaired");

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

    #[tokio::test]
    async fn semantic_similarity_scores_cosine_with_optional_threshold() {
        fn provider(pairs: &[(&str, Vec<f32>)]) -> Arc<MapEmbeddingProvider> {
            Arc::new(MapEmbeddingProvider {
                vectors: pairs
                    .iter()
                    .map(|(text, vector)| (text.to_string(), vector.clone()))
                    .collect(),
            })
        }
        let sample = |response: &str, reference: &str| {
            SingleTurnSample::new("q", response, vec!["c".to_string()]).with_reference(reference)
        };

        // Parallel vectors -> cosine 1.0.
        let parallel = provider(&[("p", vec![1.0, 0.0, 0.0]), ("pr", vec![2.0, 0.0, 0.0])]);
        let score = numeric(
            &SemanticSimilarityMetric::new(parallel)
                .score(&sample("p", "pr"))
                .await
                .expect("score"),
        );
        assert!(
            (score - 1.0).abs() < 1e-9,
            "parallel cosine is 1.0, got {score}"
        );

        // Orthogonal vectors -> cosine 0.0.
        let orthogonal = provider(&[("a", vec![1.0, 0.0]), ("b", vec![0.0, 1.0])]);
        let score = numeric(
            &SemanticSimilarityMetric::new(orthogonal)
                .score(&sample("a", "b"))
                .await
                .expect("score"),
        );
        assert!(score.abs() < 1e-9, "orthogonal cosine is 0.0, got {score}");

        // Partial overlap -> cosine ~0.7071; threshold gates it inclusively.
        let partial = provider(&[("e", vec![1.0, 1.0]), ("f", vec![1.0, 0.0])]);
        let raw = numeric(
            &SemanticSimilarityMetric::new(partial.clone())
                .score(&sample("e", "f"))
                .await
                .expect("raw"),
        );
        assert!(
            (raw - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-6,
            "got {raw}"
        );
        let pass = numeric(
            &SemanticSimilarityMetric::new(partial.clone())
                .with_threshold(0.7)
                .score(&sample("e", "f"))
                .await
                .expect("pass"),
        );
        assert_eq!(pass, 1.0, "0.7071 >= 0.7 inclusive -> 1.0");
        let fail = numeric(
            &SemanticSimilarityMetric::new(partial)
                .with_threshold(0.8)
                .score(&sample("e", "f"))
                .await
                .expect("fail"),
        );
        assert_eq!(fail, 0.0, "0.7071 < 0.8 -> 0.0");

        // A threshold of 0.0 is "no threshold" (ragas truthiness): the raw cosine is returned,
        // NOT binarized to 1.0 (every clamped cosine is >= 0.0).
        let zero_threshold = provider(&[("a", vec![1.0, 0.0]), ("b", vec![0.0, 1.0])]);
        let raw_zero = numeric(
            &SemanticSimilarityMetric::new(zero_threshold)
                .with_threshold(0.0)
                .score(&sample("a", "b"))
                .await
                .expect("zero threshold"),
        );
        assert_eq!(
            raw_zero, 0.0,
            "threshold 0.0 means no threshold -> raw cosine 0.0"
        );

        // An empty response is coerced to " " before embedding (ragas `answer or " "`). With " "
        // and the reference mapped to the same vector the score is 1.0 — proving "" was coerced,
        // not sent through (an unmapped "" -> empty vector -> cosine 0.0).
        let coerced = provider(&[(" ", vec![1.0, 0.0]), ("ref", vec![1.0, 0.0])]);
        let empty_response = numeric(
            &SemanticSimilarityMetric::new(coerced)
                .score(&sample("", "ref"))
                .await
                .expect("empty response coerced"),
        );
        assert!(
            (empty_response - 1.0).abs() < 1e-9,
            "empty response coerced to ' ' -> cosine 1.0, got {empty_response}"
        );

        // A missing reference is a hard error (the metric is reference-based).
        let no_reference = SingleTurnSample::new("q", "resp", vec!["c".to_string()]);
        let error = SemanticSimilarityMetric::new(provider(&[]))
            .score(&no_reference)
            .await
            .expect_err("reference is required");
        assert!(
            error.to_string().contains("requires a reference"),
            "{error}"
        );
    }

    // ---------------------------------------------------------------------
    // LIVE discrimination gate (IGNORED unless OPENAI_API_KEY is set).
    //
    // These are the project's real "done" gate for the LLM metrics: they call a
    // real provider and assert that a faithful/relevant sample scores STRICTLY
    // higher than an adversarial one. They are #[ignore]-attributed so they never
    // run in normal `cargo test`; run them with:
    //   OPENAI_API_KEY=... cargo test --lib -- --ignored
    // Live-verified 2026-06-06: all of these gates passed against a real provider
    // (DeepSeek chat + SiliconFlow embeddings), 20 passed / 0 failed. Evidence:
    // docs/live-verification/results.md. They stay #[ignore] (no key in CI); re-run
    // to re-verify on another provider/model.
    // ---------------------------------------------------------------------
    use crate::OpenAiCompatibleClient;

    /// Build the live chat client from [`crate::ProviderConfig`], or `None` to skip when no key.
    fn live_client() -> Option<Arc<OpenAiCompatibleClient>> {
        crate::ProviderConfig::from_env()
            .chat_client()
            .map(Arc::new)
    }

    /// Embeddings may live on a different provider than chat (e.g. DeepSeek chat +
    /// SiliconFlow embeddings) via `OPENAI_EMBEDDING_BASE_URL`/`OPENAI_EMBEDDING_API_KEY`;
    /// [`crate::ProviderConfig`] resolves the fallback to the chat endpoint/key.
    fn live_embedding_client() -> Option<Arc<OpenAiCompatibleClient>> {
        crate::ProviderConfig::from_env()
            .embedding_client()
            .map(Arc::new)
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
    async fn live_semantic_similarity_scores_similar_above_dissimilar() {
        let Some(embedding) = live_embedding_client() else {
            return;
        };
        let metric = SemanticSimilarityMetric::new(embedding);

        let similar = SingleTurnSample::new(
            "Where is the Eiffel Tower?",
            "The Eiffel Tower is located in Paris, France.",
            vec!["c".to_string()],
        )
        .with_reference("Paris, France is home to the Eiffel Tower.");
        let dissimilar = SingleTurnSample::new(
            "Where is the Eiffel Tower?",
            "The Eiffel Tower is located in Paris, France.",
            vec!["c".to_string()],
        )
        .with_reference("Photosynthesis converts sunlight into chemical energy in plants.");

        let high = numeric(&metric.score(&similar).await.expect("similar"));
        let low = numeric(&metric.score(&dissimilar).await.expect("dissimilar"));
        assert!(
            high > low,
            "semantically-similar reference={high} must exceed dissimilar={low}"
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
            .with_reference("Ragas evaluates LLM applications. Exploding Gradients maintains it.");
        let unsupported = SingleTurnSample::new("What is Ragas?", "answer", contexts)
            .with_reference("Mount Everest is the tallest mountain. The ocean is deep and blue.");

        let supported_score = numeric(&metric.score(&supported).await.expect("supported"));
        let unsupported_score = numeric(&metric.score(&unsupported).await.expect("unsupported"));

        assert!(
            supported_score > unsupported_score,
            "supported={supported_score} must exceed unsupported={unsupported_score}"
        );
    }

    // ---------------------------------------------------------------------
    // Phase 1 metrics — unit discrimination (ScriptedLlm, offline).
    // ---------------------------------------------------------------------

    #[tokio::test]
    async fn context_utilization_scores_average_precision_without_a_reference() {
        // Per-context verdicts [1, 0, 1] -> AP = (1/1 + 2/3) / 2 (2 relevant).
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"verdict":1}"#,
            r#"{"verdict":0}"#,
            r#"{"verdict":1}"#,
        ]));
        let metric = ContextUtilizationMetric::new(llm.clone());
        let sample = SingleTurnSample::new(
            "What is Ragas?",
            "Ragas evaluates LLM applications.",
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );

        let result = metric.score(&sample).await.expect("context utilization");

        assert_eq!(result.metric_name, "context_utilization");
        assert!((numeric(&result) - (1.0 + 2.0 / 3.0) / 2.0).abs() < 1e-9);
        // Judges each context against the RESPONSE (no reference involved).
        let prompts = llm.prompts();
        assert_eq!(prompts.len(), 3);
        assert!(prompts[0].contains("ANSWER: Ragas evaluates LLM applications."));
    }

    #[tokio::test]
    async fn context_utilization_all_useless_is_zero_and_empty_skips_llm() {
        let useless = ContextUtilizationMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"verdict":0}"#,
            r#"{"verdict":0}"#,
        ])));
        let sample = SingleTurnSample::new("q", "resp", vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            numeric(&useless.score(&sample).await.expect("useless")),
            0.0
        );

        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let empty = ContextUtilizationMetric::new(llm.clone());
        let no_ctx = SingleTurnSample::new("q", "resp", Vec::new());
        assert_eq!(numeric(&empty.score(&no_ctx).await.expect("empty")), 0.0);
        assert!(llm.prompts().is_empty());
    }

    #[tokio::test]
    async fn aspect_critic_emits_binary_verdict_under_its_configured_name() {
        let sample = SingleTurnSample::new("q", "resp", vec!["ctx".to_string()]);

        let positive = AspectCriticMetric::new(
            "on_topic",
            "Does the submission answer the question?",
            Arc::new(ScriptedLlm::new(vec![r#"{"verdict":1,"reason":"yes"}"#])),
        );
        let result = positive.score(&sample).await.expect("aspect");
        assert_eq!(result.metric_name, "on_topic");
        assert_eq!(numeric(&result), 1.0);

        let negative = AspectCriticMetric::new(
            "on_topic",
            "Does the submission answer the question?",
            Arc::new(ScriptedLlm::new(vec![r#"{"verdict":0}"#])),
        );
        assert_eq!(
            numeric(&negative.score(&sample).await.expect("aspect")),
            0.0
        );
    }

    #[tokio::test]
    async fn aspect_critic_strictness_takes_majority_vote() {
        // Three calls -> [1, 1, 0] -> majority 1.
        let metric = AspectCriticMetric::new(
            "c",
            "def",
            Arc::new(ScriptedLlm::new(vec![
                r#"{"verdict":1}"#,
                r#"{"verdict":1}"#,
                r#"{"verdict":0}"#,
            ])),
        )
        .with_strictness(3);
        let sample = SingleTurnSample::new("q", "resp", vec!["ctx".to_string()]);
        assert_eq!(numeric(&metric.score(&sample).await.expect("strict")), 1.0);
    }

    #[tokio::test]
    async fn simple_criteria_returns_the_raw_integer_score() {
        let metric = SimpleCriteriaScoreMetric::new(
            "helpfulness",
            "Score from 0 to 5 how helpful the answer is.",
            Arc::new(ScriptedLlm::new(vec![r#"{"score":4,"reason":"good"}"#])),
        );
        let sample = SingleTurnSample::new("q", "resp", vec!["ctx".to_string()]);
        let result = metric.score(&sample).await.expect("simple criteria");
        assert_eq!(result.metric_name, "helpfulness");
        assert_eq!(numeric(&result), 4.0);
    }

    #[tokio::test]
    async fn sql_equivalence_exact_match_short_circuits_without_llm() {
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let metric = SqlSemanticEquivalenceMetric::new(llm.clone());
        let sample = SingleTurnSample::new(
            "q",
            "SELECT  id, name  FROM users;",
            vec!["schema".to_string()],
        )
        .with_reference("select id, name from users");

        let result = metric.score(&sample).await.expect("sql exact");

        assert_eq!(numeric(&result), 1.0);
        assert!(
            llm.prompts().is_empty(),
            "exact match must not call the LLM"
        );
    }

    #[tokio::test]
    async fn sql_equivalence_falls_back_to_the_llm_judge() {
        let sample = SingleTurnSample::new(
            "q",
            "SELECT name, id FROM users",
            vec!["schema".to_string()],
        )
        .with_reference("SELECT id, name FROM users");

        let equivalent = SqlSemanticEquivalenceMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"verdict":1,"reason":"same result"}"#,
        ])));
        assert_eq!(
            numeric(&equivalent.score(&sample).await.expect("equiv")),
            1.0
        );

        let different = SqlSemanticEquivalenceMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"verdict":0,"reason":"different"}"#,
        ])));
        assert_eq!(numeric(&different.score(&sample).await.expect("diff")), 0.0);
    }

    #[tokio::test]
    async fn sql_equivalence_missing_reference_is_an_error() {
        let metric = SqlSemanticEquivalenceMetric::new(Arc::new(ScriptedLlm::new(vec![])));
        let sample = SingleTurnSample::new("q", "SELECT 1", vec!["s".to_string()]);
        let error = metric.score(&sample).await.expect_err("missing reference");
        assert!(error.to_string().contains("reference"));
    }

    #[tokio::test]
    async fn answer_correctness_discriminates_correct_from_incorrect() {
        let sample = SingleTurnSample::new("q", "the response", vec!["ctx".to_string()])
            .with_reference("the reference");

        // TP=2, FP=0, FN=0 -> factual 1.0; identical embeddings -> semantic 1.0 -> 1.0.
        let correct = AnswerCorrectnessMetric::new(
            Arc::new(ScriptedLlm::new(vec![
                r#"{"TP":["a","b"],"FP":[],"FN":[]}"#,
            ])),
            Arc::new(MapEmbeddingProvider {
                vectors: HashMap::from([
                    ("the response".to_string(), vec![1.0, 0.0]),
                    ("the reference".to_string(), vec![1.0, 0.0]),
                ]),
            }),
        );
        let correct_score = numeric(&correct.score(&sample).await.expect("correct"));
        assert!((correct_score - 1.0).abs() < 1e-9);

        // TP=0, FP=2, FN=2 -> factual 0.0; orthogonal embeddings -> semantic 0.0 -> 0.0.
        let incorrect = AnswerCorrectnessMetric::new(
            Arc::new(ScriptedLlm::new(vec![
                r#"{"TP":[],"FP":["x","y"],"FN":["a","b"]}"#,
            ])),
            Arc::new(MapEmbeddingProvider {
                vectors: HashMap::from([
                    ("the response".to_string(), vec![1.0, 0.0]),
                    ("the reference".to_string(), vec![0.0, 1.0]),
                ]),
            }),
        );
        let incorrect_score = numeric(&incorrect.score(&sample).await.expect("incorrect"));
        assert!(incorrect_score.abs() < 1e-9);
        assert!(incorrect_score < correct_score);
    }

    #[tokio::test]
    async fn answer_accuracy_averages_two_swapped_rating_passes() {
        let sample =
            SingleTurnSample::new("q", "resp", vec!["ctx".to_string()]).with_reference("ref");

        // Both passes rate 4 -> (1.0 + 1.0) / 2 = 1.0.
        let high = AnswerAccuracyMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"rating":4}"#,
            r#"{"rating":4}"#,
        ])));
        assert_eq!(numeric(&high.score(&sample).await.expect("high")), 1.0);

        // Both passes rate 0 -> 0.0 (discrimination: low).
        let low = AnswerAccuracyMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"rating":0}"#,
            r#"{"rating":0}"#,
        ])));
        assert_eq!(numeric(&low.score(&sample).await.expect("low")), 0.0);
    }

    // ---------------------------------------------------------------------
    // Phase 1 metrics — LIVE discrimination gates (IGNORED unless a key is set).
    //   OPENAI_API_KEY=... cargo test --lib -- --ignored
    // ---------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_context_utilization_ranks_useful_context_above_useless() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = ContextUtilizationMetric::new(client);
        let response = "Ragas evaluates LLM applications.";
        let useful_first = SingleTurnSample::new(
            "What is Ragas?",
            response,
            vec![
                "Ragas is a framework to evaluate LLM applications.".to_string(),
                "The Eiffel Tower is located in Paris, France.".to_string(),
            ],
        );
        let useful_last = SingleTurnSample::new(
            "What is Ragas?",
            response,
            vec![
                "The Eiffel Tower is located in Paris, France.".to_string(),
                "Ragas is a framework to evaluate LLM applications.".to_string(),
            ],
        );

        let first = numeric(&metric.score(&useful_first).await.expect("useful first"));
        let last = numeric(&metric.score(&useful_last).await.expect("useful last"));
        assert!(
            first > last,
            "useful-first={first} must exceed useful-last={last}"
        );
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_aspect_critic_discriminates_on_topic_answers() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = AspectCriticMetric::new(
            "on_topic",
            "Does the submission directly and correctly answer the question?",
            client,
        );
        let good = SingleTurnSample::new(
            "What is the capital of France?",
            "The capital of France is Paris.",
            vec!["n/a".to_string()],
        );
        let bad = SingleTurnSample::new(
            "What is the capital of France?",
            "I really enjoy hiking on the weekends.",
            vec!["n/a".to_string()],
        );

        let good_score = numeric(&metric.score(&good).await.expect("good"));
        let bad_score = numeric(&metric.score(&bad).await.expect("bad"));
        assert!(good_score > bad_score, "good={good_score} bad={bad_score}");
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_simple_criteria_scores_better_answer_higher() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = SimpleCriteriaScoreMetric::new(
            "correctness",
            "Score from 0 to 5 how factually correct the submission is.",
            client,
        );
        let good = SingleTurnSample::new("What is 2+2?", "2 + 2 = 4.", vec!["n/a".to_string()]);
        let bad = SingleTurnSample::new("What is 2+2?", "2 + 2 = 17.", vec!["n/a".to_string()]);

        let good_score = numeric(&metric.score(&good).await.expect("good"));
        let bad_score = numeric(&metric.score(&bad).await.expect("bad"));
        assert!(good_score > bad_score, "good={good_score} bad={bad_score}");
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_sql_equivalence_scores_equivalent_above_different() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = SqlSemanticEquivalenceMetric::new(client);
        let schema = vec!["Table users(id INT, name TEXT, age INT)".to_string()];
        let equivalent = SingleTurnSample::new(
            "Get all user names",
            "SELECT name FROM users",
            schema.clone(),
        )
        .with_reference("SELECT users.name FROM users");
        let different =
            SingleTurnSample::new("Get all user names", "SELECT age FROM users", schema)
                .with_reference("SELECT users.name FROM users");

        let eq = numeric(&metric.score(&equivalent).await.expect("equivalent"));
        let df = numeric(&metric.score(&different).await.expect("different"));
        assert!(eq > df, "equivalent={eq} different={df}");
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_answer_correctness_scores_correct_above_incorrect() {
        let (Some(llm), Some(embedding)) = (live_client(), live_embedding_client()) else {
            return;
        };
        let metric = AnswerCorrectnessMetric::new(llm, embedding);
        let correct = SingleTurnSample::new(
            "What is the capital of France?",
            "The capital of France is Paris.",
            vec!["n/a".to_string()],
        )
        .with_reference("Paris is the capital of France.");
        let incorrect = SingleTurnSample::new(
            "What is the capital of France?",
            "The capital of France is Berlin.",
            vec!["n/a".to_string()],
        )
        .with_reference("Paris is the capital of France.");

        let correct_score = numeric(&metric.score(&correct).await.expect("correct"));
        let incorrect_score = numeric(&metric.score(&incorrect).await.expect("incorrect"));
        assert!(
            correct_score > incorrect_score,
            "correct={correct_score} incorrect={incorrect_score}"
        );
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_answer_accuracy_scores_correct_above_incorrect() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = AnswerAccuracyMetric::new(client);
        let correct = SingleTurnSample::new(
            "What is the capital of France?",
            "Paris.",
            vec!["n/a".to_string()],
        )
        .with_reference("The capital of France is Paris.");
        let incorrect = SingleTurnSample::new(
            "What is the capital of France?",
            "Berlin.",
            vec!["n/a".to_string()],
        )
        .with_reference("The capital of France is Paris.");

        let correct_score = numeric(&metric.score(&correct).await.expect("correct"));
        let incorrect_score = numeric(&metric.score(&incorrect).await.expect("incorrect"));
        assert!(
            correct_score > incorrect_score,
            "correct={correct_score} incorrect={incorrect_score}"
        );
    }

    // ---------------------------------------------------------------------
    // Phase 2 deterministic metrics — fully offline (no provider, no key).
    // ---------------------------------------------------------------------

    #[tokio::test]
    async fn deterministic_string_metrics_discriminate_match_from_mismatch() {
        let matching = SingleTurnSample::new("q", "the quick brown fox", vec!["c".to_string()])
            .with_reference("the quick brown fox");
        let different = SingleTurnSample::new("q", "totally unrelated text", vec!["c".to_string()])
            .with_reference("the quick brown fox");

        // ExactMatch: identical -> 1.0, different -> 0.0.
        let em = ExactMatchMetric::new();
        assert_eq!(em.name(), "exact_match");
        assert_eq!(numeric(&em.score(&matching).await.expect("em match")), 1.0);
        assert_eq!(numeric(&em.score(&different).await.expect("em diff")), 0.0);

        // StringPresence: reference substring present -> 1.0; absent -> 0.0.
        let sp = StringPresenceMetric::new();
        let contains = SingleTurnSample::new(
            "q",
            "well, the quick brown fox jumped",
            vec!["c".to_string()],
        )
        .with_reference("quick brown fox");
        assert_eq!(numeric(&sp.score(&contains).await.expect("sp yes")), 1.0);
        assert_eq!(numeric(&sp.score(&different).await.expect("sp no")), 0.0);

        // StringSimilarity: identical -> 1.0; very different -> strictly lower.
        let ss = StringSimilarityMetric::new();
        let id_score = numeric(&ss.score(&matching).await.expect("ss identical"));
        let diff_score = numeric(&ss.score(&different).await.expect("ss different"));
        assert!((id_score - 1.0).abs() < 1e-9);
        assert!(diff_score < id_score);
    }

    #[tokio::test]
    async fn deterministic_metrics_require_a_reference() {
        let no_ref = SingleTurnSample::new("q", "resp", vec!["c".to_string()]);
        assert!(ExactMatchMetric::new().score(&no_ref).await.is_err());
        assert!(StringPresenceMetric::new().score(&no_ref).await.is_err());
        assert!(StringSimilarityMetric::new().score(&no_ref).await.is_err());
    }

    #[tokio::test]
    async fn string_similarity_metric_honors_distance_measure() {
        // A transposition: Levenshtein needs 2 edits (1 - 2/6), Jaro scores it 17/18.
        let sample =
            SingleTurnSample::new("q", "MARTHA", vec!["c".to_string()]).with_reference("MARHTA");
        let levenshtein = numeric(
            &StringSimilarityMetric::new()
                .score(&sample)
                .await
                .expect("levenshtein"),
        );
        let jaro = numeric(
            &StringSimilarityMetric::new()
                .with_distance_measure(crate::DistanceMeasure::Jaro)
                .score(&sample)
                .await
                .expect("jaro"),
        );
        assert!((levenshtein - (1.0 - 2.0 / 6.0)).abs() < 1e-9);
        assert!((jaro - 17.0 / 18.0).abs() < 1e-9);
        assert!(jaro > levenshtein);
    }

    #[tokio::test]
    async fn bleu_and_chrf_metrics_discriminate_match_from_mismatch() {
        let identical =
            SingleTurnSample::new("q", "the quick brown fox jumps", vec!["c".to_string()])
                .with_reference("the quick brown fox jumps");
        let disjoint =
            SingleTurnSample::new("q", "alpha beta gamma delta epsilon", vec!["c".to_string()])
                .with_reference("the quick brown fox jumps");

        // BLEU-4: identical multi-token text -> 1.0; disjoint -> 0.0.
        let bleu = BleuScoreMetric::new();
        assert_eq!(bleu.name(), "bleu_score");
        assert!((numeric(&bleu.score(&identical).await.expect("bleu id")) - 1.0).abs() < 1e-9);
        assert_eq!(
            numeric(&bleu.score(&disjoint).await.expect("bleu disjoint")),
            0.0
        );

        // chrF: identical -> 1.0; a near string scores strictly higher than a disjoint one.
        let chrf = ChrfScoreMetric::new();
        assert_eq!(chrf.name(), "chrf");
        assert!((numeric(&chrf.score(&identical).await.expect("chrf id")) - 1.0).abs() < 1e-9);
        let near = SingleTurnSample::new("q", "the quick brown fox", vec!["c".to_string()])
            .with_reference("the quick brown fox jumps");
        let far = SingleTurnSample::new("q", "zzzzzzzz", vec!["c".to_string()])
            .with_reference("the quick brown fox jumps");
        let near_score = numeric(&chrf.score(&near).await.expect("chrf near"));
        let far_score = numeric(&chrf.score(&far).await.expect("chrf far"));
        assert!(near_score > far_score, "near={near_score} far={far_score}");
    }

    #[test]
    fn bleu_and_chrf_functions_handle_edges() {
        use crate::{bleu_score, chrf};
        // Both empty -> 1.0 (perfect match of nothing); one empty -> 0.0.
        assert_eq!(bleu_score("", "").score.unwrap(), 1.0);
        assert_eq!(chrf("", "").score.unwrap(), 1.0);
        assert_eq!(bleu_score("hello world", "").score.unwrap(), 0.0);
        assert_eq!(chrf("hello", "").score.unwrap(), 0.0);
        // Candidate shorter than the 4-gram order -> BLEU-4 is 0 (no smoothing).
        assert_eq!(
            bleu_score("two tokens", "two tokens only here")
                .score
                .unwrap(),
            0.0
        );
        // chrF on an identical short string -> 1.0 (longer orders are skipped gracefully).
        assert!((chrf("abc", "abc").score.unwrap() - 1.0).abs() < 1e-9);
    }

    // ---------------------------------------------------------------------
    // Phase 3 metrics — unit discrimination (ScriptedLlm, offline).
    // ---------------------------------------------------------------------

    #[tokio::test]
    async fn factual_correctness_discriminates_via_tp_fp_fn() {
        let sample =
            SingleTurnSample::new("q", "resp", vec!["c".to_string()]).with_reference("ref");

        // TP=2,FP=0,FN=0 -> 2*2/(4+0+0) = 1.0.
        let correct = FactualCorrectnessMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"TP":["a","b"],"FP":[],"FN":[]}"#,
        ])));
        assert!((numeric(&correct.score(&sample).await.expect("correct")) - 1.0).abs() < 1e-9);

        // TP=0,FP=2,FN=2 -> 0.0.
        let wrong = FactualCorrectnessMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"TP":[],"FP":["x","y"],"FN":["a","b"]}"#,
        ])));
        assert_eq!(numeric(&wrong.score(&sample).await.expect("wrong")), 0.0);

        // TP=1,FP=1,FN=1 -> 2/(2+1+1) = 0.5.
        let partial = FactualCorrectnessMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"TP":["a"],"FP":["x"],"FN":["b"]}"#,
        ])));
        assert!((numeric(&partial.score(&sample).await.expect("partial")) - 0.5).abs() < 1e-9);
    }

    #[tokio::test]
    async fn context_entity_recall_scores_entity_overlap() {
        let sample =
            SingleTurnSample::new("q", "resp", vec!["ctx".to_string()]).with_reference("ref");

        // reference {paris, france}; contexts {paris, eiffel tower} -> 1/2 = 0.5.
        let half = ContextEntityRecallMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"entities":["Paris","France"]}"#,
            r#"{"entities":["Paris","Eiffel Tower"]}"#,
        ])));
        assert!((numeric(&half.score(&sample).await.expect("half")) - 0.5).abs() < 1e-9);

        // all reference entities recalled (case-insensitive) -> 1.0.
        let full = ContextEntityRecallMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"entities":["Paris"]}"#,
            r#"{"entities":["paris","london"]}"#,
        ])));
        assert_eq!(numeric(&full.score(&sample).await.expect("full")), 1.0);
    }

    #[tokio::test]
    async fn nv_dual_raters_average_two_passes() {
        let sample = SingleTurnSample::new("q", "resp", vec!["ctx".to_string()]);

        // context_relevance: ratings 4,4 -> (1.0 + 1.0)/2 = 1.0.
        let relevant = ContextRelevanceMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"rating":4}"#,
            r#"{"rating":4}"#,
        ])));
        assert_eq!(numeric(&relevant.score(&sample).await.expect("cr")), 1.0);

        // response_groundedness: ratings 0,2 -> (0.0 + 0.5)/2 = 0.25.
        let grounded = ResponseGroundednessMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"rating":0}"#,
            r#"{"rating":2}"#,
        ])));
        assert!((numeric(&grounded.score(&sample).await.expect("rg")) - 0.25).abs() < 1e-9);

        // No contexts -> 0.0 without any LLM call.
        let llm = Arc::new(ScriptedLlm::new(vec![]));
        let empty = ContextRelevanceMetric::new(llm.clone());
        let no_ctx = SingleTurnSample::new("q", "resp", Vec::new());
        assert_eq!(numeric(&empty.score(&no_ctx).await.expect("empty")), 0.0);
        assert!(llm.prompts().is_empty());
    }

    #[tokio::test]
    async fn rubrics_score_returns_the_rubric_integer() {
        let rubric = vec![(1, "poor".to_string()), (5, "excellent".to_string())];
        let metric = RubricsScoreMetric::new(
            "helpfulness",
            rubric,
            Arc::new(ScriptedLlm::new(vec![r#"{"score":4,"feedback":"good"}"#])),
        );
        let sample = SingleTurnSample::new("q", "resp", vec!["ctx".to_string()]);
        let result = metric.score(&sample).await.expect("rubric");
        assert_eq!(result.metric_name, "helpfulness");
        assert_eq!(numeric(&result), 4.0);
    }

    // ---------------------------------------------------------------------
    // Phase 3 metrics — LIVE discrimination gates (IGNORED unless a key is set).
    // ---------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_factual_correctness_scores_correct_above_incorrect() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = FactualCorrectnessMetric::new(client);
        let correct = SingleTurnSample::new(
            "What is the capital of France?",
            "The capital of France is Paris, a city on the Seine.",
            vec!["n/a".to_string()],
        )
        .with_reference("Paris is the capital of France and sits on the Seine river.");
        let incorrect = SingleTurnSample::new(
            "What is the capital of France?",
            "The capital of France is Berlin, a city in Bavaria.",
            vec!["n/a".to_string()],
        )
        .with_reference("Paris is the capital of France and sits on the Seine river.");

        let correct_score = numeric(&metric.score(&correct).await.expect("correct"));
        let incorrect_score = numeric(&metric.score(&incorrect).await.expect("incorrect"));
        assert!(
            correct_score > incorrect_score,
            "correct={correct_score} incorrect={incorrect_score}"
        );
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_context_entity_recall_scores_present_above_absent() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = ContextEntityRecallMetric::new(client);
        let reference = "Albert Einstein was born in Ulm, Germany in 1879.";
        let present = SingleTurnSample::new(
            "q",
            "answer",
            vec![
                "Albert Einstein was born in Ulm, Germany in 1879 and later moved to Switzerland."
                    .to_string(),
            ],
        )
        .with_reference(reference);
        let absent = SingleTurnSample::new(
            "q",
            "answer",
            vec![
                "Photosynthesis is the process by which plants convert sunlight into energy."
                    .to_string(),
            ],
        )
        .with_reference(reference);

        let present_score = numeric(&metric.score(&present).await.expect("present"));
        let absent_score = numeric(&metric.score(&absent).await.expect("absent"));
        assert!(
            present_score > absent_score,
            "present={present_score} absent={absent_score}"
        );
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_context_relevance_scores_relevant_above_irrelevant() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = ContextRelevanceMetric::new(client);
        let question = "What is the capital of France?";
        let relevant = SingleTurnSample::new(
            question,
            "answer",
            vec!["Paris is the capital and largest city of France.".to_string()],
        );
        let irrelevant = SingleTurnSample::new(
            question,
            "answer",
            vec!["The blue whale is the largest animal known to have existed.".to_string()],
        );

        let relevant_score = numeric(&metric.score(&relevant).await.expect("relevant"));
        let irrelevant_score = numeric(&metric.score(&irrelevant).await.expect("irrelevant"));
        assert!(
            relevant_score > irrelevant_score,
            "relevant={relevant_score} irrelevant={irrelevant_score}"
        );
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_response_groundedness_scores_grounded_above_ungrounded() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = ResponseGroundednessMetric::new(client);
        let contexts =
            vec!["Ragas is an open-source framework to evaluate LLM applications.".to_string()];
        let grounded = SingleTurnSample::new(
            "What is Ragas?",
            "Ragas is a framework for evaluating LLM applications.",
            contexts.clone(),
        );
        let ungrounded = SingleTurnSample::new(
            "What is Ragas?",
            "Ragas is a brand of Italian sports cars founded in 1947.",
            contexts,
        );

        let grounded_score = numeric(&metric.score(&grounded).await.expect("grounded"));
        let ungrounded_score = numeric(&metric.score(&ungrounded).await.expect("ungrounded"));
        assert!(
            grounded_score > ungrounded_score,
            "grounded={grounded_score} ungrounded={ungrounded_score}"
        );
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_rubrics_score_scores_better_answer_higher() {
        let Some(client) = live_client() else {
            return;
        };
        let rubric = vec![
            (1, "completely incorrect".to_string()),
            (3, "partially correct".to_string()),
            (5, "fully correct and complete".to_string()),
        ];
        let metric = RubricsScoreMetric::new("correctness", rubric, client);
        let good = SingleTurnSample::new(
            "What is the boiling point of water at sea level in Celsius?",
            "Water boils at 100 degrees Celsius at sea level.",
            vec!["n/a".to_string()],
        );
        let bad = SingleTurnSample::new(
            "What is the boiling point of water at sea level in Celsius?",
            "Water boils at 7 degrees Celsius at sea level.",
            vec!["n/a".to_string()],
        );

        let good_score = numeric(&metric.score(&good).await.expect("good"));
        let bad_score = numeric(&metric.score(&bad).await.expect("bad"));
        assert!(good_score > bad_score, "good={good_score} bad={bad_score}");
    }

    #[tokio::test]
    async fn instance_specific_rubrics_scores_from_per_sample_rubric() {
        let sample = SingleTurnSample::new("What is 2 + 2?", "4", vec![]).with_rubrics(vec![
            (1, "incorrect".to_string()),
            (5, "correct".to_string()),
        ]);
        let metric = InstanceSpecificRubricsMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"score":5,"feedback":"the answer is correct"}"#,
        ])));
        let result = metric.score(&sample).await.expect("score");
        assert_eq!(result.metric_name, "instance_specific_rubrics");
        assert_eq!(numeric(&result), 5.0);
        assert_eq!(result.reason.as_deref(), Some("the answer is correct"));
    }

    #[tokio::test]
    async fn instance_specific_rubrics_requires_a_per_sample_rubric() {
        let no_rubric = SingleTurnSample::new("q", "a", vec![]);
        let metric = InstanceSpecificRubricsMetric::new(Arc::new(ScriptedLlm::new(Vec::new())));
        assert!(metric.score(&no_rubric).await.is_err());
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_instance_specific_rubrics_scores_better_answer_higher() {
        let Some(client) = live_client() else {
            return;
        };
        let rubric = vec![
            (1, "completely incorrect".to_string()),
            (3, "partially correct".to_string()),
            (5, "fully correct and complete".to_string()),
        ];
        let metric = InstanceSpecificRubricsMetric::new(client);
        let good = SingleTurnSample::new(
            "What is the boiling point of water at sea level in Celsius?",
            "Water boils at 100 degrees Celsius at sea level.",
            vec![],
        )
        .with_rubrics(rubric.clone());
        let bad = SingleTurnSample::new(
            "What is the boiling point of water at sea level in Celsius?",
            "Water boils at 7 degrees Celsius at sea level.",
            vec![],
        )
        .with_rubrics(rubric);

        let good_score = numeric(&metric.score(&good).await.expect("good"));
        let bad_score = numeric(&metric.score(&bad).await.expect("bad"));
        assert!(good_score > bad_score, "good={good_score} bad={bad_score}");
    }

    #[tokio::test]
    async fn summarization_score_covers_fraction_of_questions() {
        let sample = SingleTurnSample::new("q", "summary text", vec!["source".to_string()]);

        // 2 questions, both answerable from the summary -> 1.0.
        let full = SummarizationScoreMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"questions":["q1","q2"]}"#,
            r#"{"answers":[{"verdict":1},{"verdict":1}]}"#,
        ])));
        assert_eq!(numeric(&full.score(&sample).await.expect("full")), 1.0);

        // 1 of 2 -> 0.5.
        let half = SummarizationScoreMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"questions":["q1","q2"]}"#,
            r#"{"answers":[{"verdict":1},{"verdict":0}]}"#,
        ])));
        assert_eq!(numeric(&half.score(&sample).await.expect("half")), 0.5);
    }

    #[tokio::test]
    async fn summarization_requires_source_contexts() {
        let no_ctx = SingleTurnSample::new("q", "summary", Vec::new());
        let metric = SummarizationScoreMetric::new(Arc::new(ScriptedLlm::new(vec![])));
        assert!(metric.score(&no_ctx).await.is_err());
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_summarization_score_scores_faithful_summary_above_off_topic() {
        let Some(client) = live_client() else {
            return;
        };
        let metric = SummarizationScoreMetric::new(client);
        let source = vec![
            "The Apollo 11 mission landed the first humans on the Moon on July 20, 1969. Neil \
Armstrong and Buzz Aldrin walked on the lunar surface while Michael Collins orbited above in the \
command module."
                .to_string(),
        ];
        let good = SingleTurnSample::new(
            "Summarize the source.",
            "Apollo 11 landed the first humans on the Moon in 1969; Armstrong and Aldrin walked on \
the surface while Collins stayed in orbit.",
            source.clone(),
        );
        let bad = SingleTurnSample::new(
            "Summarize the source.",
            "The recipe calls for two cups of flour and a pinch of salt.",
            source,
        );

        let good_score = numeric(&metric.score(&good).await.expect("good"));
        let bad_score = numeric(&metric.score(&bad).await.expect("bad"));
        assert!(good_score > bad_score, "good={good_score} bad={bad_score}");
    }

    #[tokio::test]
    async fn noise_sensitivity_counts_incorrect_claims_grounded_in_relevant_contexts() {
        let sample =
            SingleTurnSample::new("q", "resp", vec!["ctx1".to_string(), "ctx2".to_string()])
                .with_reference("ref");
        // statements [a,b]; correctness [1,0]; relevance [1,0] (ctx1 relevant);
        // grounded vs relevant ctx [0,1] -> only statement b is incorrect AND grounded -> 1/2 = 0.5.
        let metric = NoiseSensitivityMetric::new(Arc::new(ScriptedLlm::new(vec![
            r#"{"statements":["a","b"]}"#,
            r#"{"verdicts":[{"verdict":1},{"verdict":0}]}"#,
            r#"{"verdicts":[{"verdict":1},{"verdict":0}]}"#,
            r#"{"verdicts":[{"verdict":0},{"verdict":1}]}"#,
        ])));
        assert_eq!(metric.name(), "noise_sensitivity_relevant");
        assert!((numeric(&metric.score(&sample).await.expect("ns")) - 0.5).abs() < 1e-9);
    }

    #[tokio::test]
    async fn noise_sensitivity_zero_when_no_target_contexts() {
        let sample =
            SingleTurnSample::new("q", "resp", vec!["ctx1".to_string(), "ctx2".to_string()])
                .with_reference("ref");
        // All contexts irrelevant -> Relevant mode has an empty target subset -> 0, and the
        // grounding call is skipped (only decompose + correctness + relevance run).
        let llm = Arc::new(ScriptedLlm::new(vec![
            r#"{"statements":["a","b"]}"#,
            r#"{"verdicts":[{"verdict":0},{"verdict":0}]}"#,
            r#"{"verdicts":[{"verdict":0},{"verdict":0}]}"#,
        ]));
        let metric = NoiseSensitivityMetric::new(llm.clone());
        assert_eq!(numeric(&metric.score(&sample).await.expect("ns")), 0.0);
        assert_eq!(llm.prompts().len(), 3);
    }

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY; live LLM discrimination gate"]
    async fn live_noise_sensitivity_flags_noisy_response_above_clean() {
        let Some(client) = live_client() else {
            return;
        };
        let metric =
            NoiseSensitivityMetric::new(client).with_mode(NoiseSensitivityMode::Irrelevant);
        let reference = "The Eiffel Tower is located in Paris.";
        let contexts = vec![
            "The Eiffel Tower is located in Paris, France.".to_string(),
            "The Statue of Liberty is located in New York, USA.".to_string(),
        ];
        let clean = SingleTurnSample::new(
            "Where is the Eiffel Tower?",
            "The Eiffel Tower is in Paris.",
            contexts.clone(),
        )
        .with_reference(reference);
        let noisy = SingleTurnSample::new(
            "Where is the Eiffel Tower?",
            "The Eiffel Tower is in Paris. The Statue of Liberty is in New York.",
            contexts,
        )
        .with_reference(reference);

        let clean_score = numeric(&metric.score(&clean).await.expect("clean"));
        let noisy_score = numeric(&metric.score(&noisy).await.expect("noisy"));
        assert!(
            noisy_score > clean_score,
            "noisy={noisy_score} clean={clean_score}"
        );
    }

    // ---------------------------------------------------------------------
    // NonLLM context precision/recall — deterministic, use reference_contexts.
    // ---------------------------------------------------------------------

    #[tokio::test]
    async fn non_llm_context_precision_and_recall_use_string_similarity() {
        let sample = SingleTurnSample::new(
            "q",
            "resp",
            vec![
                "the quick brown fox".to_string(),
                "totally unrelated content here".to_string(),
            ],
        )
        .with_reference_contexts(vec!["the quick brown fox".to_string()]);

        // retrieved[0] matches the reference (sim 1.0), retrieved[1] does not -> [1,0] -> AP = 1.0.
        let precision = NonLlmContextPrecisionMetric::new();
        assert_eq!(
            numeric(&precision.score(&sample).await.expect("precision")),
            1.0
        );

        // the single reference context is covered by retrieved[0] -> recall 1/1 = 1.0.
        let recall = NonLlmContextRecallMetric::new();
        assert_eq!(numeric(&recall.score(&sample).await.expect("recall")), 1.0);
    }

    #[tokio::test]
    async fn non_llm_context_metrics_discriminate_and_require_reference_contexts() {
        // No reference_contexts -> error.
        let no_ref = SingleTurnSample::new("q", "resp", vec!["c".to_string()]);
        assert!(
            NonLlmContextPrecisionMetric::new()
                .score(&no_ref)
                .await
                .is_err()
        );
        assert!(
            NonLlmContextRecallMetric::new()
                .score(&no_ref)
                .await
                .is_err()
        );

        // Retrieved context unrelated to the reference -> precision and recall both 0.
        let miss = SingleTurnSample::new("q", "resp", vec!["apples and oranges".to_string()])
            .with_reference_contexts(vec!["the quick brown fox".to_string()]);
        assert_eq!(
            numeric(
                &NonLlmContextPrecisionMetric::new()
                    .score(&miss)
                    .await
                    .expect("p")
            ),
            0.0
        );
        assert_eq!(
            numeric(
                &NonLlmContextRecallMetric::new()
                    .score(&miss)
                    .await
                    .expect("r")
            ),
            0.0
        );
    }

    // ---------------------------------------------------------------------
    // DataCompyScore — deterministic CSV row comparison.
    // ---------------------------------------------------------------------

    #[tokio::test]
    async fn datacompy_score_compares_csv_rows() {
        // Identical CSV tables -> precision = recall = F1 = 1.0.
        let identical =
            SingleTurnSample::new("q", "id,name\n1,alice\n2,bob", vec!["c".to_string()])
                .with_reference("id,name\n1,alice\n2,bob");
        assert!(
            (numeric(
                &DataCompyScoreMetric::new()
                    .score(&identical)
                    .await
                    .expect("identical")
            ) - 1.0)
                .abs()
                < 1e-9
        );

        // Response = reference rows + 1 extra row: 4 response rows, 3 reference rows, 3 matched.
        // precision = 3/4 = 0.75; recall = 3/3 = 1.0.
        let partial = SingleTurnSample::new("q", "id,name\n1,a\n2,b\n3,c", vec!["c".to_string()])
            .with_reference("id,name\n1,a\n2,b");
        let precision = numeric(
            &DataCompyScoreMetric::new()
                .with_mode(DataCompyMode::Precision)
                .score(&partial)
                .await
                .expect("precision"),
        );
        let recall = numeric(
            &DataCompyScoreMetric::new()
                .with_mode(DataCompyMode::Recall)
                .score(&partial)
                .await
                .expect("recall"),
        );
        assert!((precision - 0.75).abs() < 1e-9);
        assert!((recall - 1.0).abs() < 1e-9);

        // Disjoint tables -> 0.
        let disjoint = SingleTurnSample::new("q", "x,y\n9,z", vec!["c".to_string()])
            .with_reference("id,name\n1,a");
        assert_eq!(
            numeric(
                &DataCompyScoreMetric::new()
                    .score(&disjoint)
                    .await
                    .expect("disjoint")
            ),
            0.0
        );
    }

    #[tokio::test]
    async fn datacompy_requires_a_reference() {
        let no_ref = SingleTurnSample::new("q", "a,b\n1,2", vec!["c".to_string()]);
        assert!(DataCompyScoreMetric::new().score(&no_ref).await.is_err());
    }

    // ---------------------------------------------------------------------
    // ID-based context precision/recall — deterministic, use the *_context_ids fields.
    // ---------------------------------------------------------------------

    #[tokio::test]
    async fn id_based_context_metrics_score_id_overlap() {
        // retrieved [c1,c2,c3], reference [c1,c2]:
        //   precision = retrieved ids in reference / retrieved = 2/3; recall = matched / reference = 1.0.
        let sample = SingleTurnSample::new("q", "resp", vec!["c".to_string()]).with_context_ids(
            vec!["c1".to_string(), "c2".to_string(), "c3".to_string()],
            vec!["c1".to_string(), "c2".to_string()],
        );
        let precision = numeric(
            &IdBasedContextPrecisionMetric::new()
                .score(&sample)
                .await
                .expect("precision"),
        );
        let recall = numeric(
            &IdBasedContextRecallMetric::new()
                .score(&sample)
                .await
                .expect("recall"),
        );
        assert!((precision - 2.0 / 3.0).abs() < 1e-9);
        assert!((recall - 1.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn id_based_context_recall_misses_unretrieved_ids() {
        // retrieved [c1], reference [c1,c2,c3] -> recall 1/3.
        let sample = SingleTurnSample::new("q", "resp", vec!["c".to_string()]).with_context_ids(
            vec!["c1".to_string()],
            vec!["c1".to_string(), "c2".to_string(), "c3".to_string()],
        );
        assert!(
            (numeric(
                &IdBasedContextRecallMetric::new()
                    .score(&sample)
                    .await
                    .expect("recall")
            ) - 1.0 / 3.0)
                .abs()
                < 1e-9
        );
    }

    #[tokio::test]
    async fn id_based_context_metrics_require_reference_ids() {
        let no_ids = SingleTurnSample::new("q", "resp", vec!["c".to_string()]);
        assert!(
            IdBasedContextPrecisionMetric::new()
                .score(&no_ids)
                .await
                .is_err()
        );
        assert!(
            IdBasedContextRecallMetric::new()
                .score(&no_ids)
                .await
                .is_err()
        );
    }
}
