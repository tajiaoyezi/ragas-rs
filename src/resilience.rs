//! Resilient, caching, and usage-recording provider decorators.
//!
//! [`RunConfig`]'s `retry` and `timeout` were previously dead config — nothing in the
//! evaluation path (`evaluate` / `AsyncExecutor`) applied them, and no layer cached provider
//! responses. These composable wrappers fix that: wrap any [`LlmProvider`] / [`EmbeddingProvider`]
//! to add retry-with-exponential-backoff + a per-operation timeout, and/or an in-memory response
//! cache. They are opt-in and compose, e.g.:
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use ragas::{LlmProvider, ResilientLlmProvider, CachingLlmProvider, RunConfig};
//! # fn wrap(base: Arc<dyn LlmProvider>) -> Arc<dyn LlmProvider> {
//! let config = RunConfig::default();
//! Arc::new(CachingLlmProvider::new(Arc::new(
//!     ResilientLlmProvider::from_run_config(base, &config),
//! )))
//! # }
//! ```

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio::time::{Duration, sleep, timeout};

use crate::{
    EmbeddingProvider, EmbeddingRequest, EmbeddingResponse, LlmProvider, LlmRequest, LlmResponse,
    RagasError, RetryConfig, RunConfig, TimeoutConfig, UsageTracker,
};

