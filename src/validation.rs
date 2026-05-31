use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{EvaluationDataset, Metric, SingleTurnSample};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleField {
    UserInput,
    Response,
    RetrievedContexts,
    Reference,
    Metadata(String),
}

impl SampleField {
    pub fn path(&self) -> String {
        match self {
            Self::UserInput => "user_input".to_string(),
            Self::Response => "response".to_string(),
            Self::RetrievedContexts => "retrieved_contexts".to_string(),
            Self::Reference => "reference".to_string(),
            Self::Metadata(key) => format!("metadata.{key}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRequirements {
    metric_name: String,
    required_fields: Vec<SampleField>,
}

impl MetricRequirements {
    pub fn new(metric_name: impl Into<String>, required_fields: Vec<SampleField>) -> Self {
        Self {
            metric_name: metric_name.into(),
            required_fields,
        }
    }

    pub fn metric_name(&self) -> &str {
        &self.metric_name
    }

    pub fn required_fields(&self) -> &[SampleField] {
        &self.required_fields
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub sample_index: usize,
    pub metric_name: String,
    pub field_path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    fn new(issues: Vec<ValidationIssue>) -> Self {
        Self { issues }
    }

    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }
}

pub fn validate_single_turn_samples(
    samples: &[SingleTurnSample],
    requirements: &[MetricRequirements],
) -> Result<(), ValidationReport> {
    let mut issues = Vec::new();
    for (sample_index, sample) in samples.iter().enumerate() {
        for requirement in requirements {
            for field in requirement.required_fields() {
                if !field_is_present(sample, field) {
                    let field_path = field.path();
                    issues.push(ValidationIssue {
                        sample_index,
                        metric_name: requirement.metric_name().to_string(),
                        message: format!(
                            "metric {} requires non-empty field {}",
                            requirement.metric_name(),
                            field_path
                        ),
                        field_path,
                    });
                }
            }
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(ValidationReport::new(issues))
    }
}

pub fn validate_dataset_requirements(
    dataset: &EvaluationDataset,
    requirements: &[MetricRequirements],
) -> Result<(), ValidationReport> {
    validate_single_turn_samples(dataset.samples(), requirements)
}

pub fn validate_before_evaluate(
    dataset: &EvaluationDataset,
    metrics: &[Arc<dyn Metric>],
) -> Result<(), ValidationReport> {
    let requirements = metrics
        .iter()
        .map(|metric| metric.requirements())
        .collect::<Vec<_>>();
    validate_dataset_requirements(dataset, &requirements)
}

fn field_is_present(sample: &SingleTurnSample, field: &SampleField) -> bool {
    match field {
        SampleField::UserInput => !sample.user_input.trim().is_empty(),
        SampleField::Response => !sample.response.trim().is_empty(),
        SampleField::RetrievedContexts => sample
            .retrieved_contexts
            .iter()
            .any(|context| !context.trim().is_empty()),
        SampleField::Reference => sample
            .reference
            .as_deref()
            .is_some_and(|reference| !reference.trim().is_empty()),
        SampleField::Metadata(key) => sample
            .metadata
            .get(key)
            .is_some_and(|value| !value.trim().is_empty()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use crate::{FnMetric, MetricResult, MetricValue};

    #[test]
    fn test_5_3_1_validator_detects_missing_fields_required_by_metric() {
        // SCEN-5.3.1 / AC1 / TEST-5.3.1
        let dataset = EvaluationDataset::from_sample(SingleTurnSample::new(
            "question",
            "answer",
            vec!["context".to_string()],
        ))
        .expect("dataset");
        let requirements = vec![MetricRequirements::new(
            "answer_correctness",
            vec![SampleField::Reference],
        )];

        let report =
            validate_dataset_requirements(&dataset, &requirements).expect_err("missing reference");

        assert_eq!(report.issues().len(), 1);
        assert_eq!(report.issues()[0].sample_index, 0);
        assert_eq!(report.issues()[0].metric_name, "answer_correctness");
        assert_eq!(report.issues()[0].field_path, "reference");
    }

    #[test]
    fn test_5_3_2_validator_reports_sample_index_and_field_path_for_invalid_records() {
        // SCEN-5.3.2 / AC2 / TEST-5.3.2
        let samples = vec![
            SingleTurnSample::new("valid", "answer", vec!["context".to_string()]),
            SingleTurnSample::new("missing response", "", vec!["context".to_string()]),
            SingleTurnSample::new("missing contexts", "answer", vec![]),
        ];
        let requirements = vec![MetricRequirements::new(
            "faithfulness",
            vec![
                SampleField::Response,
                SampleField::RetrievedContexts,
                SampleField::UserInput,
            ],
        )];

        let report =
            validate_single_turn_samples(&samples, &requirements).expect_err("invalid samples");

        assert!(report.issues().iter().any(|issue| {
            issue.sample_index == 1
                && issue.metric_name == "faithfulness"
                && issue.field_path == "response"
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.sample_index == 2
                && issue.metric_name == "faithfulness"
                && issue.field_path == "retrieved_contexts"
        }));
    }

    #[test]
    fn test_5_3_3_validation_fails_before_evaluate_without_provider_or_scorer_calls() {
        // SCEN-5.3.3 / AC3 / TEST-5.3.3
        let dataset = EvaluationDataset::from_sample(SingleTurnSample::new(
            "question",
            "answer",
            vec!["context".to_string()],
        ))
        .expect("dataset");
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_metric = Arc::clone(&calls);
        let metric = FnMetric::new("needs_reference", move |_sample: &SingleTurnSample| {
            calls_for_metric.fetch_add(1, Ordering::SeqCst);
            Box::pin(async {
                Ok(MetricResult::success(
                    "needs_reference",
                    MetricValue::numeric(1.0),
                ))
            })
        })
        .with_required_fields(vec![SampleField::Reference]);
        let metrics: Vec<Arc<dyn Metric>> = vec![Arc::new(metric)];

        let report = validate_before_evaluate(&dataset, &metrics).expect_err("missing reference");

        assert_eq!(report.issues()[0].field_path, "reference");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
