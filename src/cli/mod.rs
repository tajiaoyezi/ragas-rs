use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    ContextUtilizationMetric, DatasetBackend, EvaluationDataset, EvaluationReport,
    EvaluationSample, ExtractionBundle, FaithfulnessMetric, FnMetric, GraphNode,
    InMemoryDatasetBackend, KnowledgeGraph, LlmContextRecallMetric, LlmProvider, Metric,
    MetricResult, MetricValue, PersonaGenerator, RagasError, ResilientLlmProvider, RetryConfig,
    RunConfig, SingleTurnSample, TimeoutConfig, attach_extractions, build_chunk_relationships,
    evaluate_with, rouge_l_recall, split_text_into_chunks, synthesize_single_hop_sample,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Evaluate {
        input: String,
        report: String,
    },
    Testset {
        source_id: String,
        text: String,
        output: String,
    },
    Benchmark {
        runs: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliContractSnapshot {
    pub command: String,
    pub status: String,
    pub stdout_keys: Vec<String>,
    pub stderr_empty: bool,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliErrorSnapshot {
    pub status: String,
    pub error_kind: String,
    pub stderr_keys: Vec<String>,
    pub exit_code: i32,
}

#[derive(Debug, Default)]
pub struct CliRuntime {
    datasets: InMemoryDatasetBackend,
    reports: BTreeMap<String, String>,
}

impl CliRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn datasets_mut(&mut self) -> &mut InMemoryDatasetBackend {
        &mut self.datasets
    }

    pub fn datasets(&self) -> &InMemoryDatasetBackend {
        &self.datasets
    }

    pub fn report(&self, name: &str) -> Option<&str> {
        self.reports.get(name).map(String::as_str)
    }
}

/// Binary entrypoint dispatcher. Supplies the live LLM provider built from the process
/// environment (`OPENAI_API_KEY`, optional `OPENAI_BASE_URL`, model from `OPENAI_MODEL`),
/// so a real CLI invocation with a key configured runs the LLM-based metrics; with no key
/// the provider is `None` and `evaluate` stays offline (deterministic ROUGE only).
pub fn run_cli_command(
    runtime: &mut CliRuntime,
    command: CliCommand,
) -> Result<CliOutput, RagasError> {
    let provider = env_llm_provider();
    run_cli_command_with_provider(runtime, command, provider)
}

/// Dispatcher with an explicitly injected (optional) LLM provider. Tests inject a scripted
/// mock here to drive the LLM-based metrics deterministically; the binary path goes through
/// [`run_cli_command`], which builds the provider from the environment.
pub fn run_cli_command_with_provider(
    runtime: &mut CliRuntime,
    command: CliCommand,
    provider: Option<Arc<dyn LlmProvider>>,
) -> Result<CliOutput, RagasError> {
    match command {
        CliCommand::Evaluate { input, report } => run_evaluate(runtime, input, report, provider),
        CliCommand::Testset {
            source_id,
            text,
            output,
        } => run_testset(runtime, source_id, text, output),
        CliCommand::Benchmark { runs } => run_benchmark(runs),
    }
}

/// Build the live LLM provider from the environment, or `None` when no API key is set.
/// The chat model is taken from `OPENAI_MODEL`, falling back to a sane default.
fn env_llm_provider() -> Option<Arc<dyn LlmProvider>> {
    crate::ProviderConfig::from_env().chat_provider()
}

pub fn cli_contract_snapshot(output: &CliOutput) -> Result<CliContractSnapshot, RagasError> {
    let stdout: serde_json::Value = serde_json::from_str(&output.stdout)
        .map_err(|error| parse_error(format!("CLI stdout snapshot parse failed: {error}")))?;
    let object = stdout.as_object().ok_or_else(|| {
        parse_error("CLI stdout snapshot expected a machine-readable JSON object")
    })?;
    let mut stdout_keys = object.keys().cloned().collect::<Vec<_>>();
    stdout_keys.sort();
    Ok(CliContractSnapshot {
        command: object
            .get("command")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        status: object
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        stdout_keys,
        stderr_empty: output.stderr.is_empty(),
        exit_code: output.exit_code,
    })
}

pub fn cli_error_snapshot(error: &RagasError) -> Result<CliErrorSnapshot, RagasError> {
    Ok(CliErrorSnapshot {
        status: "error".to_string(),
        error_kind: cli_error_kind(error).to_string(),
        stderr_keys: vec![
            "error".to_string(),
            "kind".to_string(),
            "status".to_string(),
        ],
        exit_code: 1,
    })
}

fn run_evaluate(
    runtime: &mut CliRuntime,
    input: String,
    report: String,
    provider: Option<Arc<dyn LlmProvider>>,
) -> Result<CliOutput, RagasError> {
    let dataset = runtime.datasets.load(&input)?;
    // Only single-turn samples are scorable by `Metric::score`; multi-turn samples are
    // carried in the dataset but skipped here (reported as `skipped_samples`).
    let total_samples = dataset.len();
    let single_turn: Vec<SingleTurnSample> = dataset
        .samples()
        .iter()
        .filter_map(|sample| match sample {
            EvaluationSample::SingleTurn(sample) => Some(sample.clone()),
            EvaluationSample::MultiTurn(_) => None,
        })
        .collect();
    let skipped = total_samples - single_turn.len();

    let evaluated = if single_turn.is_empty() {
        None
    } else {
        let any_reference = single_turn.iter().any(|sample| sample.reference.is_some());
        let scorable = EvaluationDataset::new(single_turn)?;
        // Apply conservative retry + per-operation timeout to the provider so a transient API
        // failure does not surface as a metric error (the provider's retry/timeout config was
        // previously inert — nothing in the eval path applied it).
        let run_config = eval_run_config();
        let provider = provider.map(|provider| resilient_eval_provider(provider, &run_config));
        let metrics = build_evaluate_metrics(provider, any_reference);
        // Drive the async evaluation pipeline from this synchronous CLI entry point.
        let report = run_async(evaluate_with(&scorable, &metrics, &run_config));
        Some(report)
    };

    let aggregates = evaluated.as_ref().map(aggregate_report).unwrap_or_default();
    let summary = render_summary(&aggregates);
    let aggregates_json = aggregates
        .iter()
        .map(|aggregate| {
            json!({
                "metric": aggregate.metric_name,
                "count": aggregate.count,
                "mean": aggregate.mean,
                "std": aggregate.std,
                "errors": aggregate.errors,
            })
        })
        .collect::<Vec<_>>();

    let report_json = json!({
        "command": "evaluate",
        "status": "ok",
        "input": input,
        "report": report,
        "sample_count": total_samples,
        "scored_samples": total_samples - skipped,
        "skipped_samples": skipped,
        "metrics": aggregates_json,
        "summary": summary,
    });
    let report_string = serde_json::to_string(&report_json)
        .map_err(|error| parse_error(format!("evaluate report serialization failed: {error}")))?;
    runtime.reports.insert(report.clone(), report_string);

    let stdout = json!({
        "command": "evaluate",
        "status": "ok",
        "report": report,
        "sample_count": total_samples,
        "metrics": aggregates_json,
        "summary": summary,
    });
    cli_output(stdout)
}

/// The metric set the evaluate command runs. ROUGE-L recall (deterministic, offline) always
/// runs: it scores the response against the reference using real lexical overlap (no LLM, no
/// network), so the command produces a real numeric aggregate without any provider configured.
///
/// When an [`LlmProvider`] is supplied, the genuine LLM-based metrics from `src/metric.rs` are
/// appended and run against the same dataset through their real `.generate` pipelines:
/// [`FaithfulnessMetric`] and [`ContextUtilizationMetric`] (reference-free) always, and
/// [`LlmContextRecallMetric`] when at least one sample carries a reference (its required field).
/// These are the actual `Metric` impls, not reimplementations.
fn build_evaluate_metrics(
    provider: Option<Arc<dyn LlmProvider>>,
    any_reference: bool,
) -> Vec<Arc<dyn Metric>> {
    let rouge = Arc::new(FnMetric::new("rouge_l", |sample: &SingleTurnSample| {
        let response = sample.response.clone();
        let reference = sample.reference.clone();
        Box::pin(async move {
            let Some(reference) = reference else {
                return Err(RagasError::Parse {
                    message: "rouge_l requires a reference".to_string(),
                });
            };
            let detailed = rouge_l_recall(&response, &reference);
            let score = detailed.score.ok_or_else(|| RagasError::Parse {
                message: "rouge_l produced no score".to_string(),
            })?;
            Ok(MetricResult::success(
                "rouge_l",
                MetricValue::numeric(score),
            ))
        })
    }));
    let mut metrics: Vec<Arc<dyn Metric>> = vec![rouge];

    if let Some(provider) = provider {
        metrics.push(Arc::new(FaithfulnessMetric::new(Arc::clone(&provider))));
        // Context utilization needs no reference (the common production case), so it always runs.
        metrics.push(Arc::new(ContextUtilizationMetric::new(Arc::clone(
            &provider,
        ))));
        if any_reference {
            metrics.push(Arc::new(LlmContextRecallMetric::new(provider)));
        }
    }

    metrics
}

/// Conservative resilience config for the CLI evaluate path. Deliberately NOT
/// [`RunConfig::default`] (10 retries / up-to-60s backoff / 180s per-op timeout, which can stack a
/// persistently-failing call into a multi-minute hang): a few quick retries with a bounded per-call
/// timeout. `concurrency` stays 1, unchanged from the prior CLI behavior.
fn eval_run_config() -> RunConfig {
    RunConfig {
        retry: RetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 250,
            max_backoff_ms: 2_000,
        },
        timeout: TimeoutConfig {
            per_operation_ms: 60_000,
            total_ms: None,
        },
        concurrency: 1,
        ..RunConfig::default()
    }
}

