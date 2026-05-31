use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{EvaluationDataset, Metric, MetricResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationOptions {
    pub concurrency: usize,
}

impl Default for EvaluationOptions {
    fn default() -> Self {
        Self { concurrency: 4 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationReport {
    pub results: Vec<SampleEvaluation>,
    pub metric_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SampleEvaluation {
    pub sample_index: usize,
    pub results: Vec<MetricResult>,
}

pub async fn evaluate(
    _dataset: &EvaluationDataset,
    _metrics: &[Arc<dyn Metric>],
    _options: EvaluationOptions,
) -> EvaluationReport {
    unimplemented!("TEST-4.1.1: async evaluate orchestration is not implemented yet")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FnMetric, MetricValue, RagasError, SingleTurnSample};

    #[tokio::test]
    async fn test_4_1_1_evaluate_runs_all_metrics_and_isolates_failures() {
        // SCEN-4.1.1 / AC1 / TEST-4.1.1
        let dataset = EvaluationDataset::new(vec![
            SingleTurnSample::new("q1", "a1", vec!["c1".to_string()]),
            SingleTurnSample::new("q2", "a2", vec!["c2".to_string()]),
        ])
        .expect("dataset");
        let ok_metric = Arc::new(FnMetric::new("ok", |sample: &SingleTurnSample| {
            let score = sample.response.len() as f64;
            Box::pin(async move { Ok(MetricResult::success("ok", MetricValue::numeric(score))) })
        }));
        let failing_metric = Arc::new(FnMetric::new("fail", |_sample: &SingleTurnSample| {
            Box::pin(async move {
                Err(RagasError::Provider {
                    message: "provider failed".to_string(),
                })
            })
        }));
        let metrics: Vec<Arc<dyn Metric>> = vec![ok_metric, failing_metric];

        let report = evaluate(&dataset, &metrics, EvaluationOptions { concurrency: 2 }).await;

        assert_eq!(report.metric_names, vec!["ok", "fail"]);
        assert_eq!(report.results.len(), 2);
        for sample in &report.results {
            assert_eq!(sample.results.len(), 2);
            assert_eq!(sample.results[0].metric_name, "ok");
            assert!(sample.results[0].error.is_none());
            assert_eq!(sample.results[1].metric_name, "fail");
            assert_eq!(sample.results[1].error.as_deref(), Some("provider error: provider failed"));
        }
    }
}
