use std::collections::BTreeMap;

use crate::{
    DetailedMetricResult, EmbeddingProvider, EmbeddingRequest, MetricEvidence, MetricValueType,
    RagasError, ScoreNormalizationPolicy, cosine_similarity,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SemanticThresholdPolicy {
    pub threshold: f64,
    pub inclusive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotedSpan {
    pub text: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub char_start: usize,
    pub char_end: usize,
}

impl QuotedSpan {
    pub fn new(
        text: impl Into<String>,
        byte_start: usize,
        byte_end: usize,
        char_start: usize,
        char_end: usize,
    ) -> Self {
        Self {
            text: text.into(),
            byte_start,
            byte_end,
            char_start,
            char_end,
        }
    }
}

impl SemanticThresholdPolicy {
    pub fn inclusive(threshold: f64) -> Self {
        Self {
            threshold,
            inclusive: true,
        }
    }

    pub fn exclusive(threshold: f64) -> Self {
        Self {
            threshold,
            inclusive: false,
        }
    }
}

pub fn lexical_tokenizer_assumptions() -> Vec<&'static str> {
    vec!["whitespace-lowercase", "character-unigram"]
}

pub fn exact_match(candidate: &str, reference: &str) -> DetailedMetricResult {
    let candidate = candidate.trim();
    let reference = reference.trim();
    if candidate.is_empty() && reference.is_empty() {
        return numeric_result(
            "exact_match",
            1.0,
            "both strings empty; provider-free exact match treats them as equal",
            Vec::new(),
        );
    }

    numeric_result(
        "exact_match",
        if candidate == reference { 1.0 } else { 0.0 },
        "provider-free deterministic exact string match after trim",
        Vec::new(),
    )
}

pub fn string_distance_similarity(candidate: &str, reference: &str) -> DetailedMetricResult {
    let candidate_len = candidate.chars().count();
    let reference_len = reference.chars().count();
    if candidate_len == 0 && reference_len == 0 {
        return numeric_result(
            "string_distance_similarity",
            1.0,
            "both strings empty; normalized edit similarity is 1",
            Vec::new(),
        );
    }

    let max_len = candidate_len.max(reference_len);
    let distance = levenshtein_distance(candidate, reference);
    let score = 1.0 - distance as f64 / max_len as f64;
    numeric_result(
        "string_distance_similarity",
        score,
        "provider-free normalized Levenshtein similarity",
        vec![MetricEvidence::new(
            "levenshtein",
            format!("distance={distance} max_len={max_len}"),
        )],
    )
}

pub fn bleu_unigram(candidate: &str, reference: &str) -> DetailedMetricResult {
    let candidate_tokens = whitespace_lowercase_tokens(candidate);
    let reference_tokens = whitespace_lowercase_tokens(reference);
    if candidate_tokens.is_empty() && reference_tokens.is_empty() {
        return numeric_result(
            "bleu_unigram",
            1.0,
            "both strings empty under whitespace-lowercase tokenizer",
            Vec::new(),
        );
    }
    if candidate_tokens.is_empty() {
        return numeric_result(
            "bleu_unigram",
            0.0,
            "empty candidate under whitespace-lowercase tokenizer",
            Vec::new(),
        );
    }

    let reference_counts = token_counts(&reference_tokens);
    let mut candidate_counts = BTreeMap::new();
    for token in &candidate_tokens {
        *candidate_counts.entry(token.clone()).or_insert(0usize) += 1;
    }
    let overlap = candidate_counts
        .iter()
        .map(|(token, count)| count.min(reference_counts.get(token).unwrap_or(&0)))
        .sum::<usize>();

    numeric_result(
        "bleu_unigram",
        overlap as f64 / candidate_tokens.len() as f64,
        "BLEU-1 clipped precision with whitespace-lowercase tokenizer",
        Vec::new(),
    )
}

pub fn rouge_l_recall(candidate: &str, reference: &str) -> DetailedMetricResult {
    let candidate_tokens = whitespace_lowercase_tokens(candidate);
    let reference_tokens = whitespace_lowercase_tokens(reference);
    if candidate_tokens.is_empty() && reference_tokens.is_empty() {
        return numeric_result(
            "rouge_l_recall",
            1.0,
            "both strings empty under whitespace-lowercase tokenizer",
            Vec::new(),
        );
    }
    if reference_tokens.is_empty() {
        return numeric_result(
            "rouge_l_recall",
            0.0,
            "empty reference under whitespace-lowercase tokenizer",
            Vec::new(),
        );
    }

    let lcs = longest_common_subsequence_len(&candidate_tokens, &reference_tokens);
    numeric_result(
        "rouge_l_recall",
        lcs as f64 / reference_tokens.len() as f64,
        "ROUGE-L recall with whitespace-lowercase tokenizer",
        vec![MetricEvidence::new("lcs", format!("lcs={lcs}"))],
    )
}

pub fn chrf_score(candidate: &str, reference: &str) -> DetailedMetricResult {
    let candidate_chars = character_unigrams(candidate);
    let reference_chars = character_unigrams(reference);
    if candidate_chars.is_empty() && reference_chars.is_empty() {
        return numeric_result(
            "chrf",
            1.0,
            "both strings empty under character-unigram tokenizer",
            Vec::new(),
        );
    }
    if candidate_chars.is_empty() || reference_chars.is_empty() {
        return numeric_result(
            "chrf",
            0.0,
            "empty candidate or reference under character-unigram tokenizer",
            Vec::new(),
        );
    }

    let overlap = multiset_overlap(&candidate_chars, &reference_chars);
    let precision = overlap as f64 / candidate_chars.len() as f64;
    let recall = overlap as f64 / reference_chars.len() as f64;
    let score = if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    };
    numeric_result(
        "chrf",
        score,
        "CHRF character F1 with character-unigram tokenizer",
        Vec::new(),
    )
}

