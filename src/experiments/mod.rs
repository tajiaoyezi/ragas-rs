use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{EvaluationReport, MetricValue};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExperimentRecord {
    pub run_id: String,
    pub dataset_name: String,
    pub sample_count: usize,
    pub metric_names: Vec<String>,
    pub provider_config: BTreeMap<String, String>,
    pub report: EvaluationReport,
    pub outputs: BTreeMap<String, String>,
}

impl ExperimentRecord {
    pub fn new(
        run_id: impl Into<String>,
        dataset_name: impl Into<String>,
        sample_count: usize,
        metric_names: Vec<String>,
        provider_config: BTreeMap<String, String>,
        report: EvaluationReport,
    ) -> Self {
        Self {
            run_id: run_id.into(),
            dataset_name: dataset_name.into(),
            sample_count,
            metric_names,
            provider_config,
            report,
            outputs: BTreeMap::new(),
        }
    }

    pub fn with_output(mut self, name: impl Into<String>, location: impl Into<String>) -> Self {
        self.outputs.insert(name.into(), location.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunComparison {
    pub baseline_run_id: String,
    pub candidate_run_id: String,
    pub sample_count_delta: isize,
    pub metric_deltas: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExperimentSummary {
    pub run_id: String,
    pub dataset_name: String,
    pub sample_count: usize,
    pub metric_means: BTreeMap<String, f64>,
    pub provider_config: BTreeMap<String, String>,
    pub outputs: BTreeMap<String, String>,
}

pub fn compare_runs(baseline: &ExperimentRecord, candidate: &ExperimentRecord) -> RunComparison {
    let baseline_means = metric_means(baseline);
    let candidate_means = metric_means(candidate);
    let mut metric_deltas = BTreeMap::new();

    for metric_name in baseline
        .metric_names
        .iter()
        .chain(candidate.metric_names.iter())
    {
        if metric_deltas.contains_key(metric_name) {
            continue;
        }
        let baseline_mean = baseline_means.get(metric_name).copied().unwrap_or(0.0);
        let candidate_mean = candidate_means.get(metric_name).copied().unwrap_or(0.0);
        metric_deltas.insert(metric_name.clone(), candidate_mean - baseline_mean);
    }

    RunComparison {
        baseline_run_id: baseline.run_id.clone(),
        candidate_run_id: candidate.run_id.clone(),
        sample_count_delta: candidate.sample_count as isize - baseline.sample_count as isize,
        metric_deltas,
    }
}

pub fn summarize_experiment(record: &ExperimentRecord) -> ExperimentSummary {
    ExperimentSummary {
        run_id: record.run_id.clone(),
        dataset_name: record.dataset_name.clone(),
        sample_count: record.sample_count,
        metric_means: metric_means(record),
        provider_config: record.provider_config.clone(),
        outputs: record.outputs.clone(),
    }
}

fn metric_means(record: &ExperimentRecord) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    for sample in &record.report.results {
        for result in &sample.results {
            let Some(MetricValue::Numeric(value)) = &result.value else {
                continue;
            };
            let entry = totals.entry(result.metric_name.clone()).or_insert((0.0, 0));
            entry.0 += value;
            entry.1 += 1;
        }
    }

    totals
        .into_iter()
        .filter_map(|(metric_name, (sum, count))| {
            (count > 0).then_some((metric_name, sum / count as f64))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MetricResult, MetricValue, SampleEvaluation};

    fn provider_config() -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                "llm".to_string(),
                "openai-compatible:gpt-4o-mini".to_string(),
            ),
            (
                "embedding".to_string(),
                "openai-compatible:text-embedding-3-small".to_string(),
            ),
        ])
    }

    fn report(faithfulness: f64, relevancy: f64) -> EvaluationReport {
        EvaluationReport {
            metric_names: vec!["faithfulness".to_string(), "answer_relevancy".to_string()],
            results: vec![
                SampleEvaluation {
                    sample_index: 0,
                    results: vec![
                        MetricResult::success("faithfulness", MetricValue::numeric(faithfulness)),
                        MetricResult::success("answer_relevancy", MetricValue::numeric(relevancy)),
                    ],
                },
                SampleEvaluation {
                    sample_index: 1,
                    results: vec![
                        MetricResult::success("faithfulness", MetricValue::numeric(faithfulness)),
                        MetricResult::success("answer_relevancy", MetricValue::numeric(relevancy)),
                    ],
                },
            ],
        }
    }

    fn record(run_id: &str, faithfulness: f64, relevancy: f64) -> ExperimentRecord {
        ExperimentRecord::new(
            run_id,
            "eval-dataset",
            2,
            vec!["faithfulness".to_string(), "answer_relevancy".to_string()],
            provider_config(),
            report(faithfulness, relevancy),
        )
        .with_output("report_json", format!("memory://reports/{run_id}.json"))
    }

    #[test]
    fn test_15_1_1_experiment_records_inputs_metrics_provider_config_and_outputs() {
        // SCEN-15.1.1 / AC1 / TEST-15.1.1
        let record = record("run-a", 0.75, 0.8);

        assert_eq!(record.run_id, "run-a");
        assert_eq!(record.dataset_name, "eval-dataset");
        assert_eq!(record.sample_count, 2);
        assert_eq!(
            record.metric_names,
            vec!["faithfulness".to_string(), "answer_relevancy".to_string()]
        );
        assert_eq!(
            record.provider_config.get("llm").map(String::as_str),
            Some("openai-compatible:gpt-4o-mini")
        );
        assert_eq!(
            record.outputs.get("report_json").map(String::as_str),
            Some("memory://reports/run-a.json")
        );
        assert_eq!(record.report.results.len(), 2);
    }

    #[test]
    fn test_15_1_2_compare_runs_computes_metric_deltas() {
        // SCEN-15.1.2 / AC2 / TEST-15.1.2
        let baseline = record("baseline", 0.60, 0.90);
        let candidate = record("candidate", 0.85, 0.75);

        let comparison = compare_runs(&baseline, &candidate);

        assert_eq!(comparison.baseline_run_id, "baseline");
        assert_eq!(comparison.candidate_run_id, "candidate");
        assert_eq!(comparison.sample_count_delta, 0);
        assert!((comparison.metric_deltas["faithfulness"] - 0.25).abs() < 1e-9);
        assert!((comparison.metric_deltas["answer_relevancy"] - -0.15).abs() < 1e-9);
    }

    #[test]
    fn test_15_1_3_report_summary_serializes_to_json() {
        // SCEN-15.1.3 / AC3 / TEST-15.1.3
        let record = record("summary-run", 0.70, 0.88);

        let summary = summarize_experiment(&record);
        let json = serde_json::to_value(&summary).expect("summary JSON");

        assert_eq!(json["run_id"], "summary-run");
        assert_eq!(json["dataset_name"], "eval-dataset");
        assert_eq!(json["sample_count"], 2);
        assert_eq!(json["metric_means"]["faithfulness"], 0.70);
        assert_eq!(json["metric_means"]["answer_relevancy"], 0.88);
        assert_eq!(
            json["provider_config"]["embedding"],
            "openai-compatible:text-embedding-3-small"
        );
        assert_eq!(
            json["outputs"]["report_json"],
            "memory://reports/summary-run.json"
        );
    }
}
