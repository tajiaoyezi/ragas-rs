use std::collections::BTreeSet;

use crate::{
    DetailedMetricResult, FewShotExample, JudgeOutputParser, MetricEvidence, MetricValueType,
    OutputParseDiagnostic, PromptTemplate, PromptValueKind, PromptVariables, RagasError,
    RenderedPrompt, ScoreNormalizationPolicy, SingleTurnSample, cosine_similarity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPrecisionVariant {
    RankedRelevance,
    IdOverlap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactualCorrectnessCounts {
    pub true_positive: usize,
    pub false_positive: usize,
    pub false_negative: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnswerCorrectnessWeights {
    pub semantic: f64,
    pub factual: f64,
}

impl AnswerCorrectnessWeights {
    pub fn new(semantic: f64, factual: f64) -> Self {
        Self { semantic, factual }
    }
}

impl FactualCorrectnessCounts {
    pub fn new(true_positive: usize, false_positive: usize, false_negative: usize) -> Self {
        Self {
            true_positive,
            false_positive,
            false_negative,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaithfulnessJudgeContract;

impl Default for FaithfulnessJudgeContract {
    fn default() -> Self {
        Self::new()
    }
}

impl FaithfulnessJudgeContract {
    pub fn new() -> Self {
        Self
    }

    pub fn render_prompt(&self, sample: &SingleTurnSample) -> Result<RenderedPrompt, RagasError> {
        let template = PromptTemplate::new(
            "faithfulness",
            "Question: {{question}}\nResponse: {{response}}\nContexts:\n{{contexts}}\nReturn JSON with numeric score and reason.",
        )
        .require_variable("question", PromptValueKind::Text)
        .require_variable("response", PromptValueKind::Text)
        .require_variable("contexts", PromptValueKind::Text)
        .with_few_shot(FewShotExample::new(
            PromptVariables::new()
                .with_text("question", "What does ragas evaluate?")
                .with_text("response", "Ragas evaluates LLM applications.")
                .with_text("contexts", "Ragas evaluates LLM applications."),
            r#"{"score":1.0,"reason":"response is supported by context"}"#,
        ));

        template.render(
            &PromptVariables::new()
                .with_text("question", &sample.user_input)
                .with_text("response", &sample.response)
                .with_text("contexts", sample.retrieved_contexts.join("\n")),
        )
    }

    pub fn parse_judge_output(
        &self,
        output: &str,
    ) -> Result<DetailedMetricResult, OutputParseDiagnostic> {
        let parsed = JudgeOutputParser::new().parse(output)?;
        let reason = parsed
            .reason
            .unwrap_or_else(|| "faithfulness judge returned a score".to_string());
        Ok(numeric_result(
            "faithfulness",
            parsed.score,
            reason,
            Vec::new(),
        ))
    }
}

pub fn response_groundedness(response: &str, contexts: &[String]) -> DetailedMetricResult {
    let claims = split_reference_claims(response);
    if claims.is_empty() {
        return numeric_result(
            "response_groundedness",
            0.0,
            "response groundedness has no response claims",
            Vec::new(),
        );
    }

    let mut grounded = 0usize;
    let mut evidence = Vec::new();
    for claim in &claims {
        if let Some((context_index, context)) = contexts
            .iter()
            .enumerate()
            .find(|(_, context)| claim_is_supported_by_context(claim, context))
        {
            grounded += 1;
            evidence.push(MetricEvidence::new(
                format!("context[{context_index}]"),
                context.clone(),
            ));
        }
    }

    numeric_result(
        "response_groundedness",
        grounded as f64 / claims.len() as f64,
        "response claims grounded in retrieved contexts",
        evidence,
    )
}

pub fn factual_correctness(counts: FactualCorrectnessCounts) -> DetailedMetricResult {
    let score = factual_score(&counts);
    numeric_result(
        "factual_correctness",
        score,
        format!(
            "F1 from TP={} FP={} FN={}",
            counts.true_positive, counts.false_positive, counts.false_negative
        ),
        vec![MetricEvidence::new(
            "factual_counts",
            format!(
                "TP={} FP={} FN={}",
                counts.true_positive, counts.false_positive, counts.false_negative
            ),
        )],
    )
}

fn factual_score(counts: &FactualCorrectnessCounts) -> f64 {
    let numerator = 2 * counts.true_positive;
    let denominator = numerator + counts.false_positive + counts.false_negative;
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

pub fn answer_relevancy_from_embedding_similarity(
    question_embedding: &[f32],
    answer_embedding: &[f32],
) -> DetailedMetricResult {
    let score = cosine_similarity(question_embedding, answer_embedding).clamp(0.0, 1.0);
    numeric_result(
        "answer_relevancy",
        score,
        "embedding cosine similarity between question and answer",
        vec![MetricEvidence::new(
            "embedding_similarity",
            format!("cosine={score:.6}"),
        )],
    )
}

pub fn answer_relevancy_from_judge_output(
    output: &str,
) -> Result<DetailedMetricResult, OutputParseDiagnostic> {
    let parsed = JudgeOutputParser::new().parse(output)?;
    let reason = parsed
        .reason
        .unwrap_or_else(|| "answer relevancy judge returned a score".to_string());
    Ok(numeric_result(
        "answer_relevancy",
        parsed.score,
        reason,
        Vec::new(),
    ))
}

pub fn answer_correctness(
    semantic_similarity: f64,
    factual_counts: FactualCorrectnessCounts,
    weights: AnswerCorrectnessWeights,
) -> DetailedMetricResult {
    let semantic = semantic_similarity.clamp(0.0, 1.0);
    let factual = factual_score(&factual_counts);
    let total_weight = weights.semantic + weights.factual;
    let score = if total_weight <= 0.0 {
        0.0
    } else {
        ((semantic * weights.semantic) + (factual * weights.factual)) / total_weight
    };

    numeric_result(
        "answer_correctness",
        score,
        format!("weighted answer correctness from semantic={semantic:.3} factual={factual:.3}"),
        vec![
            MetricEvidence::new("semantic_similarity", format!("{semantic:.6}")),
            MetricEvidence::new(
                "factual_correctness",
                format!(
                    "TP={} FP={} FN={} factual_f1={factual:.6}",
                    factual_counts.true_positive,
                    factual_counts.false_positive,
                    factual_counts.false_negative
                ),
            ),
        ],
    )
}

pub fn noise_sensitivity(clean_score: f64, noisy_score: f64) -> DetailedMetricResult {
    let clean = clean_score.clamp(0.0, 1.0);
    let noisy = noisy_score.clamp(0.0, 1.0);
    let score = (clean - noisy).max(0.0);
    numeric_result(
        "noise_sensitivity",
        score,
        format!("score drop under noise: clean={clean:.3} noisy={noisy:.3}"),
        vec![
            MetricEvidence::new("clean_score", format!("{clean:.6}")),
            MetricEvidence::new("noisy_score", format!("{noisy:.6}")),
        ],
    )
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

pub fn context_precision_from_relevance(relevance: &[bool]) -> DetailedMetricResult {
    let mut relevant_seen = 0usize;
    let mut precision_sum = 0.0f64;
    let mut evidence = Vec::new();

    for (index, is_relevant) in relevance.iter().copied().enumerate() {
        if is_relevant {
            relevant_seen += 1;
            let rank = index + 1;
            let precision_at_k = relevant_seen as f64 / rank as f64;
            precision_sum += precision_at_k;
            evidence.push(MetricEvidence::new(
                format!("context[{index}]"),
                format!("relevant at rank {rank}; precision@{rank}={precision_at_k:.6}"),
            ));
        }
    }

    let score = if relevant_seen == 0 {
        0.0
    } else {
        precision_sum / relevant_seen as f64
    };

    numeric_result(
        "context_precision",
        score,
        "ranked precision@k average over relevant contexts",
        evidence,
    )
}

pub fn id_based_context_precision<R, G>(
    retrieved_context_ids: &[R],
    reference_context_ids: &[G],
) -> DetailedMetricResult
where
    R: AsRef<str>,
    G: AsRef<str>,
{
    if retrieved_context_ids.is_empty() {
        return numeric_result(
            "id_based_context_precision",
            0.0,
            "id overlap precision has no retrieved ids",
            Vec::new(),
        );
    }

    let reference_ids = reference_context_ids
        .iter()
        .map(|id| id.as_ref().trim().to_string())
        .filter(|id| !id.is_empty())
        .collect::<BTreeSet<_>>();

    let mut overlap = 0usize;
    let mut evidence = Vec::new();
    for (index, id) in retrieved_context_ids.iter().enumerate() {
        let id = id.as_ref().trim();
        if reference_ids.contains(id) {
            overlap += 1;
            evidence.push(MetricEvidence::new(
                format!("retrieved_context_id[{index}]"),
                id.to_string(),
            ));
        }
    }

    numeric_result(
        "id_based_context_precision",
        overlap as f64 / retrieved_context_ids.len() as f64,
        "id overlap precision: retrieved reference id overlap / retrieved ids",
        evidence,
    )
}

pub fn context_recall(reference: &str, contexts: &[String]) -> DetailedMetricResult {
    let claims = split_reference_claims(reference);
    if claims.is_empty() {
        return numeric_result(
            "context_recall",
            0.0,
            "context recall has no reference claims",
            Vec::new(),
        );
    }

    let context_tokens = contexts
        .iter()
        .flat_map(|context| meaningful_tokens(context))
        .collect::<BTreeSet<_>>();
    let mut covered = 0usize;
    let mut evidence = Vec::new();

    for (index, claim) in claims.iter().enumerate() {
        let claim_tokens = meaningful_tokens(claim);
        if !claim_tokens.is_empty()
            && claim_tokens
                .iter()
                .all(|token| context_tokens.contains(token))
        {
            covered += 1;
            evidence.push(MetricEvidence::new(
                format!("reference[{index}]"),
                claim.clone(),
            ));
        }
    }

    numeric_result(
        "context_recall",
        covered as f64 / claims.len() as f64,
        "reference claims covered by retrieved contexts",
        evidence,
    )
}

pub fn context_entity_recall(reference: &str, contexts: &[String]) -> DetailedMetricResult {
    let entities = extract_entities(reference);
    if entities.is_empty() {
        return numeric_result(
            "context_entity_recall",
            0.0,
            "entity recall has no reference entities",
            Vec::new(),
        );
    }

    let context_tokens = contexts
        .iter()
        .flat_map(|context| meaningful_tokens(context))
        .collect::<BTreeSet<_>>();
    let mut covered = 0usize;
    let mut evidence = Vec::new();

    for entity in &entities {
        let normalized = normalize_token(entity);
        if context_tokens.contains(&normalized) {
            covered += 1;
            evidence.push(MetricEvidence::new(
                format!("entity:{entity}"),
                "matched in retrieved contexts",
            ));
        }
    }

    numeric_result(
        "context_entity_recall",
        covered as f64 / entities.len() as f64,
        "reference entities covered by retrieved contexts",
        evidence,
    )
}

pub fn context_relevance(user_input: &str, contexts: &[String]) -> DetailedMetricResult {
    if contexts.is_empty() {
        return numeric_result(
            "context_relevance",
            0.0,
            "context relevance has no retrieved contexts",
            Vec::new(),
        );
    }

    let query_tokens = meaningful_tokens(user_input)
        .into_iter()
        .collect::<BTreeSet<_>>();
    if query_tokens.is_empty() {
        return numeric_result(
            "context_relevance",
            0.0,
            "context relevance has no usable query tokens",
            Vec::new(),
        );
    }

    let mut overlap_ratio_sum = 0.0f64;
    let mut evidence = Vec::new();
    for (index, context) in contexts.iter().enumerate() {
        let context_tokens = meaningful_tokens(context)
            .into_iter()
            .collect::<BTreeSet<_>>();
        let overlap = query_tokens
            .iter()
            .filter(|token| context_tokens.contains(*token))
            .count();
        if overlap > 0 {
            evidence.push(MetricEvidence::new(
                format!("context[{index}]"),
                context.clone(),
            ));
        }
        overlap_ratio_sum += overlap as f64 / query_tokens.len() as f64;
    }

    numeric_result(
        "context_relevance",
        overlap_ratio_sum / contexts.len() as f64,
        "lexical overlap between user_input and retrieved contexts",
        evidence,
    )
}

fn numeric_result(
    metric_name: &str,
    score: f64,
    reason: impl Into<String>,
    evidence: Vec<MetricEvidence>,
) -> DetailedMetricResult {
    let mut result = DetailedMetricResult::new(metric_name, MetricValueType::Numeric)
        .with_score(score, ScoreNormalizationPolicy::Reject)
        .expect("deterministic RAG metric score is normalized")
        .with_reason(reason);
    for item in evidence {
        result = result.with_evidence(item);
    }
    result
}

fn split_reference_claims(reference: &str) -> Vec<String> {
    reference
        .split(['.', '?', '!', ';'])
        .map(str::trim)
        .filter(|claim| !claim.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn claim_is_supported_by_context(claim: &str, context: &str) -> bool {
    let claim_tokens = meaningful_tokens(claim);
    if claim_tokens.is_empty() {
        return false;
    }
    let context_tokens = meaningful_tokens(context)
        .into_iter()
        .collect::<BTreeSet<_>>();
    claim_tokens
        .iter()
        .all(|token| context_tokens.contains(token))
}

fn extract_entities(text: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut entities = Vec::new();
    for raw in text.split_whitespace() {
        let cleaned = raw
            .trim_matches(|character: char| !character.is_ascii_alphanumeric())
            .to_string();
        if cleaned.is_empty() {
            continue;
        }
        let starts_uppercase = cleaned
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_uppercase());
        let has_letter = cleaned
            .chars()
            .any(|character| character.is_ascii_alphabetic());
        if starts_uppercase && has_letter {
            let normalized = normalize_token(&cleaned);
            if seen.insert(normalized) {
                entities.push(cleaned);
            }
        }
    }
    entities
}

fn meaningful_tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(normalize_token)
        .filter(|token| token.len() > 1 && !is_stopword(token))
        .collect()
}

fn normalize_token(raw: &str) -> String {
    let mut token = raw
        .trim_matches(|character: char| !character.is_ascii_alphanumeric())
        .to_ascii_lowercase();
    if token.len() > 3 && token != "does" && token.ends_with('s') {
        token.pop();
    }
    token
}

fn is_stopword(token: &str) -> bool {
    matches!(
        token,
        "a" | "an"
            | "and"
            | "are"
            | "by"
            | "does"
            | "for"
            | "how"
            | "in"
            | "is"
            | "of"
            | "the"
            | "to"
            | "with"
    )
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
        assert!(
            ranked
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("precision@k")
        );

        let id_based =
            id_based_context_precision(&["doc-a", "doc-b", "doc-c"], &["doc-b", "doc-c", "doc-z"]);
        assert_score_close(&id_based, 2.0 / 3.0);
        assert!(
            id_based
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("id overlap")
        );
    }

    #[test]
    fn test_10_1_2_context_recall_and_entity_recall_use_reference_and_contexts() {
        // SCEN-10.1.2 / AC2 / TEST-10.1.2
        let contexts = vec![
            "Ragas evaluates LLM apps with metrics.".to_string(),
            "OpenAI embeddings score relevance for retrieval.".to_string(),
        ];
        let reference =
            "Ragas evaluates LLM apps. OpenAI embeddings score relevance. Rust services run fast.";

        let recall = context_recall(reference, &contexts);
        assert_score_close(&recall, 2.0 / 3.0);
        assert_eq!(recall.evidence.len(), 2);
        assert!(
            recall
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("reference claims")
        );

        let entity_recall = context_entity_recall("Ragas uses OpenAI with Rust.", &contexts);
        assert_score_close(&entity_recall, 2.0 / 3.0);
        assert!(
            entity_recall
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("entities")
        );
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
        assert!(
            relevance
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("lexical overlap")
        );
    }

    #[test]
    fn test_10_2_1_faithfulness_uses_prompt_parser_contract_from_phase_8() {
        // SCEN-10.2.1 / AC1 / TEST-10.2.1
        let contract = FaithfulnessJudgeContract::new();
        let sample = SingleTurnSample::new(
            "What does ragas evaluate?",
            "Ragas evaluates LLM applications.",
            vec!["Ragas evaluates LLM applications with metrics.".to_string()],
        );

        let rendered = contract.render_prompt(&sample).expect("rendered prompt");
        assert!(
            rendered
                .text
                .contains("Question: What does ragas evaluate?")
        );
        assert!(
            rendered
                .text
                .contains("Response: Ragas evaluates LLM applications.")
        );
        assert!(rendered.text.contains("Contexts:"));
        assert_eq!(rendered.few_shot_examples.len(), 1);

        let parsed = contract
            .parse_judge_output(r#"{"score":0.75,"reason":"supported by context"}"#)
            .expect("parsed judge output");
        assert_eq!(parsed.metric_name, "faithfulness");
        assert_score_close(&parsed, 0.75);
        assert_eq!(parsed.reason.as_deref(), Some("supported by context"));
    }

    #[test]
    fn test_10_2_2_response_groundedness_records_supporting_context_evidence() {
        // SCEN-10.2.2 / AC2 / TEST-10.2.2
        let contexts = vec![
            "Ragas evaluates LLM applications with metrics.".to_string(),
            "The framework can compare model outputs.".to_string(),
        ];
        let result = response_groundedness(
            "Ragas evaluates LLM applications. Rust services run fast.",
            &contexts,
        );

        assert_score_close(&result, 0.5);
        assert_eq!(result.metric_name, "response_groundedness");
        assert_eq!(result.evidence.len(), 1);
        assert_eq!(result.evidence[0].source, "context[0]");
        assert!(
            result
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("response claims")
        );
    }

    #[test]
    fn test_10_2_3_factual_correctness_handles_tp_fp_fn_style_output() {
        // SCEN-10.2.3 / AC3 / TEST-10.2.3
        let result = factual_correctness(FactualCorrectnessCounts::new(3, 1, 2));

        assert_score_close(&result, 6.0 / 9.0);
        assert_eq!(result.metric_name, "factual_correctness");
        assert!(result.reason.as_deref().unwrap_or("").contains("TP=3"));
        assert!(result.reason.as_deref().unwrap_or("").contains("FP=1"));
        assert!(result.reason.as_deref().unwrap_or("").contains("FN=2"));
    }

    #[test]
    fn test_10_3_1_answer_relevancy_supports_embedding_and_llm_judge_paths() {
        // SCEN-10.3.1 / AC1 / TEST-10.3.1
        let embedding = answer_relevancy_from_embedding_similarity(&[1.0, 0.0], &[0.5, 0.5]);
        assert_score_close(&embedding, std::f64::consts::FRAC_1_SQRT_2);
        assert!(
            embedding
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("embedding")
        );

        let judge = answer_relevancy_from_judge_output(
            r#"{"score":0.82,"reason":"answer directly addresses the question"}"#,
        )
        .expect("judge path");
        assert_score_close(&judge, 0.82);
        assert_eq!(
            judge.reason.as_deref(),
            Some("answer directly addresses the question")
        );
    }

    #[test]
    fn test_10_3_2_answer_correctness_combines_semantic_and_factual_signals() {
        // SCEN-10.3.2 / AC2 / TEST-10.3.2
        let result = answer_correctness(
            0.8,
            FactualCorrectnessCounts::new(2, 1, 1),
            AnswerCorrectnessWeights::new(0.25, 0.75),
        );

        assert_score_close(&result, 0.7);
        assert_eq!(result.metric_name, "answer_correctness");
        assert!(
            result
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("semantic=0.800")
        );
        assert!(
            result
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("factual=0.667")
        );
        assert_eq!(result.evidence.len(), 2);
    }

    #[test]
    fn test_10_3_3_noise_sensitivity_returns_interpretable_numeric_score() {
        // SCEN-10.3.3 / AC3 / TEST-10.3.3
        let result = noise_sensitivity(0.8, 0.5);

        assert_score_close(&result, 0.3);
        assert_eq!(result.metric_name, "noise_sensitivity");
        assert!(
            result
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("clean=0.800")
        );
        assert!(
            result
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("noisy=0.500")
        );
    }
}
