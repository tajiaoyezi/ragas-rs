use std::collections::BTreeMap;

use async_trait::async_trait;

use crate::validation::{MetricRequirements, SampleField};
use crate::{
    DetailedMetricResult, EmbeddingProvider, EmbeddingRequest, Metric, MetricEvidence,
    MetricResult, MetricValue, MetricValueType, RagasError, ScoreNormalizationPolicy,
    SingleTurnSample, cosine_similarity,
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

/// Which character-distance backend [`string_distance_similarity_with`] uses, mirroring the
/// `DistanceMeasure` enum of Python ragas's `NonLLMStringSimilarity`. All four are deterministic,
/// case-sensitive (no folding), and symmetric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DistanceMeasure {
    #[default]
    Levenshtein,
    Hamming,
    Jaro,
    JaroWinkler,
}

impl DistanceMeasure {
    fn as_str(self) -> &'static str {
        match self {
            Self::Levenshtein => "levenshtein",
            Self::Hamming => "hamming",
            Self::Jaro => "jaro",
            Self::JaroWinkler => "jaro_winkler",
        }
    }
}

/// Normalized string similarity in [0, 1] between `candidate` and `reference` under `measure`,
/// matching `1 - rapidfuzz.distance.<Measure>.normalized_distance(...)` as used by Python ragas's
/// `NonLLMStringSimilarity`. Two empty strings score 1.0 for every measure. Case-sensitive.
pub fn string_distance_similarity_with(
    candidate: &str,
    reference: &str,
    measure: DistanceMeasure,
) -> DetailedMetricResult {
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    let reference_chars = reference.chars().collect::<Vec<_>>();
    if candidate_chars.is_empty() && reference_chars.is_empty() {
        return numeric_result(
            "string_distance_similarity",
            1.0,
            "both strings empty; normalized similarity is 1",
            Vec::new(),
        );
    }

    let max_len = candidate_chars.len().max(reference_chars.len());
    let (score, detail) = match measure {
        DistanceMeasure::Levenshtein => {
            let distance = levenshtein_distance(candidate, reference);
            (
                1.0 - distance as f64 / max_len as f64,
                format!("distance={distance} max_len={max_len}"),
            )
        }
        DistanceMeasure::Hamming => {
            let distance = hamming_distance(&candidate_chars, &reference_chars);
            (
                1.0 - distance as f64 / max_len as f64,
                format!("distance={distance} max_len={max_len}"),
            )
        }
        DistanceMeasure::Jaro => {
            let similarity = jaro_similarity(&candidate_chars, &reference_chars);
            (similarity, format!("jaro={similarity:.6}"))
        }
        DistanceMeasure::JaroWinkler => {
            let similarity = jaro_winkler_similarity(&candidate_chars, &reference_chars);
            (similarity, format!("jaro_winkler={similarity:.6}"))
        }
    };
    numeric_result(
        "string_distance_similarity",
        score,
        format!("provider-free normalized {} similarity", measure.as_str()),
        vec![MetricEvidence::new(measure.as_str(), detail)],
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

/// Which ROUGE variant to compute: unigram overlap (`Rouge1`), bigram overlap
/// (`Rouge2`), or longest-common-subsequence (`RougeL`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RougeType {
    Rouge1,
    Rouge2,
    #[default]
    RougeL,
}

impl RougeType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Rouge1 => "rouge1",
            Self::Rouge2 => "rouge2",
            Self::RougeL => "rougeL",
        }
    }
}

/// Which component of the precision/recall/F-measure triple a [`RougeScore`] reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RougeMode {
    Precision,
    Recall,
    #[default]
    FMeasure,
}

impl RougeMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Precision => "precision",
            Self::Recall => "recall",
            Self::FMeasure => "fmeasure",
        }
    }
}

/// Precision/recall/F-measure produced by one ROUGE computation.
///
/// `precision = overlap / candidate_ngram_count`,
/// `recall    = overlap / reference_ngram_count`,
/// `fmeasure  = 2*P*R/(P+R)` (defined as `0.0` when `P+R == 0`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RougeScores {
    pub precision: f64,
    pub recall: f64,
    pub fmeasure: f64,
}

