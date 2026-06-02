use std::collections::BTreeMap;

use ragas::{
    EvaluationReport, ExperimentRecord, MetricResult, MetricValue, SampleEvaluation, compare_runs,
    summarize_experiment,
};

fn report(faithfulness: f64, answer_relevancy: f64) -> EvaluationReport {
    EvaluationReport {
        metric_names: vec!["faithfulness".to_string(), "answer_relevancy".to_string()],
        results: vec![
            sample(0, faithfulness, answer_relevancy),
            sample(1, faithfulness, answer_relevancy),
        ],
    }
}

fn sample(index: usize, faithfulness: f64, answer_relevancy: f64) -> SampleEvaluation {
    SampleEvaluation {
        sample_index: index,
        results: vec![
            MetricResult::success("faithfulness", MetricValue::numeric(faithfulness)),
            MetricResult::success("answer_relevancy", MetricValue::numeric(answer_relevancy)),
        ],
    }
}

fn experiment_record(
    run_id: &str,
    faithfulness: f64,
    answer_relevancy: f64,
) -> ExperimentRecord {
    ExperimentRecord::new(
        run_id,
        "quickstart-dataset",
        2,
        vec!["faithfulness".to_string(), "answer_relevancy".to_string()],
        BTreeMap::from([("llm".to_string(), "mock-llm".to_string())]),
        report(faithfulness, answer_relevancy),
    )
    .with_output("report_json", format!("memory://reports/{run_id}.json"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let baseline = experiment_record("baseline", 0.72, 0.81);
    let candidate = experiment_record("candidate", 0.86, 0.84);
    let summary = summarize_experiment(&candidate);
    let comparison = compare_runs(&baseline, &candidate);

    let output = serde_json::json!({
        "summary": summary,
        "comparison": comparison,
    });

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
