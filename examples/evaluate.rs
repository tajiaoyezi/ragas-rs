use std::sync::Arc;

use ragas::{
    EvaluationDataset, EvaluationOptions, FnMetric, MetricResult, MetricValue, SingleTurnSample,
    evaluate,
};

#[tokio::main]
async fn main() {
    let dataset = EvaluationDataset::new(vec![SingleTurnSample::new(
        "What does ragas-rs evaluate?",
        "RAG and LLM application quality.",
        vec!["ragas-rs evaluates RAG and LLM applications.".to_string()],
    )])
    .expect("example dataset");

    let response_length = Arc::new(FnMetric::new("response_length", |sample| {
        let score = sample.response.len() as f64;
        Box::pin(async move {
            Ok(MetricResult::success(
                "response_length",
                MetricValue::numeric(score),
            ))
        })
    }));

    let report = evaluate(
        &dataset,
        &[response_length],
        EvaluationOptions { concurrency: 1 },
    )
    .await;

    println!(
        "{} sample(s), metrics: {}",
        report.results.len(),
        report.metric_names.join(", ")
    );
}