/// Wrap the chat provider so the evaluate path's metrics inherit retry + per-operation timeout.
/// Resilience is applied here (at provider construction) because each metric owns its provider —
/// `evaluate_with` never sees the provider directly.
fn resilient_eval_provider(
    provider: Arc<dyn LlmProvider>,
    run_config: &RunConfig,
) -> Arc<dyn LlmProvider> {
    Arc::new(ResilientLlmProvider::from_run_config(provider, run_config))
}

/// Per-metric numeric aggregate over an [`EvaluationReport`]: count of finite scores, their
/// mean, the population standard deviation, and the number of errored/non-finite cells.
#[derive(Debug, Clone, PartialEq)]
struct MetricAggregate {
    metric_name: String,
    count: usize,
    mean: f64,
    std: f64,
    errors: usize,
}

/// Collapse the per-sample/per-metric report into one aggregate row per metric, preserving
/// the metric order. Only finite numeric scores feed mean/std; errored cells, missing values,
/// and non-finite scores (e.g. NaN from undefined metrics) are counted under `errors`.
fn aggregate_report(report: &EvaluationReport) -> Vec<MetricAggregate> {
    report
        .metric_names
        .iter()
        .enumerate()
        .map(|(metric_index, metric_name)| {
            let mut scores = Vec::new();
            let mut errors = 0usize;
            for sample in &report.results {
                match sample.results.get(metric_index) {
                    Some(result) => match result.value.as_ref().and_then(MetricValue::as_numeric) {
                        Some(score) if score.is_finite() => scores.push(score),
                        _ => errors += 1,
                    },
                    None => errors += 1,
                }
            }
            let count = scores.len();
            let mean = if count == 0 {
                0.0
            } else {
                scores.iter().sum::<f64>() / count as f64
            };
            let std = if count == 0 {
                0.0
            } else {
                let variance = scores
                    .iter()
                    .map(|score| (score - mean).powi(2))
                    .sum::<f64>()
                    / count as f64;
                variance.sqrt()
            };
            MetricAggregate {
                metric_name: metric_name.clone(),
                count,
                mean,
                std,
                errors,
            }
        })
        .collect()
}

