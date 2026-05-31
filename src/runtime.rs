use std::{future::Future, pin::Pin};

use serde::{Deserialize, Serialize};

use crate::RagasError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunConfig {
    pub timeout: TimeoutConfig,
    pub retry: RetryConfig,
    pub concurrency: usize,
    pub cancellation: CancellationConfig,
    pub seed: u64,
}

impl RunConfig {
    pub fn builder() -> RunConfigBuilder {
        RunConfigBuilder::default()
    }

    pub fn validate(&self) -> Result<(), RunConfigError> {
        if self.timeout.per_operation_ms == 0 {
            return Err(RunConfigError::new(
                "timeout.per_operation_ms",
                "per-operation timeout must be greater than 0 ms",
            ));
        }
        if self
            .timeout
            .total_ms
            .is_some_and(|total_ms| total_ms < self.timeout.per_operation_ms)
        {
            return Err(RunConfigError::new(
                "timeout.total_ms",
                "total timeout must be greater than or equal to per_operation_ms",
            ));
        }
        if self.retry.max_attempts == 0 {
            return Err(RunConfigError::new(
                "retry.max_attempts",
                "retry max_attempts must be at least 1",
            ));
        }
        if self.retry.initial_backoff_ms == 0 {
            return Err(RunConfigError::new(
                "retry.initial_backoff_ms",
                "initial_backoff_ms must be greater than 0",
            ));
        }
        if self.retry.max_backoff_ms == 0 {
            return Err(RunConfigError::new(
                "retry.max_backoff_ms",
                "max_backoff_ms must be greater than 0",
            ));
        }
        if self.retry.initial_backoff_ms > self.retry.max_backoff_ms {
            return Err(RunConfigError::new(
                "retry.initial_backoff_ms",
                "initial_backoff_ms must be less than or equal to max_backoff_ms",
            ));
        }
        if self.concurrency == 0 {
            return Err(RunConfigError::new(
                "concurrency",
                "concurrency must be at least 1",
            ));
        }
        Ok(())
    }
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            timeout: TimeoutConfig::default(),
            retry: RetryConfig::default(),
            concurrency: 16,
            cancellation: CancellationConfig::default(),
            seed: 42,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub per_operation_ms: u64,
    pub total_ms: Option<u64>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            per_operation_ms: 180_000,
            total_ms: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 10,
            initial_backoff_ms: 1_000,
            max_backoff_ms: 60_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancellationConfig {
    pub cooperative: bool,
    pub cancel_on_first_error: bool,
}

impl Default for CancellationConfig {
    fn default() -> Self {
        Self {
            cooperative: true,
            cancel_on_first_error: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunConfigError {
    pub field: String,
    pub message: String,
}

impl RunConfigError {
    fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RunConfigBuilder {
    timeout: Option<TimeoutConfig>,
    retry: Option<RetryConfig>,
    concurrency: Option<usize>,
    cancellation: Option<CancellationConfig>,
    seed: Option<u64>,
}

type ExecutorFuture<T> = Pin<Box<dyn Future<Output = Result<T, RagasError>> + Send + 'static>>;

pub struct AsyncExecutor<T> {
    config: RunConfig,
    jobs: Vec<ExecutorJob<T>>,
}

impl<T> AsyncExecutor<T>
where
    T: Send + 'static,
{
    pub fn new(config: RunConfig) -> Self {
        Self {
            config,
            jobs: Vec::new(),
        }
    }

    pub fn submit<F>(mut self, name: impl Into<String>, future: F) -> Self
    where
        F: Future<Output = Result<T, RagasError>> + Send + 'static,
    {
        self.jobs.push(ExecutorJob {
            name: name.into(),
            future: Box::pin(future),
        });
        self
    }

    pub async fn run(self) -> ExecutorReport<T> {
        unimplemented!("TEST-6.2.1 TEST-6.2.2 TEST-6.2.3")
    }
}

struct ExecutorJob<T> {
    name: String,
    future: ExecutorFuture<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressEventKind {
    Started,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub job_index: usize,
    pub job_name: String,
    pub kind: ProgressEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutorOutcome<T> {
    Success(T),
    Failure(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutorJobResult<T> {
    pub job_index: usize,
    pub job_name: String,
    pub outcome: ExecutorOutcome<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutorReport<T> {
    pub results: Vec<ExecutorJobResult<T>>,
    pub events: Vec<ProgressEvent>,
}

impl RunConfigBuilder {
    pub fn timeout_ms(mut self, per_operation_ms: u64) -> Self {
        self.timeout = Some(TimeoutConfig {
            per_operation_ms,
            total_ms: self.timeout.and_then(|timeout| timeout.total_ms),
        });
        self
    }

    pub fn total_timeout_ms(mut self, total_ms: Option<u64>) -> Self {
        let per_operation_ms = self
            .timeout
            .as_ref()
            .map(|timeout| timeout.per_operation_ms)
            .unwrap_or(180_000);
        self.timeout = Some(TimeoutConfig {
            per_operation_ms,
            total_ms,
        });
        self
    }

    pub fn retry(mut self, max_attempts: u32, initial_backoff_ms: u64, max_backoff_ms: u64) -> Self {
        self.retry = Some(RetryConfig {
            max_attempts,
            initial_backoff_ms,
            max_backoff_ms,
        });
        self
    }

    pub fn concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = Some(concurrency);
        self
    }

    pub fn cancellation(mut self, cooperative: bool, cancel_on_first_error: bool) -> Self {
        self.cancellation = Some(CancellationConfig {
            cooperative,
            cancel_on_first_error,
        });
        self
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn build(self) -> Result<RunConfig, RunConfigError> {
        let default = RunConfig::default();
        let config = RunConfig {
            timeout: self.timeout.unwrap_or(default.timeout),
            retry: self.retry.unwrap_or(default.retry),
            concurrency: self.concurrency.unwrap_or(default.concurrency),
            cancellation: self.cancellation.unwrap_or(default.cancellation),
            seed: self.seed.unwrap_or(default.seed),
        };
        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvaluationOptions;
    use std::{
        thread,
        time::Duration,
    };

    #[test]
    fn test_6_1_1_run_config_stores_timeout_retry_concurrency_and_cancellation() {
        // SCEN-6.1.1 / AC1 / TEST-6.1.1
        let config = RunConfig::builder()
            .timeout_ms(5_000)
            .total_timeout_ms(Some(30_000))
            .retry(3, 25, 250)
            .concurrency(8)
            .cancellation(false, true)
            .seed(7)
            .build()
            .expect("valid run config");

        assert_eq!(config.timeout.per_operation_ms, 5_000);
        assert_eq!(config.timeout.total_ms, Some(30_000));
        assert_eq!(config.retry.max_attempts, 3);
        assert_eq!(config.retry.initial_backoff_ms, 25);
        assert_eq!(config.retry.max_backoff_ms, 250);
        assert_eq!(config.concurrency, 8);
        assert!(!config.cancellation.cooperative);
        assert!(config.cancellation.cancel_on_first_error);
        assert_eq!(config.seed, 7);

        assert_eq!(EvaluationOptions::from_run_config(&config).concurrency, 8);
    }

    #[test]
    fn test_6_1_2_run_config_defaults_are_conservative_and_deterministic() {
        // SCEN-6.1.2 / AC2 / TEST-6.1.2
        let config = RunConfig::default();

        assert_eq!(config.timeout.per_operation_ms, 180_000);
        assert_eq!(config.timeout.total_ms, None);
        assert_eq!(config.retry.max_attempts, 10);
        assert_eq!(config.retry.initial_backoff_ms, 1_000);
        assert_eq!(config.retry.max_backoff_ms, 60_000);
        assert_eq!(config.concurrency, 16);
        assert!(config.cancellation.cooperative);
        assert!(!config.cancellation.cancel_on_first_error);
        assert_eq!(config.seed, 42);
        config.validate().expect("default config is valid");
    }

    #[test]
    fn test_6_1_3_invalid_run_config_returns_structured_errors() {
        // SCEN-6.1.3 / AC3 / TEST-6.1.3
        let concurrency_error = RunConfig::builder()
            .concurrency(0)
            .build()
            .expect_err("zero concurrency is invalid");
        assert_eq!(concurrency_error.field, "concurrency");
        assert!(concurrency_error.message.contains("at least 1"));

        let retry_error = RunConfig::builder()
            .retry(3, 2_000, 1_000)
            .build()
            .expect_err("initial backoff cannot exceed max backoff");
        assert_eq!(retry_error.field, "retry.initial_backoff_ms");
        assert!(retry_error.message.contains("max_backoff_ms"));
    }

    #[tokio::test]
    async fn test_6_2_1_executor_preserves_output_order_for_concurrent_tasks() {
        // SCEN-6.2.1 / AC1 / TEST-6.2.1
        let config = RunConfig::builder()
            .concurrency(3)
            .build()
            .expect("valid config");
        let report = AsyncExecutor::new(config)
            .submit("slow", async {
                thread::sleep(Duration::from_millis(30));
                Ok("first")
            })
            .submit("fast", async {
                thread::sleep(Duration::from_millis(1));
                Ok("second")
            })
            .submit("middle", async {
                thread::sleep(Duration::from_millis(10));
                Ok("third")
            })
            .run()
            .await;

        let values = report
            .results
            .iter()
            .map(|result| match &result.outcome {
                ExecutorOutcome::Success(value) => *value,
                ExecutorOutcome::Failure(error) => panic!("unexpected failure: {error}"),
            })
            .collect::<Vec<_>>();

        assert_eq!(values, vec!["first", "second", "third"]);
    }

    #[tokio::test]
    async fn test_6_2_2_executor_records_partial_failures_without_aborting_unrelated_work() {
        // SCEN-6.2.2 / AC2 / TEST-6.2.2
        let config = RunConfig::builder()
            .concurrency(2)
            .build()
            .expect("valid config");
        let report = AsyncExecutor::new(config)
            .submit("ok-1", async { Ok(10) })
            .submit("bad", async {
                Err(RagasError::Provider {
                    message: "provider failed".to_string(),
                })
            })
            .submit("ok-2", async { Ok(30) })
            .run()
            .await;

        assert_eq!(report.results.len(), 3);
        assert!(matches!(report.results[0].outcome, ExecutorOutcome::Success(10)));
        assert!(matches!(
            &report.results[1].outcome,
            ExecutorOutcome::Failure(message) if message.contains("provider failed")
        ));
        assert!(matches!(report.results[2].outcome, ExecutorOutcome::Success(30)));
    }

    #[tokio::test]
    async fn test_6_2_3_executor_emits_progress_events_for_start_success_and_failure() {
        // SCEN-6.2.3 / AC3 / TEST-6.2.3
        let report = AsyncExecutor::new(RunConfig::default())
            .submit("ok", async { Ok("done") })
            .submit("bad", async {
                Err(RagasError::Provider {
                    message: "nope".to_string(),
                })
            })
            .run()
            .await;

        assert!(report.events.iter().any(|event| {
            event.job_name == "ok" && event.kind == ProgressEventKind::Started
        }));
        assert!(report.events.iter().any(|event| {
            event.job_name == "ok" && event.kind == ProgressEventKind::Succeeded
        }));
        assert!(report.events.iter().any(|event| {
            event.job_name == "bad" && event.kind == ProgressEventKind::Started
        }));
        assert!(report.events.iter().any(|event| {
            event.job_name == "bad" && event.kind == ProgressEventKind::Failed
        }));
    }
}