impl RougeScores {
    fn from_overlap(overlap: usize, candidate_count: usize, reference_count: usize) -> Self {
        let precision = ratio(overlap, candidate_count);
        let recall = ratio(overlap, reference_count);
        let fmeasure = if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        };
        Self {
            precision,
            recall,
            fmeasure,
        }
    }

    fn select(&self, mode: RougeMode) -> f64 {
        match mode {
            RougeMode::Precision => self.precision,
            RougeMode::Recall => self.recall,
            RougeMode::FMeasure => self.fmeasure,
        }
    }
}

/// Faithful, deterministic (provider-free) port of ragas' RougeScore metric.
///
/// Tokenization: text is lowercased and split on whitespace, then each token is
/// stripped of leading/trailing non-alphanumeric characters (the crate-wide
/// `whitespace-lowercase` tokenizer). This is a deliberate, documented choice —
/// it is simpler than the reference `rouge_score` PTB tokenizer but agrees with
/// it on whitespace-separated alphanumeric text.
///
/// For `Rouge1`/`Rouge2` the overlap is the clipped multiset intersection of the
/// candidate and reference n-grams; for `RougeL` it is the length of the longest
/// common subsequence of the token streams. Empty candidate or reference are
/// handled without dividing by zero (the corresponding count is 0, so the ratio
/// is 0.0 and the F-measure collapses to 0.0).
#[derive(Debug, Clone, Copy, Default)]
pub struct RougeScore {
    pub rouge_type: RougeType,
    pub mode: RougeMode,
}

impl RougeScore {
    pub fn new(rouge_type: RougeType, mode: RougeMode) -> Self {
        Self { rouge_type, mode }
    }

    pub fn with_rouge_type(mut self, rouge_type: RougeType) -> Self {
        self.rouge_type = rouge_type;
        self
    }

    pub fn with_mode(mut self, mode: RougeMode) -> Self {
        self.mode = mode;
        self
    }

    /// Compute the full precision/recall/F-measure triple for `candidate` scored
    /// against `reference` under this metric's `rouge_type`.
    pub fn scores(&self, candidate: &str, reference: &str) -> RougeScores {
        let candidate_tokens = whitespace_lowercase_tokens(candidate);
        let reference_tokens = whitespace_lowercase_tokens(reference);
        match self.rouge_type {
            RougeType::Rouge1 => self.ngram_scores(&candidate_tokens, &reference_tokens, 1),
            RougeType::Rouge2 => self.ngram_scores(&candidate_tokens, &reference_tokens, 2),
            RougeType::RougeL => {
                let overlap = longest_common_subsequence_len(&candidate_tokens, &reference_tokens);
                RougeScores::from_overlap(overlap, candidate_tokens.len(), reference_tokens.len())
            }
        }
    }

    fn ngram_scores(&self, candidate: &[String], reference: &[String], n: usize) -> RougeScores {
        let candidate_ngrams = ngrams(candidate, n);
        let reference_ngrams = ngrams(reference, n);
        let overlap = multiset_overlap(&candidate_ngrams, &reference_ngrams);
        RougeScores::from_overlap(overlap, candidate_ngrams.len(), reference_ngrams.len())
    }

    /// Score `candidate` against `reference`, returning the single component
    /// selected by `mode` wrapped in a [`DetailedMetricResult`].
    pub fn detailed(&self, candidate: &str, reference: &str) -> DetailedMetricResult {
        let scores = self.scores(candidate, reference);
        let score = scores.select(self.mode);
        numeric_result(
            "rouge_score",
            score,
            format!(
                "{} {} with whitespace-lowercase tokenizer (P={:.4} R={:.4} F={:.4})",
                self.rouge_type.as_str(),
                self.mode.as_str(),
                scores.precision,
                scores.recall,
                scores.fmeasure
            ),
            Vec::new(),
        )
    }
}

