use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{Mutex, Semaphore};

use crate::{RagasError, TokenUsage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCacheKey {
    pub function: String,
    pub key: String,
}

pub fn generate_runtime_cache_key(
    function: impl Into<String>,
    args: &[Value],
    kwargs: &HashMap<String, Value>,
) -> RuntimeCacheKey {
    let function = function.into();
    let mut filtered_kwargs = serde_json::Map::new();
    for (key, value) in kwargs {
        if key != "callbacks" {
            filtered_kwargs.insert(key.clone(), value.clone());
        }
    }
    let payload = serde_json::json!({
        "function": function,
        "args": args,
        "kwargs": Value::Object(filtered_kwargs),
    });
    let canonical = canonical_json(&payload);
    let key = stable_hash_hex(&canonical);
    let function = payload["function"]
        .as_str()
        .expect("function is string")
        .to_string();
    RuntimeCacheKey { function, key }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LazyTokenizer {
    encoding_name: String,
    initialized: bool,
}

impl LazyTokenizer {
    pub fn new(encoding_name: impl Into<String>) -> Self {
        Self {
            encoding_name: encoding_name.into(),
            initialized: false,
        }
    }

    pub fn encoding_name(&self) -> &str {
        &self.encoding_name
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn encode(&mut self, text: &str) -> Vec<u32> {
        self.initialized = true;
        text.split_whitespace()
            .enumerate()
            .map(|(index, _)| index as u32)
            .collect()
    }

    pub fn count_tokens(&mut self, text: &str) -> usize {
        self.encode(text).len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelTokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub model: String,
}

impl ModelTokenUsage {
    pub fn new(input_tokens: u32, output_tokens: u32, model: impl Into<String>) -> Self {
        Self {
            input_tokens,
            output_tokens,
            model: model.into(),
        }
    }

    pub fn add(&self, other: &Self) -> Result<Self, RagasError> {
        if self.model != other.model {
            return Err(RagasError::Provider {
                message: format!(
                    "cannot add token usage for different models: {} vs {}",
                    self.model, other.model
                ),
            });
        }
        Ok(Self {
            input_tokens: self.input_tokens + other.input_tokens,
            output_tokens: self.output_tokens + other.output_tokens,
            model: self.model.clone(),
        })
    }

    pub fn cost(&self, input_rate: f64, output_rate: Option<f64>) -> f64 {
        let output_rate = output_rate.unwrap_or(input_rate);
        round_cost(self.input_tokens as f64 * input_rate + self.output_tokens as f64 * output_rate)
    }
}

pub fn total_model_cost(
    usage: &[ModelTokenUsage],
    per_model_rates: &HashMap<String, (f64, f64)>,
) -> Result<f64, RagasError> {
    let mut total = 0.0;
    for usage in usage {
        let Some((input_rate, output_rate)) = per_model_rates.get(&usage.model) else {
            return Err(RagasError::Provider {
                message: format!("missing token cost rates for model {}", usage.model),
            });
        };
        total += usage.cost(*input_rate, Some(*output_rate));
    }
    Ok(round_cost(total))
}

fn round_cost(value: f64) -> f64 {
    (value * 1_000_000_000_000.0).round() / 1_000_000_000_000.0
}

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
        let job_count = self.jobs.len();
        let events = Arc::new(Mutex::new(Vec::with_capacity(job_count * 2)));
        let semaphore = Arc::new(Semaphore::new(self.config.concurrency.max(1)));
        let mut handles = Vec::with_capacity(job_count);

        for (job_index, job) in self.jobs.into_iter().enumerate() {
            let events = Arc::clone(&events);
            let semaphore = Arc::clone(&semaphore);
            handles.push(tokio::spawn(async move {
                let _permit = semaphore.acquire_owned().await.expect("semaphore open");
                push_progress_event(
                    &events,
                    ProgressEvent {
                        job_index,
                        job_name: job.name.clone(),
                        kind: ProgressEventKind::Started,
                    },
                )
                .await;

                let outcome = match job.future.await {
                    Ok(value) => {
                        push_progress_event(
                            &events,
                            ProgressEvent {
                                job_index,
                                job_name: job.name.clone(),
                                kind: ProgressEventKind::Succeeded,
                            },
                        )
                        .await;
                        ExecutorOutcome::Success(value)
                    }
                    Err(error) => {
                        push_progress_event(
                            &events,
                            ProgressEvent {
                                job_index,
                                job_name: job.name.clone(),
                                kind: ProgressEventKind::Failed,
                            },
                        )
                        .await;
                        ExecutorOutcome::Failure(error.to_string())
                    }
                };

                ExecutorJobResult {
                    job_index,
                    job_name: job.name,
                    outcome,
                }
            }));
        }

        let mut ordered_results = (0..job_count).map(|_| None).collect::<Vec<_>>();
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    let index = result.job_index;
                    if let Some(slot) = ordered_results.get_mut(index) {
                        *slot = Some(result);
                    }
                }
                Err(error) => {
                    let index = ordered_results
                        .iter()
                        .position(Option::is_none)
                        .unwrap_or(job_count);
                    if let Some(slot) = ordered_results.get_mut(index) {
                        *slot = Some(ExecutorJobResult {
                            job_index: index,
                            job_name: format!("job-{index}"),
                            outcome: ExecutorOutcome::Failure(format!(
                                "executor task join failed: {error}"
                            )),
                        });
                    }
                }
            }
        }

        let events = {
            let events = events.lock().await;
            events.clone()
        };
        let results = ordered_results.into_iter().flatten().collect::<Vec<_>>();

        ExecutorReport { results, events }
    }
}

