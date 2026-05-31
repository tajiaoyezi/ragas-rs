use serde::{Deserialize, Serialize};

use crate::{DetailedMetricResult, MetricValueType, ScoreNormalizationPolicy};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RubricCriterion {
    pub name: String,
    pub description: String,
    pub weight: f64,
}

impl RubricCriterion {
    pub fn new(name: impl Into<String>, description: impl Into<String>, weight: f64) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            weight,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RubricMetric {
    pub name: String,
    pub criteria: Vec<RubricCriterion>,
}

impl RubricMetric {
    pub fn new(name: impl Into<String>, criteria: Vec<RubricCriterion>) -> Self {
        Self {
            name: name.into(),
            criteria,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AspectCriticMode {
    Binary { threshold: f64 },
    Graded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AspectCriticConfig {
    pub mode: AspectCriticMode,
}

impl AspectCriticConfig {
    pub fn binary(threshold: f64) -> Self {
        Self {
            mode: AspectCriticMode::Binary { threshold },
        }
    }

    pub fn graded() -> Self {
        Self {
            mode: AspectCriticMode::Graded,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainRubric {
    pub domain: String,
    pub criteria: Vec<RubricCriterion>,
}

impl DomainRubric {
    pub fn new(domain: impl Into<String>, criteria: Vec<RubricCriterion>) -> Self {
        Self {
            domain: domain.into(),
            criteria,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstanceRubric {
    pub instance_id: String,
    pub criteria: Vec<RubricCriterion>,
    pub notes: Option<String>,
}

impl InstanceRubric {
    pub fn new(instance_id: impl Into<String>, criteria: Vec<RubricCriterion>) -> Self {
        Self {
            instance_id: instance_id.into(),
            criteria,
            notes: None,
        }
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

pub fn score_aspect_critic(raw_score: f64, config: AspectCriticConfig) -> DetailedMetricResult {
    let raw_score = raw_score.clamp(0.0, 1.0);
    let (score, reason) = match config.mode {
        AspectCriticMode::Binary { threshold } => {
            let passed = raw_score >= threshold;
            (
                if passed { 1.0 } else { 0.0 },
                format!("binary aspect critic: raw_score={raw_score:.3} threshold={threshold:.3}"),
            )
        }
        AspectCriticMode::Graded => (
            raw_score,
            format!("graded aspect critic: raw_score={raw_score:.3}"),
        ),
    };

    DetailedMetricResult::new("aspect_critic", MetricValueType::Numeric)
        .with_score(score, ScoreNormalizationPolicy::Reject)
        .expect("aspect critic score is normalized")
        .with_reason(reason)
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
    fn test_12_1_1_rubric_metrics_accept_typed_criteria() {
        // SCEN-12.1.1 / AC1 / TEST-12.1.1
        let criterion = RubricCriterion::new("grounding", "Answer must cite context", 0.7);
        let metric = RubricMetric::new("domain_quality", vec![criterion.clone()]);

        assert_eq!(metric.name, "domain_quality");
        assert_eq!(metric.criteria, vec![criterion]);
        assert_eq!(metric.criteria[0].weight, 0.7);
    }

    #[test]
    fn test_12_1_2_aspect_critic_returns_binary_or_graded_result_by_config() {
        // SCEN-12.1.2 / AC2 / TEST-12.1.2
        let binary = score_aspect_critic(0.82, AspectCriticConfig::binary(0.8));
        assert_score_close(&binary, 1.0);
        assert!(binary.reason.as_deref().unwrap_or("").contains("binary"));

        let graded = score_aspect_critic(0.72, AspectCriticConfig::graded());
        assert_score_close(&graded, 0.72);
        assert!(graded.reason.as_deref().unwrap_or("").contains("graded"));
    }

    #[test]
    fn test_12_1_3_domain_and_instance_rubrics_serialize_for_audit() {
        // SCEN-12.1.3 / AC3 / TEST-12.1.3
        let criterion = RubricCriterion::new("style", "Use concise wording", 0.4);
        let domain = DomainRubric::new("support", vec![criterion.clone()]);
        let instance = InstanceRubric::new("row-42", vec![criterion]).with_notes("customer tier: enterprise");

        let domain_json = serde_json::to_string(&domain).expect("domain rubric JSON");
        let instance_json = serde_json::to_string(&instance).expect("instance rubric JSON");

        assert!(domain_json.contains("\"domain\":\"support\""));
        assert!(domain_json.contains("\"style\""));
        assert!(instance_json.contains("\"instance_id\":\"row-42\""));
        assert!(instance_json.contains("customer tier"));

        let roundtrip: InstanceRubric =
            serde_json::from_str(&instance_json).expect("roundtrip instance rubric");
        assert_eq!(roundtrip.notes.as_deref(), Some("customer tier: enterprise"));
        assert_eq!(roundtrip.criteria.len(), 1);
    }
}