#[async_trait]
impl Metric for RougeScore {
    fn name(&self) -> &str {
        "rouge_score"
    }

    fn requirements(&self) -> MetricRequirements {
        MetricRequirements::new(
            self.name(),
            vec![SampleField::Response, SampleField::Reference],
        )
    }

    /// Score the sample's `response` (candidate) against its `reference`. The
    /// reference is required; without it ROUGE is undefined.
    async fn score(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError> {
        let reference = match sample.reference.as_deref() {
            Some(reference) => reference,
            None => {
                return Err(RagasError::Parse {
                    message: "rouge score requires a reference".to_string(),
                });
            }
        };

        let scores = self.scores(&sample.response, reference);
        let score = scores.select(self.mode);
        Ok(
            MetricResult::success(self.name(), MetricValue::numeric(score)).with_reason(format!(
                "{} {} (P={:.4} R={:.4} F={:.4})",
                self.rouge_type.as_str(),
                self.mode.as_str(),
                scores.precision,
                scores.recall,
                scores.fmeasure
            )),
        )
    }
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

/// Build the contiguous character `n`-grams of `chars` as joined strings (multiset elements).
fn char_ngrams(chars: &[char], n: usize) -> Vec<String> {
    if n == 0 || chars.len() < n {
        return Vec::new();
    }
    chars
        .windows(n)
        .map(|window| window.iter().collect::<String>())
        .collect()
}

/// BLEU-4 (functional, not sacrebleu byte-parity): the geometric mean of modified n-gram
/// precisions for n = 1..=4 with a brevity penalty, over the whitespace+lowercase tokenizer.
/// With no smoothing a zero match at any order yields 0.0 (plain sentence BLEU).
pub fn bleu_score(candidate: &str, reference: &str) -> DetailedMetricResult {
    let candidate_tokens = whitespace_lowercase_tokens(candidate);
    let reference_tokens = whitespace_lowercase_tokens(reference);

    if candidate_tokens.is_empty() && reference_tokens.is_empty() {
        return numeric_result(
            "bleu",
            1.0,
            "both strings empty under whitespace-lowercase tokenizer",
            Vec::new(),
        );
    }
    if candidate_tokens.is_empty() || reference_tokens.is_empty() {
        return numeric_result("bleu", 0.0, "one side has no tokens", Vec::new());
    }

    const MAX_N: usize = 4;
    let mut log_precision_sum = 0.0f64;
    for n in 1..=MAX_N {
        let candidate_ngrams = ngrams(&candidate_tokens, n);
        if candidate_ngrams.is_empty() {
            return numeric_result(
                "bleu",
                0.0,
                format!("candidate shorter than the {n}-gram order"),
                Vec::new(),
            );
        }
        let clipped = multiset_overlap(&candidate_ngrams, &ngrams(&reference_tokens, n));
        if clipped == 0 {
            return numeric_result("bleu", 0.0, format!("no overlapping {n}-grams"), Vec::new());
        }
        log_precision_sum += (clipped as f64 / candidate_ngrams.len() as f64).ln();
    }
    let geometric_mean = (log_precision_sum / MAX_N as f64).exp();

    let candidate_len = candidate_tokens.len() as f64;
    let reference_len = reference_tokens.len() as f64;
    let brevity_penalty = if candidate_len > reference_len {
        1.0
    } else {
        (1.0 - reference_len / candidate_len).exp()
    };

    numeric_result(
        "bleu",
        (brevity_penalty * geometric_mean).clamp(0.0, 1.0),
        format!("BLEU-4 (bp={brevity_penalty:.4}, geo_mean={geometric_mean:.4})"),
        Vec::new(),
    )
}

/// chrF (functional): the character n-gram F-beta score (beta = 2) averaged over n = 1..=6 —
/// the standard chrF configuration. Whitespace is ignored and characters are lowercased.
pub fn chrf(candidate: &str, reference: &str) -> DetailedMetricResult {
    let candidate_chars = character_unigrams(candidate);
    let reference_chars = character_unigrams(reference);

    if candidate_chars.is_empty() && reference_chars.is_empty() {
        return numeric_result(
            "chrf",
            1.0,
            "both strings empty under the character tokenizer",
            Vec::new(),
        );
    }
    if candidate_chars.is_empty() || reference_chars.is_empty() {
        return numeric_result("chrf", 0.0, "one side has no characters", Vec::new());
    }

    const MAX_N: usize = 6;
    const BETA_SQUARED: f64 = 4.0; // beta = 2
    let mut precision_sum = 0.0f64;
    let mut recall_sum = 0.0f64;
    let mut orders = 0usize;
    for n in 1..=MAX_N {
        let candidate_ngrams = char_ngrams(&candidate_chars, n);
        let reference_ngrams = char_ngrams(&reference_chars, n);
        if candidate_ngrams.is_empty() || reference_ngrams.is_empty() {
            continue;
        }
        let overlap = multiset_overlap(&candidate_ngrams, &reference_ngrams) as f64;
        precision_sum += overlap / candidate_ngrams.len() as f64;
        recall_sum += overlap / reference_ngrams.len() as f64;
        orders += 1;
    }
    if orders == 0 {
        return numeric_result(
            "chrf",
            0.0,
            "no comparable character n-gram orders",
            Vec::new(),
        );
    }
    let precision = precision_sum / orders as f64;
    let recall = recall_sum / orders as f64;
    let denominator = BETA_SQUARED * precision + recall;
    let score = if denominator == 0.0 {
        0.0
    } else {
        (1.0 + BETA_SQUARED) * precision * recall / denominator
    };

    numeric_result(
        "chrf",
        score.clamp(0.0, 1.0),
        format!("chrF (beta=2, n<=6, precision={precision:.4}, recall={recall:.4})"),
        Vec::new(),
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

/// Hamming distance with rapidfuzz `pad=True` semantics: positional mismatches over the shared
/// prefix plus the absolute length difference (the longer string's tail counts as mismatches).
fn hamming_distance(left: &[char], right: &[char]) -> usize {
    let overlap = left.len().min(right.len());
    let mismatches = (0..overlap)
        .filter(|&index| left[index] != right[index])
        .count();
    mismatches + left.len().abs_diff(right.len())
}

/// Jaro similarity in [0, 1]. Identical strings (including two empty ones) score 1.0; if exactly
/// one side is empty there are no matches, so the score is 0.0.
fn jaro_similarity(left: &[char], right: &[char]) -> f64 {
    if left == right {
        return 1.0;
    }
    let (left_len, right_len) = (left.len(), right.len());
    if left_len == 0 || right_len == 0 {
        return 0.0;
    }

    // Match window = floor(max_len / 2) - 1, clamped at 0 for very short inputs.
    let window = (left_len.max(right_len) / 2).saturating_sub(1);
    let mut left_matched = vec![false; left_len];
    let mut right_matched = vec![false; right_len];
    let mut matches = 0usize;
    for (index, left_char) in left.iter().enumerate() {
        let lo = index.saturating_sub(window);
        let hi = (index + window + 1).min(right_len);
        for offset in lo..hi {
            if !right_matched[offset] && *left_char == right[offset] {
                left_matched[index] = true;
                right_matched[offset] = true;
                matches += 1;
                break;
            }
        }
    }
    if matches == 0 {
        return 0.0;
    }

    // Transpositions = half the number of matched chars that line up out of order.
    let left_chars = left
        .iter()
        .zip(left_matched.iter())
        .filter_map(|(character, matched)| matched.then_some(*character));
    let right_chars = right
        .iter()
        .zip(right_matched.iter())
        .filter_map(|(character, matched)| matched.then_some(*character));
    let out_of_order = left_chars
        .zip(right_chars)
        .filter(|(left_char, right_char)| left_char != right_char)
        .count();
    let transpositions = out_of_order / 2;

    let matches = matches as f64;
    ((matches / left_len as f64)
        + (matches / right_len as f64)
        + ((matches - transpositions as f64) / matches))
        / 3.0
}

/// Jaro-Winkler similarity: Jaro boosted by the common prefix (capped at 4) times the standard
/// scaling factor p = 0.1 (rapidfuzz default). Matching rapidfuzz/Winkler, the prefix bonus is
/// applied only when the underlying Jaro similarity exceeds the 0.7 boost threshold; at or below
/// it the unboosted Jaro value is returned.
fn jaro_winkler_similarity(left: &[char], right: &[char]) -> f64 {
    let jaro = jaro_similarity(left, right);
    if jaro <= 0.7 {
        return jaro;
    }
    let prefix = left
        .iter()
        .zip(right.iter())
        .take(4)
        .take_while(|(left_char, right_char)| left_char == right_char)
        .count();
    jaro + prefix as f64 * 0.1 * (1.0 - jaro)
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

/// Build the list of contiguous `n`-grams over `tokens`, each joined by a space so
/// it can be counted as a single multiset element. Returns empty when there are
/// fewer than `n` tokens.
fn ngrams(tokens: &[String], n: usize) -> Vec<String> {
    if n == 0 || tokens.len() < n {
        return Vec::new();
    }
    tokens.windows(n).map(|window| window.join(" ")).collect()
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
    fn distance_measures_match_rapidfuzz_reference_oracle() {
        // Oracle verified two ways: empirically against rapidfuzz 3.14.5 and by independent
        // hand-derivation; both sources agreed on every value (canonical Jaro/Jaro-Winkler pairs
        // MARTHA/MARHTA, DWAYNE/DUANE, DIXON/DICKSONX). Hamming uses pad=True (denominator
        // max(len), not len1+len2). All measures are case-sensitive and symmetric.
        use DistanceMeasure::{Hamming, Jaro, JaroWinkler, Levenshtein};
        let cases: &[(&str, &str, DistanceMeasure, f64)] = &[
            ("MARTHA", "MARHTA", Jaro, 17.0 / 18.0),
            ("MARTHA", "MARHTA", JaroWinkler, 173.0 / 180.0),
            ("DWAYNE", "DUANE", Jaro, 37.0 / 45.0),
            ("DWAYNE", "DUANE", JaroWinkler, 21.0 / 25.0),
            ("DIXON", "DICKSONX", Jaro, 23.0 / 30.0),
            ("DIXON", "DICKSONX", JaroWinkler, 61.0 / 75.0),
            // Jaro-Winkler boost threshold (rapidfuzz: bonus only when Jaro > 0.7). Both pairs
            // share a prefix but have Jaro = 2/3 <= 0.7, so Jaro-Winkler must NOT boost.
            ("abcde12345", "abcde67890", Jaro, 2.0 / 3.0),
            ("abcde12345", "abcde67890", JaroWinkler, 2.0 / 3.0),
            ("abcd1234", "abcd5678", JaroWinkler, 2.0 / 3.0),
            ("karolin", "kathrin", Hamming, 4.0 / 7.0),
            ("abc", "abcd", Hamming, 3.0 / 4.0),
            ("color", "colour", Hamming, 2.0 / 3.0),
            ("kitten", "sitting", Levenshtein, 4.0 / 7.0),
            ("abc", "abc", Jaro, 1.0),
            ("abc", "abc", JaroWinkler, 1.0),
            ("abc", "abc", Hamming, 1.0),
            ("abc", "abc", Levenshtein, 1.0),
            ("", "", Levenshtein, 1.0),
            ("", "", Hamming, 1.0),
            ("", "", Jaro, 1.0),
            ("", "", JaroWinkler, 1.0),
            ("abc", "", Levenshtein, 0.0),
            ("abc", "", Hamming, 0.0),
            ("abc", "", Jaro, 0.0),
            ("abc", "", JaroWinkler, 0.0),
            // Case-sensitive: no folding (rapidfuzz compares raw code points).
            ("ABC", "abc", Levenshtein, 0.0),
            ("Abc", "abc", Levenshtein, 2.0 / 3.0),
            ("ABC", "abc", Jaro, 0.0),
        ];
        for (left, right, measure, expected) in cases {
            let score = string_distance_similarity_with(left, right, *measure)
                .score
                .expect("score");
            assert!(
                (score - expected).abs() < 1e-9,
                "{left:?} vs {right:?} [{measure:?}]: expected {expected}, got {score}"
            );
            // Every measure is symmetric: swapping the arguments must not change the score.
            let swapped = string_distance_similarity_with(right, left, *measure)
                .score
                .expect("score");
            assert!(
                (swapped - expected).abs() < 1e-9,
                "asymmetry for {left:?}/{right:?} [{measure:?}]: {swapped} != {expected}"
            );
        }
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
        assert_score_close(&results[0], std::f64::consts::FRAC_1_SQRT_2);
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

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn rouge_defaults_to_rougel_fmeasure() {
        let metric = RougeScore::default();
        assert_eq!(metric.rouge_type, RougeType::RougeL);
        assert_eq!(metric.mode, RougeMode::FMeasure);
    }

    #[test]
    fn rouge1_precision_recall_fmeasure_match_hand_computed_values() {
        // reference = "the cat sat on the mat" (6 unigrams),
        // candidate = "the cat sat"            (3 unigrams).
        // All 3 candidate unigrams appear in the reference -> overlap = 3.
        //   precision = 3/3 = 1.0
        //   recall    = 3/6 = 0.5
        //   f         = 2*1.0*0.5 / (1.0 + 0.5) = 1.0 / 1.5 = 2/3
        let scores = RougeScore::new(RougeType::Rouge1, RougeMode::FMeasure)
            .scores("the cat sat", "the cat sat on the mat");
        close(scores.precision, 1.0);
        close(scores.recall, 0.5);
        close(scores.fmeasure, 2.0 / 3.0);

        // The selected component follows `mode`.
        close(
            RougeScore::new(RougeType::Rouge1, RougeMode::Precision)
                .scores("the cat sat", "the cat sat on the mat")
                .precision,
            1.0,
        );
        close(
            RougeScore::new(RougeType::Rouge1, RougeMode::Recall)
                .scores("the cat sat", "the cat sat on the mat")
                .recall,
            0.5,
        );
    }

    #[test]
    fn rouge2_uses_bigram_overlap() {
        // candidate bigrams = {"the cat", "cat sat"} (2 bigrams),
        // reference bigrams = {"the cat","cat sat","sat on","on the","the mat"} (5 bigrams).
        // overlap = 2 ->
        //   precision = 2/2 = 1.0
        //   recall    = 2/5 = 0.4
        //   f         = 2*1.0*0.4 / 1.4 = 0.8/1.4 = 4/7
        let scores = RougeScore::new(RougeType::Rouge2, RougeMode::FMeasure)
            .scores("the cat sat", "the cat sat on the mat");
        close(scores.precision, 1.0);
        close(scores.recall, 0.4);
        close(scores.fmeasure, 4.0 / 7.0);
    }

    #[test]
    fn rougel_uses_longest_common_subsequence_not_contiguous_overlap() {
        // candidate = "a c", reference = "a b c d".
        // LCS(["a","c"], ["a","b","c","d"]) = 2 (non-contiguous: a then c).
        //   precision = 2/2 = 1.0
        //   recall    = 2/4 = 0.5
        //   f         = 2*1.0*0.5 / 1.5 = 2/3
        // A contiguous-overlap (bigram) metric would score 0 here, proving this is LCS-based.
        let lcs = RougeScore::new(RougeType::RougeL, RougeMode::FMeasure).scores("a c", "a b c d");
        close(lcs.precision, 1.0);
        close(lcs.recall, 0.5);
        close(lcs.fmeasure, 2.0 / 3.0);

        let bigram =
            RougeScore::new(RougeType::Rouge2, RougeMode::FMeasure).scores("a c", "a b c d");
        close(bigram.fmeasure, 0.0);
    }

    #[test]
    fn rouge_clips_repeated_ngrams_to_reference_multiset() {
        // candidate = "the the", reference = "the the the".
        // candidate has "the" x2, reference has "the" x3 -> clipped overlap = 2.
        //   precision = 2/2 = 1.0
        //   recall    = 2/3
        //   f         = 2*1.0*(2/3) / (1.0 + 2/3) = (4/3)/(5/3) = 4/5
        let scores = RougeScore::new(RougeType::Rouge1, RougeMode::FMeasure)
            .scores("the the", "the the the");
        close(scores.precision, 1.0);
        close(scores.recall, 2.0 / 3.0);
        close(scores.fmeasure, 0.8);
    }

    #[test]
    fn rouge_identical_strings_score_one_disjoint_score_zero() {
        for rouge_type in [RougeType::Rouge1, RougeType::Rouge2, RougeType::RougeL] {
            // Discrimination: a perfect candidate scores 1.0 on every component...
            let identical = RougeScore::new(rouge_type, RougeMode::FMeasure)
                .scores("the cat sat down", "the cat sat down");
            close(identical.precision, 1.0);
            close(identical.recall, 1.0);
            close(identical.fmeasure, 1.0);

            // ...and a fully disjoint candidate scores 0.0.
            let disjoint = RougeScore::new(rouge_type, RougeMode::FMeasure)
                .scores("the cat sat down", "dogs run quickly outside");
            close(disjoint.precision, 0.0);
            close(disjoint.recall, 0.0);
            close(disjoint.fmeasure, 0.0);
        }
    }

    #[test]
    fn rouge_handles_empty_candidate_and_reference_without_dividing_by_zero() {
        for rouge_type in [RougeType::Rouge1, RougeType::Rouge2, RougeType::RougeL] {
            let metric = RougeScore::new(rouge_type, RougeMode::FMeasure);

            // Empty candidate: overlap 0, candidate count 0 -> precision 0, recall 0, f 0.
            let empty_candidate = metric.scores("", "the cat sat");
            assert!(empty_candidate.precision.is_finite());
            assert!(empty_candidate.recall.is_finite());
            assert!(empty_candidate.fmeasure.is_finite());
            close(empty_candidate.precision, 0.0);
            close(empty_candidate.recall, 0.0);
            close(empty_candidate.fmeasure, 0.0);

            // Empty reference: symmetric.
            let empty_reference = metric.scores("the cat sat", "");
            close(empty_reference.recall, 0.0);
            close(empty_reference.fmeasure, 0.0);

            // Both empty: still no panic / NaN.
            let both_empty = metric.scores("", "");
            close(both_empty.fmeasure, 0.0);
        }
    }

    #[test]
    fn rouge_detailed_result_reports_selected_component_and_tokenizer() {
        let result = RougeScore::new(RougeType::Rouge1, RougeMode::Recall)
            .detailed("the cat sat", "the cat sat on the mat");
        assert_eq!(result.metric_name, "rouge_score");
        assert_score_close(&result, 0.5);
        let reason = result.reason.as_deref().unwrap_or("");
        assert!(reason.contains("rouge1"));
        assert!(reason.contains("recall"));
        assert!(reason.contains("whitespace-lowercase"));
    }

    #[tokio::test]
    async fn rouge_metric_scores_response_against_reference() {
        let metric = RougeScore::new(RougeType::Rouge1, RougeMode::FMeasure);
        let sample = SingleTurnSample::new("q", "the cat sat", vec!["ctx".to_string()])
            .with_reference("the cat sat on the mat");

        let result = metric.score(&sample).await.expect("rouge score");

        assert_eq!(result.metric_name, "rouge_score");
        close(
            result
                .value
                .and_then(|value| value.as_numeric())
                .expect("numeric"),
            2.0 / 3.0,
        );
    }

    #[tokio::test]
    async fn rouge_metric_requires_a_reference() {
        let metric = RougeScore::default();
        // SingleTurnSample::new leaves `reference` as None.
        let sample = SingleTurnSample::new("q", "the cat sat", vec!["ctx".to_string()]);

        let error = metric.score(&sample).await.expect_err("missing reference");

        assert!(error.to_string().contains("reference"));
    }
}
