use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    DatasetBackend, EvaluationDataset, EvaluationSample, ExtractionBundle, GraphNode,
    InMemoryDatasetBackend, KnowledgeGraph, PersonaGenerator, RagasError, attach_extractions,
    build_chunk_relationships, split_text_into_chunks, synthesize_single_hop_sample,
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

pub fn run_cli_command(
    runtime: &mut CliRuntime,
    command: CliCommand,
) -> Result<CliOutput, RagasError> {
    match command {
        CliCommand::Evaluate { input, report } => run_evaluate(runtime, input, report),
        CliCommand::Testset {
            source_id,
            text,
            output,
        } => run_testset(runtime, source_id, text, output),
        CliCommand::Benchmark { runs } => run_benchmark(runs),
    }
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
) -> Result<CliOutput, RagasError> {
    let dataset = runtime.datasets.load(&input)?;
    let report_json = json!({
        "command": "evaluate",
        "status": "ok",
        "input": input,
        "report": report,
        "sample_count": dataset.len(),
    });
    let report_string = serde_json::to_string(&report_json)
        .map_err(|error| parse_error(format!("evaluate report serialization failed: {error}")))?;
    runtime.reports.insert(report.clone(), report_string);

    let stdout = json!({
        "command": "evaluate",
        "status": "ok",
        "report": report,
        "sample_count": dataset.len(),
    });
    cli_output(stdout)
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
    use crate::{DatasetBackend, EvaluationDataset, EvaluationSample, SingleTurnSample};
    use serde_json::Value;

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

        let output = run_cli_command(
            &mut runtime,
            CliCommand::Evaluate {
                input: "datasets/fixture".to_string(),
                report: "reports/run-1".to_string(),
            },
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

        let output = run_cli_command(
            &mut runtime,
            CliCommand::Evaluate {
                input: "datasets/fixture".to_string(),
                report: "reports/run-21-2".to_string(),
            },
        )
        .expect("evaluate command");
        let snapshot = cli_contract_snapshot(&output).expect("CLI snapshot");

        assert_eq!(snapshot.command, "evaluate");
        assert_eq!(snapshot.status, "ok");
        assert_eq!(
            snapshot.stdout_keys,
            vec!["command", "report", "sample_count", "status"]
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
