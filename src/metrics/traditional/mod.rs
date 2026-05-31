use std::collections::BTreeMap;

use crate::{DetailedMetricResult, MetricEvidence, MetricValueType, ScoreNormalizationPolicy};

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
        assert!(exact.reason.as_deref().unwrap_or("").contains("provider-free"));

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
        assert!(bleu.reason.as_deref().unwrap_or("").contains("whitespace-lowercase"));

        let rouge = rouge_l_recall("the cat sat", "the cat slept");
        assert_score_close(&rouge, 2.0 / 3.0);

        let chrf = chrf_score("abc", "abd");
        assert_score_close(&chrf, 2.0 / 3.0);
        assert!(chrf.reason.as_deref().unwrap_or("").contains("character-unigram"));
    }

    #[test]
    fn test_11_1_3_traditional_metrics_handle_empty_strings_explicitly() {
        // SCEN-11.1.3 / AC3 / TEST-11.1.3
        let exact = exact_match("", "");
        assert_score_close(&exact, 1.0);
        assert!(exact.reason.as_deref().unwrap_or("").contains("both strings empty"));

        let bleu = bleu_unigram("", "reference text");
        assert_score_close(&bleu, 0.0);
        assert!(bleu.reason.as_deref().unwrap_or("").contains("empty candidate"));

        let chrf = chrf_score("", "");
        assert_score_close(&chrf, 1.0);
        assert!(chrf.reason.as_deref().unwrap_or("").contains("both strings empty"));
    }
}
