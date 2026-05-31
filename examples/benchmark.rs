use std::sync::Arc;

use ragas::{
    BenchmarkPrompt, BenchmarkProvider, CostRates, MockLlmProvider, TokenUsage,
    run_provider_benchmark,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = BenchmarkProvider::llm(
        "mock-llm",
        Arc::new(MockLlmProvider::new("ok").with_usage(TokenUsage {
            prompt_tokens: Some(8),
            completion_tokens: Some(2),
            total_tokens: Some(10),
        })),
    );
    let prompts = vec![BenchmarkPrompt::new("faithfulness", "Score faithfulness")];
    let report = run_provider_benchmark(
        &[provider],
        &prompts,
        CostRates {
            prompt_per_1k_tokens: 1.0,
            completion_per_1k_tokens: 2.0,
        },
    )
    .await?;

    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}
