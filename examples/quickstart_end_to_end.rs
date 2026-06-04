use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use ragas::{
    EvaluationOptions, FaithfulnessMetric, FnMetric, LlmProvider, LlmRequest, LlmResponse, Metric,
    MetricResult, MetricValue, RagasError, SingleTurnSample, Synthesizer, evaluate, rouge_l_recall,
};

/// End-to-end, fully OFFLINE walkthrough of the ragas-rs stack:
///
///   1. a [`Synthesizer`] turns a short document into a real [`EvaluationDataset`]
///      using a scripted LLM (no network, no API key);
///   2. [`evaluate`] runs that dataset through real metrics — deterministic ROUGE-L
///      recall plus the genuine [`FaithfulnessMetric`] pipeline driven by another
///      scripted LLM;
///   3. the per-metric scores are printed.
///
/// The metrics and the synthesizer are the real implementations from the library; only
/// the provider responses are canned here so the example is reproducible. The LLM-driven
/// paths are exercised against mocks only — real-LLM behaviour is UNVERIFIED. To get a
/// real signal, run with [`ragas::OpenAiCompatibleClient::from_env`] and `OPENAI_API_KEY`.
#[tokio::main]
async fn main() {
    // --- Step 1: synthesize a dataset from a document -----------------------------------
    //
    // The document below tokenizes into two ~60-char chunks. The synthesizer therefore
    // makes three real `generate()` calls: one single-hop Q/A per chunk, then one
    // multi-hop Q/A over the single `next` edge connecting them. We script exactly those
    // three responses, in order.
    const DOCUMENT: &str = "Ragas evaluates retrieval quality with grounded metrics. Rust services compile to fast native binaries.";

    let synth_llm = Arc::new(ScriptedLlm::new(vec![
        r#"{"question": "What does Ragas evaluate?", "answer": "Ragas evaluates retrieval quality."}"#,
        r#"{"question": "What do Rust services compile to?", "answer": "Rust services compile to fast native binaries."}"#,
        r#"{"question": "How do Ragas and Rust each deliver quality?", "answer": "Ragas evaluates retrieval quality and Rust compiles to fast binaries."}"#,
    ]));

    let dataset = Synthesizer::new(synth_llm.clone())
        .with_chunk_max_chars(60)
        .generate_testset("quickstart-doc", DOCUMENT)
        .await
        .expect("synthesizer should produce a dataset from the document");

    println!(
        "Synthesized {} sample(s) from {} scripted LLM call(s):",
        dataset.len(),
        synth_llm.call_count(),
    );
    for sample in dataset.iter() {
        let kind = sample
            .metadata
            .get("synthesis_type")
            .map(String::as_str)
            .unwrap_or("?");
        println!("  [{kind}] Q: {}", sample.user_input);
    }
    println!();

    // --- Step 2: evaluate the dataset with real metrics ---------------------------------
    //
    // ROUGE-L recall is deterministic and offline: it scores `response` against
    // `reference` using lexical overlap. The synthesizer sets both to the same answer, so
    // every sample scores 1.0 — a real (if unsurprising) signal that the wiring works.
    let rouge = Arc::new(FnMetric::new("rouge_l", |sample: &SingleTurnSample| {
        let response = sample.response.clone();
        let reference = sample.reference.clone();
        Box::pin(async move {
            let reference = reference.ok_or_else(|| RagasError::Parse {
                message: "rouge_l requires a reference".to_string(),
            })?;
            let score =
                rouge_l_recall(&response, &reference)
                    .score
                    .ok_or_else(|| RagasError::Parse {
                        message: "rouge_l produced no score".to_string(),
                    })?;
            Ok(MetricResult::success(
                "rouge_l",
                MetricValue::numeric(score),
            ))
        })
    }));

    // Faithfulness is the genuine two-step LLM pipeline: per sample it (1) decomposes the
    // response into atomic statements, then (2) verifies each against the retrieved
    // contexts. We script one statement + one supporting verdict per sample, so each
    // sample scores 1/1 = 1.0. Two calls per sample x N samples, in dataset order.
    let mut faithfulness_script = Vec::new();
    for sample in dataset.iter() {
        faithfulness_script.push(format!(
            r#"{{"statements": [{}]}}"#,
            serde_json::to_string(&sample.response).expect("json string")
        ));
        faithfulness_script.push(
            r#"{"verdicts": [{"statement": "s", "verdict": 1, "reason": "supported by context"}]}"#
                .to_string(),
        );
    }
    let faithfulness_llm = Arc::new(ScriptedLlm::new(
        faithfulness_script.iter().map(String::as_str).collect(),
    ));
    let faithfulness = Arc::new(FaithfulnessMetric::new(faithfulness_llm.clone()));

    let metrics: Vec<Arc<dyn Metric>> = vec![rouge, faithfulness];
    let report = evaluate(&dataset, &metrics, EvaluationOptions { concurrency: 1 }).await;

    // --- Step 3: print a readable per-metric summary ------------------------------------
    println!(
        "Evaluation report ({} metric(s)):",
        report.metric_names.len()
    );
    for sample in &report.results {
        println!("  sample #{}", sample.sample_index);
        for result in &sample.results {
            match (
                result.value.as_ref().and_then(MetricValue::as_numeric),
                &result.error,
            ) {
                (Some(score), _) => println!("    {} = {score:.3}", result.metric_name),
                (None, Some(error)) => println!("    {} errored: {error}", result.metric_name),
                (None, None) => println!("    {} = <none>", result.metric_name),
            }
        }
    }

    // Mean per metric, so the run ends with a single headline number per metric.
    println!("\nMean per metric:");
    for (index, name) in report.metric_names.iter().enumerate() {
        let scores: Vec<f64> = report
            .results
            .iter()
            .filter_map(|sample| {
                sample.results[index]
                    .value
                    .as_ref()
                    .and_then(MetricValue::as_numeric)
            })
            .collect();
        if scores.is_empty() {
            println!("  {name}: no numeric scores");
        } else {
            let mean = scores.iter().sum::<f64>() / scores.len() as f64;
            println!("  {name}: {mean:.3} (over {} sample(s))", scores.len());
        }
    }
}

/// Mock LLM that replays scripted responses in order and records the prompts it saw, so
/// the synthesizer and faithfulness pipelines can be driven deterministically with no
/// network call. Mirrors the `ScriptedLlm` test double in `src/metric.rs` / `src/testset`.
struct ScriptedLlm {
    responses: Mutex<VecDeque<String>>,
    prompts: Mutex<Vec<String>>,
}

impl ScriptedLlm {
    fn new(responses: Vec<&str>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().map(str::to_string).collect()),
            prompts: Mutex::new(Vec::new()),
        }
    }

    fn call_count(&self) -> usize {
        self.prompts.lock().expect("prompts").len()
    }
}

#[async_trait]
impl LlmProvider for ScriptedLlm {
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, RagasError> {
        let prompt = request
            .messages
            .iter()
            .map(|message| message.content.clone())
            .collect::<Vec<_>>()
            .join("\n");
        self.prompts.lock().expect("prompts").push(prompt);
        let content = self
            .responses
            .lock()
            .expect("responses")
            .pop_front()
            .ok_or_else(|| RagasError::Provider {
                message: "scripted LLM ran out of responses".to_string(),
            })?;
        Ok(LlmResponse {
            content,
            usage: None,
        })
    }
}