pub async fn semantic_similarity_batch<P>(
    provider: &P,
    pairs: &[(String, String)],
) -> Result<Vec<DetailedMetricResult>, RagasError>
where
    P: EmbeddingProvider + ?Sized,
{
    if pairs.is_empty() {
        return Ok(Vec::new());
    }

    let mut input = Vec::with_capacity(pairs.len() * 2);
    for (left, right) in pairs {
        input.push(left.clone());
        input.push(right.clone());
    }

    let response = provider.embed(EmbeddingRequest { input }).await?;
    if response.embeddings.len() != pairs.len() * 2 {
        return Err(RagasError::Parse {
            message: format!(
                "semantic similarity embedding count mismatch: expected {}, got {}",
                pairs.len() * 2,
                response.embeddings.len()
            ),
        });
    }

    Ok(response
        .embeddings
        .chunks_exact(2)
        .enumerate()
        .map(|(pair_index, chunk)| {
            let mut result = semantic_similarity_from_vectors(&chunk[0], &chunk[1]);
            result = result.with_evidence(MetricEvidence::new(
                format!("pair[{pair_index}]"),
                "embedding cosine similarity",
            ));
            result
        })
        .collect())
}

pub fn threshold_semantic_similarity(
    score: f64,
    policy: SemanticThresholdPolicy,
) -> DetailedMetricResult {
    let normalized = score.clamp(0.0, 1.0);
    let passed = if policy.inclusive {
        normalized >= policy.threshold
    } else {
        normalized > policy.threshold
    };
    let mode = if policy.inclusive {
        "inclusive"
    } else {
        "exclusive"
    };
    numeric_result(
        "threshold_semantic_similarity",
        if passed { 1.0 } else { 0.0 },
        format!(
            "{mode} threshold policy: score={normalized:.3} threshold={:.3}",
            policy.threshold
        ),
        Vec::new(),
    )
}

