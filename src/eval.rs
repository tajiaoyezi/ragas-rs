use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

use crate::RunConfig;
use crate::{EvaluationDataset, Metric, MetricResult, RagasError, UsageSummary, UsageTracker};

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
    /// Token usage actually consumed during evaluation, aggregated per provider and per metric.
    /// Populated by [`evaluate_with_config`] when an [`EvaluationConfig::usage_tracker`] is
    /// supplied (the metrics must be wrapped in a `UsageRecordingLlmProvider` sharing that
    /// tracker); all-zero otherwise. `#[serde(default)]` keeps older reports deserializable.
    #[serde(default)]
    pub usage: UsageSummary,
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
    let metric_names = metric_names(metrics);
    // `raise_exceptions = false` collects per-cell failures and never returns Err.
    let results = run_scoring(dataset, metrics, options.concurrency, false)
        .await
        .expect("run_scoring is infallible when raise_exceptions = false");
    EvaluationReport {
        results,
        metric_names,
        usage: UsageSummary::default(),
    }
}

fn metric_names(metrics: &[Arc<dyn Metric>]) -> Vec<String> {
    metrics
        .iter()
        .map(|metric| metric.name().to_string())
        .collect()
}

/// Fan out `metric.score` over every (sample, metric) cell, bounded by `concurrency`.
///
/// When `raise_exceptions` is false (the default), a metric error is recorded as a per-cell
/// [`MetricResult::failure`] and scoring continues — matching Python ragas's `np.nan` sentinel.
///
/// When true, the run aborts with the first failing cell (a metric `Err`, or a panicked/cancelled
/// task) in deterministic `(sample_index, metric_index)` order. Note two intentional divergences
/// from Python's `raise_exceptions=True`: Python raises whichever errored cell *completes* first
/// (nondeterministic), and it stops scheduling further work; this implementation lets all in-flight
/// cells finish, then returns the order-deterministic first error.
async fn run_scoring(
    dataset: &EvaluationDataset,
    metrics: &[Arc<dyn Metric>],
    concurrency: usize,
    raise_exceptions: bool,
) -> Result<Vec<SampleEvaluation>, RagasError> {
    let names = metric_names(metrics);
    let metric_count = metrics.len();
    let sample_count = dataset.len();
    let concurrency = concurrency.max(1);
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();

    for (sample_index, sample) in dataset.iter().cloned().enumerate() {
        for (metric_index, metric) in metrics.iter().cloned().enumerate() {
            let semaphore = Arc::clone(&semaphore);
            let sample = sample.clone();
            handles.push(tokio::spawn(async move {
                let _permit = semaphore.acquire_owned().await.expect("semaphore open");
                let outcome = metric.score(&sample).await;
                (sample_index, metric_index, outcome)
            }));
        }
    }

    // A `None` cell means the spawned task itself failed to join (panic/cancel).
    let mut cells: Vec<Vec<Option<Result<MetricResult, RagasError>>>> = (0..sample_count)
        .map(|_| (0..metric_count).map(|_| None).collect())
        .collect();
    for handle in handles {
        if let Ok((sample_index, metric_index, outcome)) = handle.await
            && let Some(row) = cells.get_mut(sample_index)
            && let Some(cell) = row.get_mut(metric_index)
        {
            *cell = Some(outcome);
        }
    }

    if raise_exceptions {
        for (sample_index, row) in cells.iter_mut().enumerate() {
            for (metric_index, cell) in row.iter_mut().enumerate() {
                // A None cell is a panicked/cancelled task; under fail-fast it must abort too,
                // not silently degrade to a recorded failure.
                if cell.is_none() {
                    return Err(RagasError::Provider {
                        message: format!(
                            "metric '{}' task failed for sample {sample_index}",
                            names[metric_index]
                        ),
                    });
                }
                if matches!(cell, Some(Err(_)))
                    && let Some(Err(error)) = cell.take()
                {
                    return Err(error);
                }
            }
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
                .map(|(metric_index, cell)| match cell {
                    Some(Ok(result)) => result,
                    Some(Err(error)) => {
                        MetricResult::failure(&names[metric_index], error.to_string())
                    }
                    None => MetricResult::failure(&names[metric_index], "metric task failed"),
                })
                .collect(),
        })
        .collect();

    Ok(results)
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

/// Full evaluation options carrier — the faithful analog of Python ragas's `evaluate(...)`
/// keyword arguments that govern run behavior (as opposed to the dataset/metrics themselves).
///
/// - `run_config` drives concurrency (and, at provider-construction time, retry/timeout — see
///   [`evaluate_with`]).
/// - `raise_exceptions` toggles fail-fast vs collect-and-continue (Python default: `false`).
/// - `usage_tracker`, when supplied, is read after the run and its summary is attached to the
///   returned [`EvaluationReport::usage`]. Wrap each LLM metric's provider in a
///   `UsageRecordingLlmProvider` sharing this same tracker so the recorded usage reflects the
///   calls actually made.
#[derive(Clone, Default)]
pub struct EvaluationConfig {
    pub run_config: RunConfig,
    pub raise_exceptions: bool,
    pub usage_tracker: Option<Arc<Mutex<UsageTracker>>>,
}

impl EvaluationConfig {
    pub fn new(run_config: RunConfig) -> Self {
        Self {
            run_config,
            raise_exceptions: false,
            usage_tracker: None,
        }
    }

    pub fn raise_exceptions(mut self, raise_exceptions: bool) -> Self {
        self.raise_exceptions = raise_exceptions;
        self
    }

    pub fn with_usage_tracker(mut self, usage_tracker: Arc<Mutex<UsageTracker>>) -> Self {
        self.usage_tracker = Some(usage_tracker);
        self
    }
}

/// Run evaluation under a full [`EvaluationConfig`]. Returns `Err` only when
/// `raise_exceptions` is true and a metric fails (the first error, in `(sample, metric)` order);
/// with `raise_exceptions = false` it always returns `Ok`, with failures recorded per-cell.
/// On success the report's `usage` is filled from the config's `usage_tracker` (if any).
pub async fn evaluate_with_config(
    dataset: &EvaluationDataset,
    metrics: &[Arc<dyn Metric>],
    config: &EvaluationConfig,
) -> Result<EvaluationReport, RagasError> {
    let metric_names = metric_names(metrics);
    let results = run_scoring(
        dataset,
        metrics,
        config.run_config.concurrency,
        config.raise_exceptions,
    )
    .await?;
    let usage = config
        .usage_tracker
        .as_ref()
        .map(|tracker| {
            tracker
                .lock()
                .expect("usage tracker not poisoned")
                .summary()
        })
        .unwrap_or_default();
    Ok(EvaluationReport {
        results,
        metric_names,
        usage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FnMetric, MetricValue, RagasError, SingleTurnSample, TokenUsage};

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

    fn one_sample_dataset() -> EvaluationDataset {
        EvaluationDataset::new(vec![SingleTurnSample::new("q", "a", vec!["c".to_string()])])
            .expect("dataset")
    }

    fn ok_metric() -> Arc<dyn Metric> {
        Arc::new(FnMetric::new("ok", |_: &SingleTurnSample| {
            Box::pin(async { Ok(MetricResult::success("ok", MetricValue::numeric(1.0))) })
        }))
    }

    fn failing_metric() -> Arc<dyn Metric> {
        Arc::new(FnMetric::new("fail", |_: &SingleTurnSample| {
            Box::pin(async {
                Err(RagasError::Provider {
                    message: "boom".to_string(),
                })
            })
        }))
    }

    #[tokio::test]
    async fn evaluate_with_config_raise_exceptions_propagates_first_error() {
        let dataset = one_sample_dataset();
        let metrics = vec![ok_metric(), failing_metric()];

        // raise_exceptions = true -> the first failing cell aborts the run with its real error.
        let config = EvaluationConfig::new(RunConfig::default()).raise_exceptions(true);
        let error = evaluate_with_config(&dataset, &metrics, &config)
            .await
            .expect_err("a failing metric must abort when raise_exceptions = true");
        assert!(matches!(error, RagasError::Provider { message } if message == "boom"));
    }

    #[tokio::test]
    async fn evaluate_with_config_collects_failures_when_not_raising() {
        let dataset = one_sample_dataset();
        let metrics = vec![ok_metric(), failing_metric()];

        // raise_exceptions = false (default) -> the failure is recorded per-cell, run continues.
        let config = EvaluationConfig::new(RunConfig::default());
        let report = evaluate_with_config(&dataset, &metrics, &config)
            .await
            .expect("raise_exceptions = false never errors");
        assert_eq!(report.metric_names, vec!["ok", "fail"]);
        let cells = &report.results[0].results;
        assert!(cells[0].error.is_none());
        assert_eq!(cells[1].metric_name, "fail");
        assert!(cells[1].error.is_some());
        // No tracker supplied -> usage is all-zero.
        assert_eq!(report.usage.total.total_tokens, 0);
    }

    #[tokio::test]
    async fn evaluate_with_config_attaches_usage_summary_from_tracker() {
        let dataset = one_sample_dataset();
        let metrics = vec![ok_metric()];

        // A shared tracker carrying pre-recorded usage is surfaced onto the report. (In the real
        // pipeline the tracker is populated by UsageRecordingLlmProvider during scoring.)
        let tracker = Arc::new(Mutex::new(UsageTracker::new()));
        tracker.lock().unwrap().record(
            "chat",
            "faithfulness",
            TokenUsage {
                prompt_tokens: Some(12),
                completion_tokens: Some(8),
                total_tokens: Some(20),
            },
        );
        let config =
            EvaluationConfig::new(RunConfig::default()).with_usage_tracker(Arc::clone(&tracker));

        let report = evaluate_with_config(&dataset, &metrics, &config)
            .await
            .expect("evaluation succeeds");
        assert_eq!(report.usage.total.total_tokens, 20);
        assert_eq!(report.usage.total.prompt_tokens, 12);
        assert_eq!(report.usage.by_metric["faithfulness"].completion_tokens, 8);
        assert_eq!(report.usage.by_provider["chat"].total_tokens, 20);
    }

    #[tokio::test]
    async fn evaluate_with_config_raise_exceptions_propagates_a_panicked_task() {
        // A metric task that panics surfaces as a join failure (a None cell). Under fail-fast it
        // must abort with an error, not silently degrade to a recorded failure. (The panic message
        // the runtime prints to stderr during this test is expected and harmless.)
        let dataset = one_sample_dataset();
        let panicking = Arc::new(FnMetric::new("panic", |_: &SingleTurnSample| {
            Box::pin(async { panic!("metric blew up") })
        }));
        let metrics: Vec<Arc<dyn Metric>> = vec![panicking];

        let config = EvaluationConfig::new(RunConfig::default()).raise_exceptions(true);
        let error = evaluate_with_config(&dataset, &metrics, &config)
            .await
            .expect_err("a panicked task must abort under raise_exceptions = true");
        assert!(
            error.to_string().contains("task failed"),
            "unexpected error: {error}"
        );
    }
}