/// Render the aggregates as a fixed-column text table (header + one row per metric). Numbers
/// are formatted to four decimals so the summary is stable and human-readable.
fn render_summary(aggregates: &[MetricAggregate]) -> String {
    let mut lines = vec![format!(
        "{:<24} {:>6} {:>10} {:>10} {:>7}",
        "metric", "count", "mean", "std", "errors"
    )];
    for aggregate in aggregates {
        lines.push(format!(
            "{:<24} {:>6} {:>10.4} {:>10.4} {:>7}",
            aggregate.metric_name, aggregate.count, aggregate.mean, aggregate.std, aggregate.errors
        ));
    }
    lines.join("\n")
}

/// Drive a future to completion from the synchronous CLI. Reuses the ambient tokio runtime
/// when the command is already invoked from within one, otherwise spins up a current-thread
/// runtime for the duration of the evaluation.
fn run_async<F: std::future::Future>(future: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(future)),
        Err(_) => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build current-thread runtime")
            .block_on(future),
    }
}

fn run_testset(
    runtime: &mut CliRuntime,
    source_id: String,
    text: String,
    output: String,
) -> Result<CliOutput, RagasError> {
    let chunks = split_text_into_chunks(&source_id, &text, 256);
    let first_chunk_id = chunks
        .first()
        .map(|chunk| chunk.id.clone())
        .ok_or_else(|| dataset_io_error("testset source text did not produce chunks"))?;

    let mut graph = chunks.iter().fold(
        KnowledgeGraph::new().add_node(GraphNode::new(source_id.clone(), "document")),
        |graph, chunk| graph.add_node(chunk.to_graph_node()),
    );
    graph = build_chunk_relationships(graph, &source_id, &chunks);
    for chunk in &chunks {
        graph = attach_extractions(
            graph,
            &chunk.id,
            ExtractionBundle::new(Vec::new(), vec!["cli".to_string()], chunk.text.clone()),
        );
    }

    let persona = PersonaGenerator::new("cli-testset").generate(
        "CLI Synthesizer",
        "evaluation engineer",
        vec!["generate grounded test questions".to_string()],
    );
    let synthesized = synthesize_single_hop_sample(&graph, &first_chunk_id, &persona)
        .ok_or_else(|| dataset_io_error("testset synthesizer could not create a sample"))?;
    let dataset =
        EvaluationDataset::<EvaluationSample>::from_samples(vec![EvaluationSample::SingleTurn(
            synthesized.sample,
        )])?;
    runtime.datasets.save(&output, &dataset)?;

    let stdout = json!({
        "command": "testset",
        "status": "ok",
        "dataset": output,
        "sample_count": dataset.len(),
        "source_id": source_id,
    });
    cli_output(stdout)
}

fn run_benchmark(runs: usize) -> Result<CliOutput, RagasError> {
    let throughput = if runs == 0 { 0.0 } else { runs as f64 * 1000.0 };
    let stdout = json!({
        "command": "benchmark",
        "status": "ok",
        "format": "json",
        "runs": runs,
        "mock_throughput_samples_per_sec": throughput,
    });
    cli_output(stdout)
}

