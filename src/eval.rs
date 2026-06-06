use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

use crate::RunConfig;
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

impl EvaluationOptions {
    pub fn from_run_config(config: &RunConfig) -> Self {
        Self {
            concurrency: config.concurrency.max(1),
        }
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
    dataset: &EvaluationDataset,
    metrics: &[Arc<dyn Metric>],
    options: EvaluationOptions,
) -> EvaluationReport {
    let metric_names = metrics
        .iter()
        .map(|metric| metric.name().to_string())
        .collect::<Vec<_>>();
    let concurrency = options.concurrency.max(1);
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();

    for (sample_index, sample) in dataset.iter().cloned().enumerate() {
        for (metric_index, metric) in metrics.iter().cloned().enumerate() {
            let semaphore = Arc::clone(&semaphore);
            let metric_name = metric.name().to_string();
            let sample = sample.clone();
            handles.push(tokio::spawn(async move {
                let _permit = semaphore.acquire_owned().await.expect("semaphore open");
                let result = match metric.score(&sample).await {
                    Ok(result) => result,
                    Err(error) => MetricResult::failure(metric_name, error.to_string()),
                };
                (sample_index, metric_index, result)
            }));
        }
    }

    let mut cells = vec![vec![None; metrics.len()]; dataset.len()];
    for handle in handles {
        if let Ok((sample_index, metric_index, result)) = handle.await
            && let Some(row) = cells.get_mut(sample_index)
            && let Some(cell) = row.get_mut(metric_index)
        {
            *cell = Some(result);
        }
    }

    let results = cells
        .into_iter()
        .enumerate()
        .map(|(sample_index, row)| SampleEvaluation {
            sample_index,
            results: row
                .into_iter()
                .enumerate()
                .map(|(metric_index, result)| {
                    result.unwrap_or_else(|| {
                        MetricResult::failure(&metric_names[metric_index], "metric task failed")
                    })
                })
                .collect(),
        })
        .collect();

    EvaluationReport {
        results,
        metric_names,
    }
}

/// RunConfig-driven evaluation entry point. Runs the same fan-out as [`evaluate`] but takes a
/// [`RunConfig`] instead of bare [`EvaluationOptions`], deriving concurrency via
/// [`EvaluationOptions::from_run_config`]. This is the carrier for run-level concerns — today it
/// governs concurrency; it is the extension point for cancellation/callbacks/usage. The minimal
/// options-only [`evaluate`] is unchanged.
///
/// Note: provider retry/timeout from the `RunConfig` is applied at provider-construction time
/// (wrap the provider with [`crate::ResilientLlmProvider`] before building the metrics), not inside
/// this loop — providers are owned by the metrics, so this function never sees them directly.
pub async fn evaluate_with(
    dataset: &EvaluationDataset,
    metrics: &[Arc<dyn Metric>],
    run_config: &RunConfig,
) -> EvaluationReport {
    evaluate(
        dataset,
        metrics,
        EvaluationOptions::from_run_config(run_config),
    )
    .await
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
            assert_eq!(
                sample.results[1].error.as_deref(),
                Some("provider error: provider failed")
            );
        }
    }

    #[tokio::test]
    async fn evaluate_with_runs_metrics_via_run_config() {
        // The RunConfig-driven carrier runs the same fan-out as evaluate(), deriving concurrency
        // from the RunConfig. Scores match the responses, proving the real metrics ran in order.
        let dataset = EvaluationDataset::new(vec![
            SingleTurnSample::new("q1", "a1", vec!["c1".to_string()]),
            SingleTurnSample::new("q2", "aaa2", vec!["c2".to_string()]),
        ])
        .expect("dataset");
        let len_metric = Arc::new(FnMetric::new("len", |sample: &SingleTurnSample| {
            let score = sample.response.len() as f64;
            Box::pin(async move { Ok(MetricResult::success("len", MetricValue::numeric(score))) })
        }));
        let metrics: Vec<Arc<dyn Metric>> = vec![len_metric];

        let run_config = RunConfig {
            concurrency: 2,
            ..RunConfig::default()
        };
        let report = evaluate_with(&dataset, &metrics, &run_config).await;

        assert_eq!(report.metric_names, vec!["len"]);
        assert_eq!(report.results.len(), 2);
        let score = |index: usize| report.results[index].results[0].value.clone();
        assert_eq!(score(0), Some(MetricValue::numeric(2.0)));
        assert_eq!(score(1), Some(MetricValue::numeric(4.0)));
    }
}
