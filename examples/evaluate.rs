use std::sync::Arc;

use ragas::{
    EvaluationDataset, EvaluationOptions, FnMetric, LlmContextRecallMetric, Metric, MetricResult,
    MetricValue, MockEmbeddingProvider, MockLlmProvider, ResponseRelevancyMetric, SingleTurnSample,
    evaluate,
};

/// Offline demonstration that the real LLM metrics are wired into `evaluate()`.
///
/// It runs against mock providers so it compiles and runs without a network call or
/// API key. The metrics themselves are the genuine multi-step pipelines from
/// `src/metric.rs`; only the provider responses are scripted here. For a real signal,
/// run the `#[ignore]`d live discrimination tests with `OPENAI_API_KEY` set.
#[tokio::main]
async fn main() {
    let dataset = EvaluationDataset::new(vec![
        SingleTurnSample::new(
            "What does ragas-rs evaluate?",
            "ragas-rs evaluates RAG and LLM application quality.",
            vec!["ragas-rs evaluates RAG and LLM applications.".to_string()],
        )
        .with_reference("ragas-rs evaluates RAG quality. It scores LLM applications."),
    ])
    .expect("example dataset");

    // A plain custom metric, to show user metrics and library metrics run side by side.
    let response_length = Arc::new(FnMetric::new("response_length", |sample| {
        let score = sample.response.len() as f64;
        Box::pin(async move {
            Ok(MetricResult::success(
                "response_length",
                MetricValue::numeric(score),
            ))
        })
    }));

    // ResponseRelevancy (default strictness 3): the mock LLM reverse-engineers three
    // questions, then the mock embedding returns one vector per (user_input + 3 questions).
    let relevancy = Arc::new(ResponseRelevancyMetric::new(
        Arc::new(MockLlmProvider::new(
            r#"{"questions":[
                {"question":"What does ragas-rs evaluate?","noncommittal":0},
                {"question":"What kind of quality does ragas-rs measure?","noncommittal":0},
                {"question":"What applications does ragas-rs score?","noncommittal":0}
            ]}"#,
        )),
        Arc::new(MockEmbeddingProvider::new(vec![
            vec![1.0, 0.0, 0.0],
            vec![0.96, 0.20, 0.0],
            vec![0.92, 0.30, 0.10],
            vec![0.90, 0.10, 0.30],
        ])),
    ));

    // LLMContextRecall: the reference splits into two sentences, so the mock returns two
    // ordered classifications (both attributed -> recall 1.0).
    let context_recall = Arc::new(LlmContextRecallMetric::new(Arc::new(MockLlmProvider::new(
        r#"{"classifications":[{"verdict":1,"reason":"in context"},{"verdict":1,"reason":"in context"}]}"#,
    ))));

    let metrics: Vec<Arc<dyn Metric>> = vec![response_length, relevancy, context_recall];

    let report = evaluate(&dataset, &metrics, EvaluationOptions { concurrency: 1 }).await;

    println!(
        "{} sample(s), metrics: {}",
        report.results.len(),
        report.metric_names.join(", ")
    );
    for sample in &report.results {
        for result in &sample.results {
            match (
                result.value.as_ref().and_then(MetricValue::as_numeric),
                &result.error,
            ) {
                (Some(score), _) => println!("  {} = {score}", result.metric_name),
                (None, Some(error)) => println!("  {} errored: {error}", result.metric_name),
                (None, None) => println!("  {} = <none>", result.metric_name),
            }
        }
    }
}