async fn push_progress_event(events: &Mutex<Vec<ProgressEvent>>, event: ProgressEvent) {
    let mut events = events.lock().await;
    events.push(event);
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeEventKind {
    EvaluationStarted,
    MetricStarted,
    MetricSucceeded,
    MetricFailed,
    EvaluationFinished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub kind: RuntimeEventKind,
    pub run_id: String,
    pub metric_name: Option<String>,
    pub sample_index: Option<usize>,
}

impl RuntimeEvent {
    pub fn evaluation_started(run_id: impl Into<String>) -> Self {
        Self {
            kind: RuntimeEventKind::EvaluationStarted,
            run_id: run_id.into(),
            metric_name: None,
            sample_index: None,
        }
    }

    pub fn metric_started(
        run_id: impl Into<String>,
        metric_name: impl Into<String>,
        sample_index: usize,
    ) -> Self {
        Self {
            kind: RuntimeEventKind::MetricStarted,
            run_id: run_id.into(),
            metric_name: Some(metric_name.into()),
            sample_index: Some(sample_index),
        }
    }

    pub fn metric_succeeded(
        run_id: impl Into<String>,
        metric_name: impl Into<String>,
        sample_index: usize,
    ) -> Self {
        Self {
            kind: RuntimeEventKind::MetricSucceeded,
            run_id: run_id.into(),
            metric_name: Some(metric_name.into()),
            sample_index: Some(sample_index),
        }
    }
}

type RuntimeCallback = Arc<dyn Fn(&RuntimeEvent) + Send + Sync + 'static>;

#[derive(Default, Clone)]
pub struct CallbackManager {
    callbacks: Vec<RuntimeCallback>,
}

impl CallbackManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&RuntimeEvent) + Send + Sync + 'static,
    {
        self.callbacks.push(Arc::new(callback));
        self
    }

    pub fn emit(&self, event: RuntimeEvent) {
        for callback in &self.callbacks {
            callback(&event);
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageTotals {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total: UsageTotals,
    pub by_provider: HashMap<String, UsageTotals>,
    pub by_metric: HashMap<String, UsageTotals>,
}

#[derive(Debug, Default, Clone)]
pub struct UsageTracker {
    summary: UsageSummary,
}

impl UsageTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(
        &mut self,
        provider: impl Into<String>,
        metric_name: impl Into<String>,
        usage: TokenUsage,
    ) {
        let provider = provider.into();
        let metric_name = metric_name.into();
        let totals = UsageTotals::from_token_usage(&usage);
        self.summary.total.add_assign(&totals);
        self.summary
            .by_provider
            .entry(provider)
            .or_default()
            .add_assign(&totals);
        self.summary
            .by_metric
            .entry(metric_name)
            .or_default()
            .add_assign(&totals);
    }

    pub fn summary(&self) -> UsageSummary {
        self.summary.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKey {
    pub namespace: String,
    pub digest: String,
    pub redacted_payload: Value,
}

impl CacheKey {
    pub fn derive(namespace: impl Into<String>, payload: &Value) -> Self {
        let namespace = namespace.into();
        let redacted_payload = redact_value(payload);
        let canonical = canonical_json(&redacted_payload);
        let digest = stable_hash_hex(&format!("{namespace}:{canonical}"));
        Self {
            namespace,
            digest,
            redacted_payload,
        }
    }
}

impl UsageTotals {
    fn from_token_usage(usage: &TokenUsage) -> Self {
        let prompt_tokens = usage.prompt_tokens.unwrap_or_default() as u64;
        let completion_tokens = usage.completion_tokens.unwrap_or_default() as u64;
        let total_tokens = usage
            .total_tokens
            .map(u64::from)
            .unwrap_or(prompt_tokens + completion_tokens);
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens,
        }
    }

    fn add_assign(&mut self, other: &Self) {
        self.prompt_tokens += other.prompt_tokens;
        self.completion_tokens += other.completion_tokens;
        self.total_tokens += other.total_tokens;
    }
}

fn redact_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut redacted = serde_json::Map::new();
            for (key, value) in map {
                if is_secret_key(key) {
                    redacted.insert(key.clone(), Value::String("[redacted]".to_string()));
                } else {
                    redacted.insert(key.clone(), redact_value(value));
                }
            }
            Value::Object(redacted)
        }
        Value::Array(values) => Value::Array(values.iter().map(redact_value).collect()),
        Value::String(value) if looks_like_secret(value) => Value::String("[redacted]".to_string()),
        _ => value.clone(),
    }
}