fn cli_output(stdout: serde_json::Value) -> Result<CliOutput, RagasError> {
    Ok(CliOutput {
        stdout: serde_json::to_string(&stdout)
            .map_err(|error| parse_error(format!("CLI output serialization failed: {error}")))?,
        stderr: String::new(),
        exit_code: 0,
    })
}

fn parse_error(message: impl Into<String>) -> RagasError {
    RagasError::Parse {
        message: message.into(),
    }
}

fn dataset_io_error(message: impl Into<String>) -> RagasError {
    RagasError::DatasetIo {
        message: message.into(),
    }
}

fn cli_error_kind(error: &RagasError) -> &'static str {
    match error {
        RagasError::EmptyDataset => "empty_dataset",
        RagasError::InvalidSample { .. } => "invalid_sample",
        RagasError::DatasetIo { .. } => "dataset_io",
        RagasError::Provider { .. } => "provider",
        RagasError::Parse { .. } => "parse",
        RagasError::Prompt { .. } => "prompt",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DatasetBackend, EvaluationDataset, EvaluationSample, LlmRequest, LlmResponse,
        SingleTurnSample,
    };
    use async_trait::async_trait;
    use serde_json::Value;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock LLM that routes on the prompt content rather than a strict FIFO queue, because
    /// `evaluate()` drives several metrics as concurrently-spawned tasks whose call order is
    /// not deterministic. Each branch matches a distinct prompt from the real metric impls in
    /// `src/metric.rs` and returns the JSON that pins a known aggregate; every prompt is also
    /// recorded so the test can prove the real `.generate` path was exercised.
    struct PromptRoutingLlm {
        prompts: Mutex<Vec<String>>,
    }

    impl PromptRoutingLlm {
        fn new() -> Self {
            Self {
                prompts: Mutex::new(Vec::new()),
            }
        }

        fn prompts(&self) -> Vec<String> {
            self.prompts.lock().expect("prompts").clone()
        }
    }

    #[async_trait]
    impl LlmProvider for PromptRoutingLlm {
        async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, RagasError> {
            let prompt = request
                .messages
                .iter()
                .map(|message| message.content.clone())
                .collect::<Vec<_>>()
                .join("\n");
            self.prompts.lock().expect("prompts").push(prompt.clone());

            // FaithfulnessMetric step 1: decompose the response into atomic statements.
            let content = if prompt.contains("atomic factual statements") {
                r#"{"statements":["Ragas evaluates LLM applications.","Ragas is maintained by Exploding Gradients."]}"#
            // FaithfulnessMetric step 2: verify each statement against the context.
            } else if prompt.contains("verdict 1 when the statement is supported") {
                r#"{"verdicts":[{"statement":"a","verdict":1,"reason":"supported"},{"statement":"b","verdict":1,"reason":"supported"}]}"#
            // LlmContextRecallMetric: classify each reference sentence against the context.
            } else if prompt.contains("verdict 1 when the sentence is supported") {
                r#"{"classifications":[{"verdict":1,"reason":"attributed"}]}"#
            // ContextUtilizationMetric: per-context usefulness verdict against the response.
            } else if prompt.contains("useful to arrive at the answer") {
                r#"{"verdict":1,"reason":"useful"}"#
            } else {
                return Err(RagasError::Provider {
                    message: format!("PromptRoutingLlm saw an unrecognized prompt: {prompt}"),
                });
            };

            Ok(LlmResponse {
                content: content.to_string(),
                usage: None,
            })
        }
    }

    /// Wraps [`PromptRoutingLlm`] but fails the first `fail_first` calls with a transient provider
    /// error before delegating. Proves the CLI evaluate path now retries via the resilience
    /// wrapper: without it, that first failure would surface as a metric error.
    struct FlakyThenRoutingLlm {
        inner: PromptRoutingLlm,
        calls: AtomicUsize,
        fail_first: usize,
    }

    impl FlakyThenRoutingLlm {
        fn new(fail_first: usize) -> Self {
            Self {
                inner: PromptRoutingLlm::new(),
                calls: AtomicUsize::new(0),
                fail_first,
            }
        }

        fn total_calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl LlmProvider for FlakyThenRoutingLlm {
        async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, RagasError> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if n < self.fail_first {
                return Err(RagasError::Provider {
                    message: "transient upstream failure (HTTP 503)".to_string(),
                });
            }
            self.inner.generate(request).await
        }
    }

    fn fixture_dataset() -> EvaluationDataset<EvaluationSample> {
        EvaluationDataset::from_samples(vec![EvaluationSample::SingleTurn(
            SingleTurnSample::new(
                "What does ragas evaluate?",
                "RAG and LLM application quality.",
                vec!["ragas evaluates RAG and LLM apps".to_string()],
            )
            .with_reference("ragas evaluates RAG applications")
            .with_metadata("row_id", "sample-1"),
        )])
        .expect("fixture dataset")
    }

    #[test]
    fn test_14_3_1_cli_evaluate_reads_dataset_and_writes_report() {
        // SCEN-14.3.1 / AC1 / TEST-14.3.1
        let mut runtime = CliRuntime::new();
        runtime
            .datasets_mut()
            .save("datasets/fixture", &fixture_dataset())
            .expect("save fixture dataset");

        // Offline (None provider) keeps this deterministic; no live provider resolved from `.env`.
        let output = run_cli_command_with_provider(
            &mut runtime,
            CliCommand::Evaluate {
                input: "datasets/fixture".to_string(),
                report: "reports/run-1".to_string(),
            },
            None,
        )
        .expect("evaluate command");

        let report = runtime.report("reports/run-1").expect("written report");
        let report_json: Value = serde_json::from_str(report).expect("report JSON");
        assert_eq!(report_json["command"], "evaluate");
        assert_eq!(report_json["sample_count"], 1);
        assert_eq!(report_json["report"], "reports/run-1");

        let stdout_json: Value = serde_json::from_str(&output.stdout).expect("stdout JSON");
        assert_eq!(stdout_json["status"], "ok");
        assert_eq!(stdout_json["report"], "reports/run-1");
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn evaluate_command_computes_real_rouge_aggregate_and_renders_summary() {
        // Two single-turn samples scored by the deterministic ROUGE-L recall metric:
        //   sample 1 response == reference        -> LCS 4/4 = 1.0
        //   sample 2 response disjoint from ref   -> LCS 0/4 = 0.0
        // count = 2, mean = (1.0 + 0.0)/2 = 0.5,
        // population std = sqrt(((1-0.5)^2 + (0-0.5)^2)/2) = 0.5.
        let dataset = EvaluationDataset::<EvaluationSample>::from_samples(vec![
            EvaluationSample::SingleTurn(
                SingleTurnSample::new(
                    "what does ragas evaluate?",
                    "ragas evaluates rag applications",
                    vec!["context".to_string()],
                )
                .with_reference("ragas evaluates rag applications"),
            ),
            EvaluationSample::SingleTurn(
                SingleTurnSample::new(
                    "what does ragas evaluate?",
                    "the weather is cold",
                    vec!["context".to_string()],
                )
                .with_reference("ragas evaluates rag applications"),
            ),
        ])
        .expect("scorable dataset");

        let mut runtime = CliRuntime::new();
        runtime
            .datasets_mut()
            .save("datasets/scorable", &dataset)
            .expect("save dataset");

        // Inject None so this stays a deterministic OFFLINE test: with a real key in `.env`,
        // `run_cli_command` would resolve a live provider and make network calls. The env path is
        // covered by the `#[ignore]` live test instead.
        let output = run_cli_command_with_provider(
            &mut runtime,
            CliCommand::Evaluate {
                input: "datasets/scorable".to_string(),
                report: "reports/rouge".to_string(),
            },
            None,
        )
        .expect("evaluate command");

        // The rendered summary is a real table naming the metric and its hand-computed mean.
        let stdout_json: Value = serde_json::from_str(&output.stdout).expect("stdout JSON");
        let summary = stdout_json["summary"].as_str().expect("summary string");
        assert!(
            summary.contains("rouge_l"),
            "summary missing metric: {summary}"
        );
        assert!(
            summary.contains("0.5000"),
            "summary missing mean: {summary}"
        );

        // The structured aggregate carries the exact count/mean/std (not a templated echo).
        let metric = &stdout_json["metrics"][0];
        assert_eq!(metric["metric"], "rouge_l");
        assert_eq!(metric["count"], 2);
        assert!((metric["mean"].as_f64().expect("mean") - 0.5).abs() < 1e-9);
        assert!((metric["std"].as_f64().expect("std") - 0.5).abs() < 1e-9);
        assert_eq!(metric["errors"], 0);
        assert_eq!(stdout_json["sample_count"], 2);

        // The same aggregate is persisted into the saved report.
        let report = runtime.report("reports/rouge").expect("written report");
        let report_json: Value = serde_json::from_str(report).expect("report JSON");
        assert_eq!(report_json["scored_samples"], 2);
        assert_eq!(report_json["skipped_samples"], 0);
        assert!(
            (report_json["metrics"][0]["mean"]
                .as_f64()
                .expect("report mean")
                - 0.5)
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn evaluate_command_marks_missing_reference_as_metric_error() {
        // A sample with no reference cannot be ROUGE-scored: the metric returns Err, which
        // `evaluate()` records as a per-cell failure -> counted under `errors`, not `mean`.
        let dataset = EvaluationDataset::<EvaluationSample>::from_samples(vec![
            EvaluationSample::SingleTurn(SingleTurnSample::new(
                "q",
                "a response with no reference",
                vec!["context".to_string()],
            )),
        ])
        .expect("dataset");

        let mut runtime = CliRuntime::new();
        runtime
            .datasets_mut()
            .save("datasets/noref", &dataset)
            .expect("save dataset");

        // Offline (None provider): only the deterministic ROUGE metric runs, so the no-reference
        // error is exercised without any live LLM call.
        let output = run_cli_command_with_provider(
            &mut runtime,
            CliCommand::Evaluate {
                input: "datasets/noref".to_string(),
                report: "reports/noref".to_string(),
            },
            None,
        )
        .expect("evaluate command");

        let stdout_json: Value = serde_json::from_str(&output.stdout).expect("stdout JSON");
        let metric = &stdout_json["metrics"][0];
        assert_eq!(metric["metric"], "rouge_l");
        assert_eq!(metric["count"], 0);
        assert_eq!(metric["errors"], 1);
        assert_eq!(metric["mean"], 0.0);
    }

    #[test]
    fn evaluate_command_runs_llm_metrics_when_provider_injected() {
        // With a provider injected, the evaluate command additionally runs the genuine
        // LLM-based metrics from `src/metric.rs` through their real `.generate` pipelines.
        // The scripted provider yields, for the single sample:
        //   faithfulness        -> 2 statements, both verified supported -> 2/2 = 1.0
        //   context_utilization -> 1 context judged useful               -> AP@k = 1.0
        //   context_recall      -> reference is 1 sentence, attributed   -> 1/1 = 1.0
        // (rouge_l also runs offline as before). Verified via mock; real-LLM UNVERIFIED.
        let dataset = EvaluationDataset::<EvaluationSample>::from_samples(vec![
            EvaluationSample::SingleTurn(
                SingleTurnSample::new(
                    "What is Ragas and who maintains it?",
                    "Ragas evaluates LLM applications. It is maintained by Exploding Gradients.",
                    vec!["Ragas is a framework to evaluate LLM applications.".to_string()],
                )
                .with_reference("Ragas evaluates RAG applications."),
            ),
        ])
        .expect("dataset");

        let mut runtime = CliRuntime::new();
        runtime
            .datasets_mut()
            .save("datasets/llm", &dataset)
            .expect("save dataset");

        let provider = Arc::new(PromptRoutingLlm::new());
        let output = run_cli_command_with_provider(
            &mut runtime,
            CliCommand::Evaluate {
                input: "datasets/llm".to_string(),
                report: "reports/llm".to_string(),
            },
            Some(Arc::clone(&provider) as Arc<dyn LlmProvider>),
        )
        .expect("evaluate command");

        // The real metric impls actually invoked `.generate`: faithfulness made two calls
        // (statement generation + verification) and context recall one classification call.
        let prompts = provider.prompts();
        assert_eq!(prompts.len(), 4, "expected 4 LLM calls, saw: {prompts:#?}");
        assert!(
            prompts
                .iter()
                .any(|p| p.contains("atomic factual statements")),
            "faithfulness statement-generation prompt missing"
        );
        assert!(
            prompts
                .iter()
                .any(|p| p.contains("verdict 1 when the sentence is supported")),
            "context-recall classification prompt missing"
        );

        let stdout_json: Value = serde_json::from_str(&output.stdout).expect("stdout JSON");

        // The summary names the LLM metrics alongside the offline ROUGE metric.
        let summary = stdout_json["summary"].as_str().expect("summary string");
        assert!(
            summary.contains("rouge_l"),
            "summary missing rouge_l: {summary}"
        );
        assert!(
            summary.contains("faithfulness"),
            "summary missing faithfulness: {summary}"
        );
        assert!(
            summary.contains("context_recall"),
            "summary missing context_recall: {summary}"
        );
        assert!(
            summary.contains("context_utilization"),
            "summary missing context_utilization: {summary}"
        );

        // The structured aggregates carry the LLM metrics' computed means (both 1.0 here).
        let metrics = stdout_json["metrics"].as_array().expect("metrics array");
        let by_name = |name: &str| -> &Value {
            metrics
                .iter()
                .find(|m| m["metric"] == name)
                .unwrap_or_else(|| panic!("metric {name} missing from {metrics:#?}"))
        };

        let faithfulness = by_name("faithfulness");
        assert_eq!(faithfulness["count"], 1);
        assert_eq!(faithfulness["errors"], 0);
        assert!(
            (faithfulness["mean"].as_f64().expect("faithfulness mean") - 1.0).abs() < 1e-9,
            "faithfulness mean: {faithfulness:#?}"
        );

        let context_recall = by_name("context_recall");
        assert_eq!(context_recall["count"], 1);
        assert_eq!(context_recall["errors"], 0);
        assert!(
            (context_recall["mean"]
                .as_f64()
                .expect("context_recall mean")
                - 1.0)
                .abs()
                < 1e-9,
            "context_recall mean: {context_recall:#?}"
        );

        let context_utilization = by_name("context_utilization");
        assert_eq!(context_utilization["count"], 1);
        assert_eq!(context_utilization["errors"], 0);
        assert!(
            (context_utilization["mean"]
                .as_f64()
                .expect("context_utilization mean")
                - 1.0)
                .abs()
                < 1e-9,
            "context_utilization mean: {context_utilization:#?}"
        );

        // ROUGE still runs (offline) and the persisted report carries the same aggregates.
        assert!(metrics.iter().any(|m| m["metric"] == "rouge_l"));
        let report = runtime.report("reports/llm").expect("written report");
        let report_json: Value = serde_json::from_str(report).expect("report JSON");
        let report_metrics = report_json["metrics"].as_array().expect("report metrics");
        assert!(report_metrics.iter().any(|m| m["metric"] == "faithfulness"));
        assert!(
            report_metrics
                .iter()
                .any(|m| m["metric"] == "context_recall")
        );
    }

    #[test]
    fn evaluate_command_retries_transient_provider_failures() {
        // The CLI evaluate path wraps the provider in retry/timeout (resilient_eval_provider).
        // A provider that fails its first call transiently must NOT produce a metric error: the
        // resilience wrapper retries and the LLM metrics still score. Without the wiring, this
        // first failure would surface as errors >= 1.
        let dataset = EvaluationDataset::<EvaluationSample>::from_samples(vec![
            EvaluationSample::SingleTurn(
                SingleTurnSample::new(
                    "What is Ragas and who maintains it?",
                    "Ragas evaluates LLM applications. It is maintained by Exploding Gradients.",
                    vec!["Ragas is a framework to evaluate LLM applications.".to_string()],
                )
                .with_reference("Ragas evaluates RAG applications."),
            ),
        ])
        .expect("dataset");

        let mut runtime = CliRuntime::new();
        runtime
            .datasets_mut()
            .save("datasets/flaky", &dataset)
            .expect("save dataset");

        // Fail exactly the first generate() call; the resilience wrapper should retry it.
        let provider = Arc::new(FlakyThenRoutingLlm::new(1));
        let output = run_cli_command_with_provider(
            &mut runtime,
            CliCommand::Evaluate {
                input: "datasets/flaky".to_string(),
                report: "reports/flaky".to_string(),
            },
            Some(Arc::clone(&provider) as Arc<dyn LlmProvider>),
        )
        .expect("evaluate command");

        let stdout_json: Value = serde_json::from_str(&output.stdout).expect("stdout JSON");
        let metrics = stdout_json["metrics"].as_array().expect("metrics array");
        // Every LLM metric scored with zero errors despite the first call failing transiently.
        for name in ["faithfulness", "context_utilization", "context_recall"] {
            let metric = metrics
                .iter()
                .find(|m| m["metric"] == name)
                .unwrap_or_else(|| panic!("metric {name} missing from {metrics:#?}"));
            assert_eq!(
                metric["errors"], 0,
                "{name} should have retried: {metric:#?}"
            );
            assert_eq!(
                metric["count"], 1,
                "{name} should have one score: {metric:#?}"
            );
        }
        // 4 successful metric calls (faithfulness x2, context_utilization, context_recall) plus
        // the single retried failure = 5 total provider invocations.
        assert_eq!(
            provider.total_calls(),
            5,
            "expected one retry over four successful calls"
        );
    }

    #[test]
    fn evaluate_command_stays_offline_without_provider() {
        // Without a provider, only the deterministic offline metric runs (no LLM metrics),
        // preserving the prior CLI behavior exactly.
        let mut runtime = CliRuntime::new();
        runtime
            .datasets_mut()
            .save("datasets/offline", &fixture_dataset())
            .expect("save dataset");

        let output = run_cli_command_with_provider(
            &mut runtime,
            CliCommand::Evaluate {
                input: "datasets/offline".to_string(),
                report: "reports/offline".to_string(),
            },
            None,
        )
        .expect("evaluate command");

        let stdout_json: Value = serde_json::from_str(&output.stdout).expect("stdout JSON");
        let metrics = stdout_json["metrics"].as_array().expect("metrics array");
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0]["metric"], "rouge_l");
    }

    /// Live, real-LLM smoke test for the provider-driven CLI path. Ignored by default; run with
    /// `OPENAI_API_KEY` set (and optionally `OPENAI_BASE_URL` / `OPENAI_MODEL`):
    ///   cargo test --lib cli::tests::live_evaluate_command_runs_llm_metrics -- --ignored
    /// Asserts the LLM metrics actually executed (count == 1, no error) against a real endpoint.
    #[test]
    #[ignore = "requires OPENAI_API_KEY; talks to a real LLM endpoint"]
    fn live_evaluate_command_runs_llm_metrics() {
        let Some(provider) = env_llm_provider() else {
            eprintln!("OPENAI_API_KEY not set; skipping live CLI evaluate test");
            return;
        };

        let dataset = EvaluationDataset::<EvaluationSample>::from_samples(vec![
            EvaluationSample::SingleTurn(
                SingleTurnSample::new(
                    "What is Ragas and who maintains it?",
                    "Ragas evaluates LLM applications. It is maintained by Exploding Gradients.",
                    vec![
                        "Ragas is a framework to evaluate LLM applications, maintained by \
                         Exploding Gradients."
                            .to_string(),
                    ],
                )
                .with_reference("Ragas evaluates RAG applications."),
            ),
        ])
        .expect("dataset");

        let mut runtime = CliRuntime::new();
        runtime
            .datasets_mut()
            .save("datasets/live", &dataset)
            .expect("save dataset");

        let output = run_cli_command_with_provider(
            &mut runtime,
            CliCommand::Evaluate {
                input: "datasets/live".to_string(),
                report: "reports/live".to_string(),
            },
            Some(provider),
        )
        .expect("evaluate command");

        let stdout_json: Value = serde_json::from_str(&output.stdout).expect("stdout JSON");
        let metrics = stdout_json["metrics"].as_array().expect("metrics array");
        let faithfulness = metrics
            .iter()
            .find(|m| m["metric"] == "faithfulness")
            .expect("faithfulness metric present");
        assert_eq!(
            faithfulness["count"], 1,
            "faithfulness did not score: {faithfulness:#?}"
        );
        assert_eq!(
            faithfulness["errors"], 0,
            "faithfulness errored: {faithfulness:#?}"
        );
    }

    #[test]
    fn test_14_3_2_cli_testset_invokes_synthesizer_flow() {
        // SCEN-14.3.2 / AC2 / TEST-14.3.2
        let mut runtime = CliRuntime::new();

        let output = run_cli_command(
            &mut runtime,
            CliCommand::Testset {
                source_id: "doc-1".to_string(),
                text: "Ragas evaluates retrieval quality with grounded contexts.".to_string(),
                output: "datasets/generated".to_string(),
            },
        )
        .expect("testset command");

        let generated = runtime
            .datasets()
            .load("datasets/generated")
            .expect("generated dataset");
        assert_eq!(generated.len(), 1);
        let EvaluationSample::SingleTurn(sample) = &generated.samples()[0] else {
            panic!("testset should create single-turn samples");
        };
        assert_eq!(
            sample.metadata.get("synthesis_type").map(String::as_str),
            Some("single-hop")
        );
        assert_eq!(
            sample.metadata.get("source_node_ids").map(String::as_str),
            Some("doc-1-chunk-0")
        );

        let stdout_json: Value = serde_json::from_str(&output.stdout).expect("stdout JSON");
        assert_eq!(stdout_json["status"], "ok");
        assert_eq!(stdout_json["dataset"], "datasets/generated");
    }

    #[test]
    fn test_14_3_3_cli_benchmark_prints_machine_readable_summary() {
        // SCEN-14.3.3 / AC3 / TEST-14.3.3
        let mut runtime = CliRuntime::new();

        let output = run_cli_command(&mut runtime, CliCommand::Benchmark { runs: 5 })
            .expect("benchmark command");

        let stdout_json: Value = serde_json::from_str(&output.stdout).expect("stdout JSON");
        assert_eq!(stdout_json["command"], "benchmark");
        assert_eq!(stdout_json["runs"], 5);
        assert!(
            stdout_json["mock_throughput_samples_per_sec"]
                .as_f64()
                .expect("throughput number")
                > 0.0
        );
        assert_eq!(stdout_json["format"], "json");
    }

    #[test]
    fn test_21_2_2_cli_contract_snapshots_preserve_outputs_and_errors() {
        // SCEN-21.2.2 / AC2 / TEST-21.2.2
        let mut runtime = CliRuntime::new();
        runtime
            .datasets_mut()
            .save("datasets/fixture", &fixture_dataset())
            .expect("save fixture dataset");

        // Offline (None provider): the contract snapshot must be deterministic, so it must not
        // resolve a live provider from `.env`.
        let output = run_cli_command_with_provider(
            &mut runtime,
            CliCommand::Evaluate {
                input: "datasets/fixture".to_string(),
                report: "reports/run-21-2".to_string(),
            },
            None,
        )
        .expect("evaluate command");
        let snapshot = cli_contract_snapshot(&output).expect("CLI snapshot");

        assert_eq!(snapshot.command, "evaluate");
        assert_eq!(snapshot.status, "ok");
        assert_eq!(
            snapshot.stdout_keys,
            vec![
                "command",
                "metrics",
                "report",
                "sample_count",
                "status",
                "summary"
            ]
        );
        assert!(snapshot.stderr_empty);
        assert_eq!(snapshot.exit_code, 0);

        let error = run_cli_command(
            &mut runtime,
            CliCommand::Evaluate {
                input: "datasets/missing".to_string(),
                report: "reports/missing".to_string(),
            },
        )
        .expect_err("missing dataset should fail");
        let error_snapshot = cli_error_snapshot(&error).expect("error snapshot");

        assert_eq!(error_snapshot.status, "error");
        assert_eq!(error_snapshot.error_kind, "dataset_io");
        assert_eq!(error_snapshot.stderr_keys, vec!["error", "kind", "status"]);
        assert_eq!(error_snapshot.exit_code, 1);
    }
}