pub fn semantic_similarity_from_vectors(left: &[f32], right: &[f32]) -> DetailedMetricResult {
    let has_zero = vector_is_zero(left) || vector_is_zero(right);
    let score = cosine_similarity(left, right).clamp(0.0, 1.0);
    let reason = if has_zero {
        "embedding cosine similarity is stable for zero vector inputs".to_string()
    } else {
        "embedding cosine similarity".to_string()
    };
    numeric_result("semantic_similarity", score, reason, Vec::new())
}

pub fn extract_quoted_spans(text: &str) -> Vec<QuotedSpan> {
    let mut spans = Vec::new();
    let mut open: Option<(usize, usize)> = None;

    for (char_index, (byte_index, character)) in text.char_indices().enumerate() {
        if character != '"' {
            continue;
        }

        if let Some((byte_start, char_start)) = open.take() {
            let byte_end = byte_index;
            let char_end = char_index;
            spans.push(QuotedSpan::new(
                text[byte_start..byte_end].to_string(),
                byte_start,
                byte_end,
                char_start,
                char_end,
            ));
        } else {
            open = Some((byte_index + character.len_utf8(), char_index + 1));
        }
    }

    spans
}

pub fn quoted_span_overlap(candidate: &QuotedSpan, reference: &QuotedSpan) -> DetailedMetricResult {
    let intersection_start = candidate.char_start.max(reference.char_start);
    let intersection_end = candidate.char_end.min(reference.char_end);
    let intersection = intersection_end.saturating_sub(intersection_start);
    let union_start = candidate.char_start.min(reference.char_start);
    let union_end = candidate.char_end.max(reference.char_end);
    let union = union_end.saturating_sub(union_start);
    let score = if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    };
    let reason = if intersection == 0 {
        "no quoted span overlap".to_string()
    } else if score < 1.0 {
        format!("partial overlap: intersection={intersection} union={union}")
    } else {
        format!("complete overlap: intersection={intersection} union={union}")
    };
    numeric_result(
        "quoted_span_overlap",
        score,
        reason,
        vec![MetricEvidence::new(
            "char_range",
            format!(
                "candidate={}..{} reference={}..{}",
                candidate.char_start, candidate.char_end, reference.char_start, reference.char_end
            ),
        )],
    )
}

pub fn quoted_citation_coverage(answer: &str, sources: &[String]) -> DetailedMetricResult {
    let spans = extract_quoted_spans(answer);
    if spans.is_empty() {
        return numeric_result(
            "quoted_citation_coverage",
            0.0,
            "missing citations: answer contains no quoted spans",
            Vec::new(),
        );
    }
    if sources.is_empty() {
        return numeric_result(
            "quoted_citation_coverage",
            0.0,
            "missing citation sources",
            Vec::new(),
        );
    }

    let mut matched = 0usize;
    let mut evidence = Vec::new();
    for span in &spans {
        if let Some((source_index, _)) = sources
            .iter()
            .enumerate()
            .find(|(_, source)| source.contains(&span.text))
        {
            matched += 1;
            evidence.push(MetricEvidence::new(
                format!("source[{source_index}]"),
                span.text.clone(),
            ));
        }
    }

    numeric_result(
        "quoted_citation_coverage",
        matched as f64 / spans.len() as f64,
        format!(
            "quoted citation coverage: matched={matched} total={}",
            spans.len()
        ),
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
        .expect("traditional metric score is normalized")
        .with_reason(reason);
    for item in evidence {
        result = result.with_evidence(item);
    }
    result
}

fn whitespace_lowercase_tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|token| {
            token
                .trim_matches(|character: char| !character.is_ascii_alphanumeric())
                .to_ascii_lowercase()
        })
        .filter(|token| !token.is_empty())
        .collect()
}

