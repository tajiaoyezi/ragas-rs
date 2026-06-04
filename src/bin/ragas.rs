//! `ragas` — command-line interface for the ragas evaluation library.
//!
//! A thin, dependency-free executable: it parses arguments with `std::env::args`, does the
//! file IO, and delegates the actual work to the library's tested CLI handlers
//! ([`ragas::run_cli_command_with_provider`]). Provider configuration is resolved centrally
//! by [`ragas::ProviderConfig`] (environment variables / `.env` file).
//!
//! Usage:
//!   ragas config
//!   ragas evaluate  --dataset <file.jsonl> [--report <file.json>]
//!   ragas testset   --doc <file.txt> --source-id <id> [--out <file.jsonl>]
//!   ragas benchmark [--runs <n>]

use std::process::ExitCode;
use std::sync::Arc;

use ragas::{
    CliCommand, CliRuntime, DatasetBackend, EvaluationDataset, EvaluationSample, LlmProvider,
    ProviderConfig, run_cli_command_with_provider,
};

const USAGE: &str = "\
ragas — RAG/LLM evaluation CLI

USAGE:
    ragas <command> [options]

COMMANDS:
    config                          Print the resolved provider configuration (secrets redacted).
    evaluate --dataset <file.jsonl> Evaluate a JSONL dataset. Runs offline ROUGE-L always, plus the
             [--report <file>]      LLM metrics (faithfulness, context recall) when an API key is
                                    configured. Optionally write the full JSON report to a file.
    testset  --doc <file.txt>       Generate a deterministic single-hop test dataset from a text
             --source-id <id>       document. Optionally write it out as JSONL.
             [--out <file.jsonl>]
    benchmark [--runs <n>]          Run the provider micro-benchmark (default 1 run).
    help                            Show this help.

CONFIGURATION:
    Set OPENAI_API_KEY / OPENAI_BASE_URL / OPENAI_MODEL (and OPENAI_EMBEDDING_* for embeddings)
    in your shell or a .env file (see .env.example). Inspect what is resolved with `ragas config`.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str) {
        None | Some("help") | Some("-h") | Some("--help") => Ok(USAGE.to_string()),
        Some("config") => Ok(cmd_config()),
        Some("evaluate") => cmd_evaluate(&args[1..]),
        Some("testset") => cmd_testset(&args[1..]),
        Some("benchmark") => cmd_benchmark(&args[1..]),
        Some(other) => Err(format!("unknown command '{other}'. Run `ragas help`.")),
    }
}

fn cmd_config() -> String {
    let config = ProviderConfig::from_env();
    let mut out = format!(
        "Resolved ragas provider configuration (env > .env > default):\n\n{}\n",
        config.redacted_summary()
    );
    if config.has_api_key() {
        out.push_str("\n[ok] chat API key set — LLM metrics and testset run live.");
    } else {
        out.push_str(
            "\n[--] no chat API key — LLM features stay offline. Set OPENAI_API_KEY or use a .env file.",
        );
    }
    out
}

fn cmd_evaluate(args: &[String]) -> Result<String, String> {
    // A configured key turns on the LLM metrics; with none, evaluate stays offline (ROUGE only).
    let provider = ProviderConfig::from_env().chat_provider();
    cmd_evaluate_with(args, provider)
}

fn cmd_evaluate_with(
    args: &[String],
    provider: Option<Arc<dyn LlmProvider>>,
) -> Result<String, String> {
    let dataset_path = flag(args, "--dataset").ok_or("evaluate requires --dataset <file.jsonl>")?;
    let report_path = flag(args, "--report");

    let content = std::fs::read_to_string(&dataset_path)
        .map_err(|error| format!("cannot read dataset '{dataset_path}': {error}"))?;
    let dataset = EvaluationDataset::<EvaluationSample>::from_jsonl_str(&content)
        .map_err(|error| format!("cannot parse JSONL dataset '{dataset_path}': {error}"))?;

    let mut runtime = CliRuntime::new();
    runtime
        .datasets_mut()
        .save("input", &dataset)
        .map_err(|error| error.to_string())?;

    let output = run_cli_command_with_provider(
        &mut runtime,
        CliCommand::Evaluate {
            input: "input".to_string(),
            report: "report".to_string(),
        },
        provider,
    )
    .map_err(|error| error.to_string())?;

    if let Some(path) = report_path
        && let Some(report) = runtime.report("report")
    {
        std::fs::write(&path, report)
            .map_err(|error| format!("cannot write report '{path}': {error}"))?;
    }
    Ok(output.stdout)
}

