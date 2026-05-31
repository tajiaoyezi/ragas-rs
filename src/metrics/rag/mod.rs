use std::collections::BTreeSet;

use crate::{DetailedMetricResult, MetricEvidence, MetricValueType, ScoreNormalizationPolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPrecisionVariant {
    RankedRelevance,
    IdOverlap,
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
            evidence.push(MetricEvidence::new(format!("reference[{index}]"), claim.clone()));
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

    let query_tokens = meaningful_tokens(user_input).into_iter().collect::<BTreeSet<_>>();
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
        let context_tokens = meaningful_tokens(context).into_iter().collect::<BTreeSet<_>>();
        let overlap = query_tokens
            .iter()
            .filter(|token| context_tokens.contains(*token))
            .count();
        if overlap > 0 {
            evidence.push(MetricEvidence::new(format!("context[{index}]"), context.clone()));
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
        let has_letter = cleaned.chars().any(|character| character.is_ascii_alphabetic());
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

#[allow(dead_code)]
fn zero_result(metric_name: &str) -> DetailedMetricResult {
    DetailedMetricResult::new(metric_name, MetricValueType::Numeric)
        .with_score(0.0, ScoreNormalizationPolicy::Reject)
        .expect("zero score is valid")
        .with_reason("not implemented")
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
        assert!(ranked.reason.as_deref().unwrap_or("").contains("precision@k"));

        let id_based = id_based_context_precision(
            &["doc-a", "doc-b", "doc-c"],
            &["doc-b", "doc-c", "doc-z"],
        );
        assert_score_close(&id_based, 2.0 / 3.0);
        assert!(id_based.reason.as_deref().unwrap_or("").contains("id overlap"));
    }

    #[test]
    fn test_10_1_2_context_recall_and_entity_recall_use_reference_and_contexts() {
        // SCEN-10.1.2 / AC2 / TEST-10.1.2
        let contexts = vec![
            "Ragas evaluates LLM apps with metrics.".to_string(),
            "OpenAI embeddings score relevance for retrieval.".to_string(),
        ];
        let reference = "Ragas evaluates LLM apps. OpenAI embeddings score relevance. Rust services run fast.";

        let recall = context_recall(reference, &contexts);
        assert_score_close(&recall, 2.0 / 3.0);
        assert_eq!(recall.evidence.len(), 2);
        assert!(recall.reason.as_deref().unwrap_or("").contains("reference claims"));

        let entity_recall = context_entity_recall("Ragas uses OpenAI with Rust.", &contexts);
        assert_score_close(&entity_recall, 2.0 / 3.0);
        assert!(entity_recall.reason.as_deref().unwrap_or("").contains("entities"));
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
        assert!(relevance.reason.as_deref().unwrap_or("").contains("lexical overlap"));
    }
}