fn character_unigrams(text: &str) -> Vec<char> {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

fn vector_is_zero(vector: &[f32]) -> bool {
    vector.is_empty() || vector.iter().all(|value| *value == 0.0)
}

fn token_counts(tokens: &[String]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for token in tokens {
        *counts.entry(token.clone()).or_insert(0usize) += 1;
    }
    counts
}

fn multiset_overlap<T>(left: &[T], right: &[T]) -> usize
where
    T: Ord + Clone,
{
    let mut left_counts = BTreeMap::new();
    let mut right_counts = BTreeMap::new();
    for item in left {
        *left_counts.entry(item.clone()).or_insert(0usize) += 1;
    }
    for item in right {
        *right_counts.entry(item.clone()).or_insert(0usize) += 1;
    }
    left_counts
        .iter()
        .map(|(item, count)| count.min(right_counts.get(item).unwrap_or(&0)))
        .sum()
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    let mut current = vec![0usize; right.len() + 1];

    for (left_index, left_char) in left.iter().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_char) in right.iter().enumerate() {
            let insertion = current[right_index] + 1;
            let deletion = previous[right_index + 1] + 1;
            let substitution = previous[right_index] + usize::from(left_char != right_char);
            current[right_index + 1] = insertion.min(deletion).min(substitution);
        }
        previous.clone_from(&current);
    }

    previous[right.len()]
}