fn cmd_testset(args: &[String]) -> Result<String, String> {
    let doc_path = flag(args, "--doc").ok_or("testset requires --doc <file.txt>")?;
    let source_id = flag(args, "--source-id").ok_or("testset requires --source-id <id>")?;
    let out_path = flag(args, "--out");

    let text = std::fs::read_to_string(&doc_path)
        .map_err(|error| format!("cannot read doc '{doc_path}': {error}"))?;

    let mut runtime = CliRuntime::new();
    let output = run_cli_command_with_provider(
        &mut runtime,
        CliCommand::Testset {
            source_id,
            text,
            output: "testset".to_string(),
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    if let Some(path) = out_path {
        let dataset = runtime
            .datasets()
            .load("testset")
            .map_err(|error| error.to_string())?;
        let jsonl = dataset.to_jsonl_string().map_err(|error| error.to_string())?;
        std::fs::write(&path, jsonl)
            .map_err(|error| format!("cannot write '{path}': {error}"))?;
    }
    Ok(output.stdout)
}

fn cmd_benchmark(args: &[String]) -> Result<String, String> {
    let runs = match flag(args, "--runs") {
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| format!("--runs must be a non-negative number, got '{value}'"))?,
        None => 1,
    };
    let mut runtime = CliRuntime::new();
    let output =
        run_cli_command_with_provider(&mut runtime, CliCommand::Benchmark { runs }, None)
            .map_err(|error| error.to_string())?;
    Ok(output.stdout)
}

/// Extract the value of `--flag <value>` from the argument list.
fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn no_args_and_help_print_usage() {
        assert!(run(&[]).unwrap().contains("USAGE"));
        assert!(run(&args(&["help"])).unwrap().contains("evaluate --dataset"));
        assert!(run(&args(&["--help"])).unwrap().contains("COMMANDS"));
    }

    #[test]
    fn config_command_reports_resolved_configuration() {
        let out = run(&args(&["config"])).expect("config");
        assert!(out.contains("provider configuration"));
        assert!(out.contains("OPENAI_BASE_URL"));
        assert!(out.contains("OPENAI_MODEL"));
    }

    #[test]
    fn unknown_command_errors() {
        let error = run(&args(&["frobnicate"])).expect_err("unknown command");
        assert!(error.contains("unknown command 'frobnicate'"));
    }

    #[test]
    fn flag_parsing_extracts_values_and_handles_absence() {
        let a = args(&["--dataset", "data.jsonl", "--report", "out.json"]);
        assert_eq!(flag(&a, "--dataset").as_deref(), Some("data.jsonl"));
        assert_eq!(flag(&a, "--report").as_deref(), Some("out.json"));
        assert_eq!(flag(&a, "--missing"), None);
        // A trailing flag with no value is treated as absent.
        assert_eq!(flag(&args(&["--dataset"]), "--dataset"), None);
    }

    #[test]
    fn evaluate_requires_a_dataset_flag() {
        let error = run(&args(&["evaluate"])).expect_err("missing --dataset");
        assert!(error.contains("--dataset"));
    }

    #[test]
    fn evaluate_runs_offline_rouge_over_a_jsonl_file() {
        // Write a tiny JSONL dataset (one single-turn sample with a reference) and evaluate it.
        let path = std::env::temp_dir().join("ragas_cli_eval_smoke.jsonl");
        let line = r#"{"sample_type":"single_turn","user_input":"q","response":"the cat sat","retrieved_contexts":["ctx"],"reference":"the cat sat","metadata":{}}"#;
        std::fs::write(&path, format!("{line}\n")).expect("write temp dataset");

        // Force the offline path (no provider) so the test never makes a network call,
        // regardless of any OPENAI_API_KEY in the ambient environment.
        let out = cmd_evaluate_with(&args(&["--dataset", path.to_str().unwrap()]), None)
            .expect("evaluate runs");
        std::fs::remove_file(&path).ok();

        // The offline ROUGE-L metric ran (response == reference -> recall 1.0) and is reported.
        assert!(out.contains("rouge_l"));
        assert!(out.contains("\"sample_count\":1"));
    }
}