/// Run `make_future` with retry (exponential backoff) and a per-operation timeout. Each attempt
/// gets a fresh future from `make_future`; a timed-out or failed attempt is retried until
/// `retry.max_attempts` is reached, after which the last error is returned.
async fn run_with_resilience<T, Fut, MakeFut>(
    retry: &RetryConfig,
    timeout_config: &TimeoutConfig,
    operation: &str,
    mut make_future: MakeFut,
) -> Result<T, RagasError>
where
    MakeFut: FnMut() -> Fut,
    Fut: Future<Output = Result<T, RagasError>>,
{
    let max_attempts = retry.max_attempts.max(1);
    let mut backoff_ms = retry.initial_backoff_ms.max(1);
    let mut last_error: Option<RagasError> = None;

    for attempt in 1..=max_attempts {
        let future = make_future();
        let outcome = if timeout_config.per_operation_ms > 0 {
            match timeout(
                Duration::from_millis(timeout_config.per_operation_ms),
                future,
            )
            .await
            {
                Ok(result) => result,
                Err(_) => Err(RagasError::Provider {
                    message: format!(
                        "{operation} timed out after {} ms",
                        timeout_config.per_operation_ms
                    ),
                }),
            }
        } else {
            future.await
        };

        match outcome {
            Ok(value) => return Ok(value),
            Err(error) => {
                last_error = Some(error);
                if attempt < max_attempts {
                    let capped = backoff_ms.min(retry.max_backoff_ms.max(1));
                    sleep(Duration::from_millis(capped)).await;
                    backoff_ms = backoff_ms.saturating_mul(2);
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| RagasError::Provider {
        message: format!("{operation} failed without producing an error"),
    }))
}

/// An [`LlmProvider`] that retries (exponential backoff) and times out the wrapped provider.
pub struct ResilientLlmProvider {
    inner: Arc<dyn LlmProvider>,
    retry: RetryConfig,
    timeout: TimeoutConfig,
}

impl ResilientLlmProvider {
    pub fn new(inner: Arc<dyn LlmProvider>) -> Self {
        Self {
            inner,
            retry: RetryConfig::default(),
            timeout: TimeoutConfig::default(),
        }
    }

    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }

    pub fn with_timeout(mut self, timeout: TimeoutConfig) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build directly from a [`RunConfig`], adopting its `retry` + `timeout`.
    pub fn from_run_config(inner: Arc<dyn LlmProvider>, config: &RunConfig) -> Self {
        Self {
            inner,
            retry: config.retry.clone(),
            timeout: config.timeout.clone(),
        }
    }
}

#[async_trait]
impl LlmProvider for ResilientLlmProvider {
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, RagasError> {
        run_with_resilience(&self.retry, &self.timeout, "llm.generate", || {
            let inner = Arc::clone(&self.inner);
            let request = request.clone();
            async move { inner.generate(request).await }
        })
        .await
    }
}

/// An [`LlmProvider`] that records each successful response's token usage into a shared
/// [`UsageTracker`], attributed to a `(provider, metric)` label. A response carrying no `usage` is
/// passed through unrecorded. Transparent otherwise — compose it over the base (or resilient)
/// provider so the recorded usage reflects the calls actually made.
pub struct UsageRecordingLlmProvider {
    inner: Arc<dyn LlmProvider>,
    tracker: Arc<Mutex<UsageTracker>>,
    provider_label: String,
    metric_label: String,
}

impl UsageRecordingLlmProvider {
    pub fn new(
        inner: Arc<dyn LlmProvider>,
        tracker: Arc<Mutex<UsageTracker>>,
        provider_label: impl Into<String>,
        metric_label: impl Into<String>,
    ) -> Self {
        Self {
            inner,
            tracker,
            provider_label: provider_label.into(),
            metric_label: metric_label.into(),
        }
    }
}

#[async_trait]
impl LlmProvider for UsageRecordingLlmProvider {
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, RagasError> {
        let response = self.inner.generate(request).await?;
        if let Some(usage) = response.usage.clone()
            && let Ok(mut tracker) = self.tracker.lock()
        {
            tracker.record(&self.provider_label, &self.metric_label, usage);
        }
        Ok(response)
    }
}

/// An [`EmbeddingProvider`] that retries (exponential backoff) and times out the wrapped provider.
pub struct ResilientEmbeddingProvider {
    inner: Arc<dyn EmbeddingProvider>,
    retry: RetryConfig,
    timeout: TimeoutConfig,
}

impl ResilientEmbeddingProvider {
    pub fn new(inner: Arc<dyn EmbeddingProvider>) -> Self {
        Self {
            inner,
            retry: RetryConfig::default(),
            timeout: TimeoutConfig::default(),
        }
    }

    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }

    pub fn with_timeout(mut self, timeout: TimeoutConfig) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn from_run_config(inner: Arc<dyn EmbeddingProvider>, config: &RunConfig) -> Self {
        Self {
            inner,
            retry: config.retry.clone(),
            timeout: config.timeout.clone(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for ResilientEmbeddingProvider {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
        run_with_resilience(&self.retry, &self.timeout, "embedding.embed", || {
            let inner = Arc::clone(&self.inner);
            let request = request.clone();
            async move { inner.embed(request).await }
        })
        .await
    }
}

/// An [`LlmProvider`] that memoizes successful responses in-memory, keyed on the request.
/// Repeated identical requests are served from the cache without calling the inner provider.
pub struct CachingLlmProvider {
    inner: Arc<dyn LlmProvider>,
    cache: Mutex<HashMap<String, LlmResponse>>,
}

impl CachingLlmProvider {
    pub fn new(inner: Arc<dyn LlmProvider>) -> Self {
        Self {
            inner,
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.cache.lock().expect("cache lock").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl LlmProvider for CachingLlmProvider {
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, RagasError> {
        let key = serde_json::to_string(&request).unwrap_or_default();
        {
            let cache = self.cache.lock().expect("cache lock");
            if let Some(hit) = cache.get(&key) {
                return Ok(hit.clone());
            }
        }
        let response = self.inner.generate(request).await?;
        self.cache
            .lock()
            .expect("cache lock")
            .insert(key, response.clone());
        Ok(response)
    }
}

/// An [`EmbeddingProvider`] that memoizes successful responses in-memory, keyed on the request.
pub struct CachingEmbeddingProvider {
    inner: Arc<dyn EmbeddingProvider>,
    cache: Mutex<HashMap<String, EmbeddingResponse>>,
}

impl CachingEmbeddingProvider {
    pub fn new(inner: Arc<dyn EmbeddingProvider>) -> Self {
        Self {
            inner,
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.cache.lock().expect("cache lock").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl EmbeddingProvider for CachingEmbeddingProvider {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RagasError> {
        let key = serde_json::to_string(&request).unwrap_or_default();
        {
            let cache = self.cache.lock().expect("cache lock");
            if let Some(hit) = cache.get(&key) {
                return Ok(hit.clone());
            }
        }
        let response = self.inner.embed(request).await?;
        self.cache
            .lock()
            .expect("cache lock")
            .insert(key, response.clone());
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChatMessage;

    fn request() -> LlmRequest {
        LlmRequest {
            messages: vec![ChatMessage::user("hello")],
            temperature: Some(0.0),
        }
    }

    fn fast_retry(max_attempts: u32) -> RetryConfig {
        RetryConfig {
            max_attempts,
            initial_backoff_ms: 1,
            max_backoff_ms: 1,
        }
    }

    /// Fails its first `failures` calls, then succeeds; records the total call count.
    struct FlakyLlm {
        remaining_failures: Mutex<u32>,
        calls: Mutex<u32>,
    }

    impl FlakyLlm {
        fn new(failures: u32) -> Self {
            Self {
                remaining_failures: Mutex::new(failures),
                calls: Mutex::new(0),
            }
        }

        fn calls(&self) -> u32 {
            *self.calls.lock().expect("calls")
        }
    }

    #[async_trait]
    impl LlmProvider for FlakyLlm {
        async fn generate(&self, _request: LlmRequest) -> Result<LlmResponse, RagasError> {
            *self.calls.lock().expect("calls") += 1;
            let mut remaining = self.remaining_failures.lock().expect("failures");
            if *remaining > 0 {
                *remaining -= 1;
                return Err(RagasError::Provider {
                    message: "flaky provider transient error".to_string(),
                });
            }
            Ok(LlmResponse {
                content: "ok".to_string(),
                usage: None,
            })
        }
    }

    /// Records how many times it is called; always succeeds with the same content.
    struct CountingLlm {
        calls: Mutex<u32>,
    }

    impl CountingLlm {
        fn new() -> Self {
            Self {
                calls: Mutex::new(0),
            }
        }

        fn calls(&self) -> u32 {
            *self.calls.lock().expect("calls")
        }
    }

    #[async_trait]
    impl LlmProvider for CountingLlm {
        async fn generate(&self, _request: LlmRequest) -> Result<LlmResponse, RagasError> {
            *self.calls.lock().expect("calls") += 1;
            Ok(LlmResponse {
                content: "response".to_string(),
                usage: None,
            })
        }
    }

    /// Hangs far longer than any test timeout, so the timeout path can be exercised.
    struct HangingLlm;

    #[async_trait]
    impl LlmProvider for HangingLlm {
        async fn generate(&self, _request: LlmRequest) -> Result<LlmResponse, RagasError> {
            sleep(Duration::from_secs(30)).await;
            Ok(LlmResponse {
                content: "never".to_string(),
                usage: None,
            })
        }
    }

    #[tokio::test]
    async fn resilient_provider_retries_until_success() {
        let flaky = Arc::new(FlakyLlm::new(2)); // fails twice, succeeds on the third call
        let provider = ResilientLlmProvider::new(flaky.clone())
            .with_retry(fast_retry(3))
            .with_timeout(TimeoutConfig {
                per_operation_ms: 1_000,
                total_ms: None,
            });

        let response = provider
            .generate(request())
            .await
            .expect("eventually succeeds");
        assert_eq!(response.content, "ok");
        assert_eq!(flaky.calls(), 3);
    }

    #[tokio::test]
    async fn resilient_provider_surfaces_error_after_exhausting_retries() {
        let flaky = Arc::new(FlakyLlm::new(10)); // always fails within the attempt budget
        let provider = ResilientLlmProvider::new(flaky.clone()).with_retry(fast_retry(2));

        let error = provider
            .generate(request())
            .await
            .expect_err("exhausts retries");
        assert!(error.to_string().contains("flaky"));
        assert_eq!(flaky.calls(), 2);
    }

    #[tokio::test]
    async fn resilient_provider_times_out_a_hanging_call() {
        let provider = ResilientLlmProvider::new(Arc::new(HangingLlm))
            .with_retry(fast_retry(1))
            .with_timeout(TimeoutConfig {
                per_operation_ms: 20,
                total_ms: None,
            });

        let error = provider.generate(request()).await.expect_err("times out");
        assert!(error.to_string().contains("timed out"), "{error}");
    }

    #[tokio::test]
    async fn caching_provider_serves_repeat_requests_from_cache() {
        let counting = Arc::new(CountingLlm::new());
        let provider = CachingLlmProvider::new(counting.clone());

        let first = provider.generate(request()).await.expect("first");
        let second = provider.generate(request()).await.expect("second");
        assert_eq!(first.content, second.content);
        assert_eq!(
            counting.calls(),
            1,
            "second identical request must hit the cache"
        );
        assert_eq!(provider.len(), 1);

        // A different request misses the cache and reaches the inner provider.
        let other = LlmRequest {
            messages: vec![ChatMessage::user("different")],
            temperature: Some(0.0),
        };
        provider.generate(other).await.expect("other");
        assert_eq!(counting.calls(), 2);
        assert_eq!(provider.len(), 2);
    }

    #[tokio::test]
    async fn usage_recording_provider_aggregates_token_usage_by_label() {
        use crate::TokenUsage;

        struct FixedUsageLlm;
        #[async_trait]
        impl LlmProvider for FixedUsageLlm {
            async fn generate(&self, _request: LlmRequest) -> Result<LlmResponse, RagasError> {
                Ok(LlmResponse {
                    content: "ok".to_string(),
                    usage: Some(TokenUsage {
                        prompt_tokens: Some(10),
                        completion_tokens: Some(5),
                        total_tokens: Some(15),
                    }),
                })
            }
        }

        let tracker = Arc::new(Mutex::new(UsageTracker::new()));
        let provider = UsageRecordingLlmProvider::new(
            Arc::new(FixedUsageLlm),
            Arc::clone(&tracker),
            "chat",
            "faithfulness",
        );
        provider.generate(request()).await.expect("first");
        provider.generate(request()).await.expect("second");

        let summary = tracker.lock().expect("tracker").summary();
        assert_eq!(summary.total.total_tokens, 30);
        assert_eq!(summary.total.prompt_tokens, 20);
        assert_eq!(summary.total.completion_tokens, 10);
        assert_eq!(summary.by_metric["faithfulness"].total_tokens, 30);
        assert_eq!(summary.by_provider["chat"].total_tokens, 30);
    }
}