fn longest_common_subsequence_len(left: &[String], right: &[String]) -> usize {
    let mut table = vec![vec![0usize; right.len() + 1]; left.len() + 1];
    for left_index in 0..left.len() {
        for right_index in 0..right.len() {
            table[left_index + 1][right_index + 1] = if left[left_index] == right[right_index] {
                table[left_index][right_index] + 1
            } else {
                table[left_index][right_index + 1].max(table[left_index + 1][right_index])
            };
        }
    }
    table[left.len()][right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use crate::{EmbeddingRequest, EmbeddingResponse};
    use async_trait::async_trait;

    fn assert_score_close(result: &DetailedMetricResult, expected: f64) {
        let actual = result.score.expect("score");
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn test_11_1_1_exact_string_metrics_are_deterministic_and_provider_free() {
        // SCEN-11.1.1 / AC1 / TEST-11.1.1
        let exact = exact_match("Ragas evaluates apps", "Ragas evaluates apps");
        assert_score_close(&exact, 1.0);
        assert!(
            exact
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("provider-free")
        );

        let distance = string_distance_similarity("kitten", "sitting");
        assert_score_close(&distance, 4.0 / 7.0);
        assert_eq!(distance.metric_name, "string_distance_similarity");
    }

    #[test]
    fn test_11_1_2_bleu_rouge_chrf_expose_tokenizer_assumptions() {
        // SCEN-11.1.2 / AC2 / TEST-11.1.2
        let assumptions = lexical_tokenizer_assumptions();
        assert!(assumptions.contains(&"whitespace-lowercase"));
        assert!(assumptions.contains(&"character-unigram"));

        let bleu = bleu_unigram("the cat sat", "the cat slept");
        assert_score_close(&bleu, 2.0 / 3.0);
        assert!(
            bleu.reason
                .as_deref()
                .unwrap_or("")
                .contains("whitespace-lowercase")
        );

        let rouge = rouge_l_recall("the cat sat", "the cat slept");
        assert_score_close(&rouge, 2.0 / 3.0);

        let chrf = chrf_score("abc", "abd");
        assert_score_close(&chrf, 2.0 / 3.0);
        assert!(
            chrf.reason
                .as_deref()
                .unwrap_or("")
                .contains("character-unigram")
        );
    }

    #[test]
    fn test_11_1_3_traditional_metrics_handle_empty_strings_explicitly() {
        // SCEN-11.1.3 / AC3 / TEST-11.1.3
        let exact = exact_match("", "");
        assert_score_close(&exact, 1.0);
        assert!(
            exact
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("both strings empty")
        );

        let bleu = bleu_unigram("", "reference text");
        assert_score_close(&bleu, 0.0);
        assert!(
            bleu.reason
                .as_deref()
                .unwrap_or("")
                .contains("empty candidate")
        );

        let chrf = chrf_score("", "");
        assert_score_close(&chrf, 1.0);
        assert!(
            chrf.reason
                .as_deref()
                .unwrap_or("")
                .contains("both strings empty")
        );
    }

    struct RecordingEmbeddingProvider {
        calls: Arc<Mutex<Vec<Vec<String>>>>,
        vectors: HashMap<String, Vec<f32>>,
    }

    #[async_trait]
    impl EmbeddingProvider for RecordingEmbeddingProvider {
        async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
            self.calls
                .lock()
                .expect("calls")
                .push(request.input.clone());
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

    #[tokio::test]
    async fn test_11_2_1_semantic_similarity_uses_embedding_provider_with_batching() {
        // SCEN-11.2.1 / AC1 / TEST-11.2.1
        let calls = Arc::new(Mutex::new(Vec::new()));
        let provider = RecordingEmbeddingProvider {
            calls: calls.clone(),
            vectors: HashMap::from([
                ("q1".to_string(), vec![1.0, 0.0]),
                ("a1".to_string(), vec![0.5, 0.5]),
                ("q2".to_string(), vec![0.0, 1.0]),
                ("a2".to_string(), vec![0.0, 1.0]),
            ]),
        };
        let pairs = vec![
            ("q1".to_string(), "a1".to_string()),
            ("q2".to_string(), "a2".to_string()),
        ];

        let results = semantic_similarity_batch(&provider, &pairs)
            .await
            .expect("semantic batch");

        assert_eq!(
            *calls.lock().expect("calls"),
            vec![vec![
                "q1".to_string(),
                "a1".to_string(),
                "q2".to_string(),
                "a2".to_string()
            ]]
        );
        assert_score_close(&results[0], 0.70710678);
        assert_score_close(&results[1], 1.0);
    }

    #[test]
    fn test_11_2_2_threshold_policy_is_configurable() {
        // SCEN-11.2.2 / AC2 / TEST-11.2.2
        let inclusive = threshold_semantic_similarity(0.8, SemanticThresholdPolicy::inclusive(0.8));
        assert_score_close(&inclusive, 1.0);
        assert!(
            inclusive
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("inclusive")
        );

        let exclusive = threshold_semantic_similarity(0.8, SemanticThresholdPolicy::exclusive(0.8));
        assert_score_close(&exclusive, 0.0);
        assert!(
            exclusive
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("exclusive")
        );
    }

    #[test]
    fn test_11_2_3_scores_are_stable_for_zero_vectors() {
        // SCEN-11.2.3 / AC3 / TEST-11.2.3
        let result = semantic_similarity_from_vectors(&[0.0, 0.0], &[1.0, 0.0]);

        assert_score_close(&result, 0.0);
        assert!(result.score.expect("score").is_finite());
        assert!(
            result
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("zero vector")
        );
    }

    #[test]
    fn test_11_3_1_quoted_span_extraction_preserves_byte_and_char_ranges() {
        // SCEN-11.3.1 / AC1 / TEST-11.3.1
        let spans = extract_quoted_spans("A \"猫\" and \"dog\"");

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0], QuotedSpan::new("猫", 3, 6, 3, 4));
        assert_eq!(spans[1].text, "dog");
    }

    #[test]
    fn test_11_3_2_overlap_scoring_handles_partial_matches() {
        // SCEN-11.3.2 / AC2 / TEST-11.3.2
        let candidate = QuotedSpan::new("candidate", 10, 20, 10, 20);
        let reference = QuotedSpan::new("reference", 15, 25, 15, 25);

        let result = quoted_span_overlap(&candidate, &reference);

        assert_score_close(&result, 1.0 / 3.0);
        assert!(
            result
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("partial overlap")
        );
    }

    #[test]
    fn test_11_3_3_missing_citations_produce_explicit_zero_score_reason() {
        // SCEN-11.3.3 / AC3 / TEST-11.3.3
        let result = quoted_citation_coverage(
            "The answer summarizes retrieval but has no citation.",
            &["Ragas evaluates LLM applications.".to_string()],
        );

        assert_score_close(&result, 0.0);
        assert!(
            result
                .reason
                .as_deref()
                .unwrap_or("")
                .contains("missing citations")
        );
    }
}