fn is_secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    ["api_key", "authorization", "password", "secret", "token"]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn looks_like_secret(value: &str) -> bool {
    value.starts_with("sk-") || value.starts_with("Bearer ")
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => serde_json::to_string(value).expect("string serializes"),
        Value::Array(values) => {
            let inner = values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{inner}]")
        }
        Value::Object(map) => {
            let mut entries = map.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(left, _)| *left);
            let inner = entries
                .into_iter()
                .map(|(key, value)| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).expect("key serializes"),
                        canonical_json(value)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{inner}}}")
        }
    }
}

fn stable_hash_hex(input: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
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

    pub fn retry(
        mut self,
        max_attempts: u32,
        initial_backoff_ms: u64,
        max_backoff_ms: u64,
    ) -> Self {
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
        sync::{Arc as StdArc, Mutex as StdMutex},
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
        assert!(matches!(
            report.results[0].outcome,
            ExecutorOutcome::Success(10)
        ));
        assert!(matches!(
            &report.results[1].outcome,
            ExecutorOutcome::Failure(message) if message.contains("provider failed")
        ));
        assert!(matches!(
            report.results[2].outcome,
            ExecutorOutcome::Success(30)
        ));
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

        assert!(
            report.events.iter().any(|event| {
                event.job_name == "ok" && event.kind == ProgressEventKind::Started
            })
        );
        assert!(
            report.events.iter().any(|event| {
                event.job_name == "ok" && event.kind == ProgressEventKind::Succeeded
            })
        );
        assert!(
            report.events.iter().any(|event| {
                event.job_name == "bad" && event.kind == ProgressEventKind::Started
            })
        );
        assert!(
            report.events.iter().any(|event| {
                event.job_name == "bad" && event.kind == ProgressEventKind::Failed
            })
        );
    }

    #[test]
    fn test_6_3_1_callback_hooks_receive_evaluation_lifecycle_events() {
        // SCEN-6.3.1 / AC1 / TEST-6.3.1
        let events = StdArc::new(StdMutex::new(Vec::new()));
        let captured = StdArc::clone(&events);
        let callbacks = CallbackManager::new().with_callback(move |event| {
            captured.lock().expect("events lock").push(event.clone());
        });

        callbacks.emit(RuntimeEvent::evaluation_started("run-1"));
        callbacks.emit(RuntimeEvent::metric_started("run-1", "faithfulness", 0));
        callbacks.emit(RuntimeEvent::metric_succeeded("run-1", "faithfulness", 0));

        let events = events.lock().expect("events lock");
        let kinds = events
            .iter()
            .map(|event| event.kind.clone())
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                RuntimeEventKind::EvaluationStarted,
                RuntimeEventKind::MetricStarted,
                RuntimeEventKind::MetricSucceeded
            ]
        );
        assert_eq!(events[1].metric_name.as_deref(), Some("faithfulness"));
        assert_eq!(events[1].sample_index, Some(0));
    }

    #[test]
    fn test_6_3_2_token_usage_aggregates_per_provider_and_metric() {
        // SCEN-6.3.2 / AC2 / TEST-6.3.2
        let mut tracker = UsageTracker::new();
        tracker.record(
            "openai",
            "faithfulness",
            TokenUsage {
                prompt_tokens: Some(10),
                completion_tokens: Some(5),
                total_tokens: Some(15),
            },
        );
        tracker.record(
            "openai",
            "faithfulness",
            TokenUsage {
                prompt_tokens: Some(3),
                completion_tokens: Some(2),
                total_tokens: Some(5),
            },
        );
        tracker.record(
            "local",
            "context_precision",
            TokenUsage {
                prompt_tokens: Some(4),
                completion_tokens: Some(1),
                total_tokens: None,
            },
        );

        let summary = tracker.summary();

        assert_eq!(summary.total.prompt_tokens, 17);
        assert_eq!(summary.total.completion_tokens, 8);
        assert_eq!(summary.total.total_tokens, 25);
        assert_eq!(summary.by_provider["openai"].total_tokens, 20);
        assert_eq!(summary.by_metric["faithfulness"].completion_tokens, 7);
        assert_eq!(summary.by_metric["context_precision"].total_tokens, 5);
    }

    #[test]
    fn test_6_3_3_cache_key_derivation_is_stable_and_redacts_secrets() {
        // SCEN-6.3.3 / AC3 / TEST-6.3.3
        let left: Value = serde_json::from_str(
            r#"{"api_key":"sk-secret","messages":[{"content":"hello"}],"temperature":0}"#,
        )
        .expect("payload");
        let right: Value = serde_json::from_str(
            r#"{"temperature":0,"messages":[{"content":"hello"}],"api_key":"sk-secret"}"#,
        )
        .expect("payload");

        let first = CacheKey::derive("llm.generate", &left);
        let second = CacheKey::derive("llm.generate", &right);

        assert_eq!(first.digest, second.digest);
        let redacted = first.redacted_payload.to_string();
        assert!(!redacted.contains("sk-secret"));
        assert!(redacted.contains("[redacted]"));
    }

    #[test]
    fn test_18_1_1_cache_key_is_deterministic_and_excludes_callbacks() {
        // SCEN-18.1.1 / AC1 / TEST-18.1.1
        let args = vec![serde_json::json!(["alpha", {"z": 1, "a": [true, false]}])];
        let mut kwargs_a = HashMap::new();
        kwargs_a.insert("threshold".to_string(), serde_json::json!(0.8));
        kwargs_a.insert("callbacks".to_string(), serde_json::json!(["trace-a"]));

        let mut kwargs_b = HashMap::new();
        kwargs_b.insert("callbacks".to_string(), serde_json::json!(["trace-b"]));
        kwargs_b.insert("threshold".to_string(), serde_json::json!(0.8));

        let key_a = generate_runtime_cache_key("Metric.score", &args, &kwargs_a);
        let key_b = generate_runtime_cache_key("Metric.score", &args, &kwargs_b);

        assert_eq!(key_a.function, "Metric.score");
        assert_eq!(key_a.key, key_b.key);
        assert!(!key_a.key.contains("trace-a"));
        assert!(!key_a.key.contains("callbacks"));
    }

    #[test]
    fn test_18_1_2_lazy_tokenizer_initializes_on_first_use() {
        // SCEN-18.1.2 / AC2 / TEST-18.1.2
        let mut tokenizer = LazyTokenizer::new("o200k_base");

        assert_eq!(tokenizer.encoding_name(), "o200k_base");
        assert!(!tokenizer.is_initialized());

        let tokens = tokenizer.encode("hello world");

        assert!(tokenizer.is_initialized());
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokenizer.count_tokens("hello   world"), 2);
    }

    #[test]
    fn test_18_1_3_model_token_usage_addition_and_cost_rules() {
        // SCEN-18.1.3 / AC3 / TEST-18.1.3
        let first = ModelTokenUsage::new(10, 5, "gpt-4o-mini");
        let second = ModelTokenUsage::new(7, 3, "gpt-4o-mini");
        let combined = first.add(&second).expect("same model can be added");

        assert_eq!(combined.input_tokens, 17);
        assert_eq!(combined.output_tokens, 8);
        assert_eq!(combined.cost(0.01, Some(0.02)), 0.33);

        let different = ModelTokenUsage::new(1, 1, "other-model");
        assert!(combined.add(&different).is_err());

        let mut rates = HashMap::new();
        rates.insert("gpt-4o-mini".to_string(), (0.01, 0.02));
        rates.insert("other-model".to_string(), (0.10, 0.20));
        let total = total_model_cost(&[combined, different], &rates).expect("rates present");

        assert_eq!(total, 0.63);
    }
}
