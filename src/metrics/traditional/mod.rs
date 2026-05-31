use crate::{DetailedMetricResult, MetricEvidence, MetricValueType, ScoreNormalizationPolicy};

pub fn lexical_tokenizer_assumptions() -> Vec<&'static str> {
    vec!["whitespace-lowercase", "character-unigram"]
}

pub fn exact_match(_candidate: &str, _reference: &str) -> DetailedMetricResult {
    zero_result("exact_match")
}

pub fn string_distance_similarity(_candidate: &str, _reference: &str) -> DetailedMetricResult {
    zero_result("string_distance_similarity")
}

pub fn bleu_unigram(_candidate: &str, _reference: &str) -> DetailedMetricResult {
    zero_result("bleu_unigram")
}

pub fn rouge_l_recall(_candidate: &str, _reference: &str) -> DetailedMetricResult {
    zero_result("rouge_l_recall")
}

pub fn chrf_score(_candidate: &str, _reference: &str) -> DetailedMetricResult {
    zero_result("chrf")
}

fn zero_result(metric_name: &str) -> DetailedMetricResult {
    DetailedMetricResult::new(metric_name, MetricValueType::Numeric)
        .with_score(0.0, ScoreNormalizationPolicy::Reject)
        .expect("zero score is valid")
        .with_reason("not implemented")
}

#[allow(dead_code)]
fn unused_evidence(source: impl Into<String>, text: impl Into<String>) -> MetricEvidence {
    MetricEvidence::new(source, text)
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
