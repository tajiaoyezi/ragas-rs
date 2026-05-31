use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    ChatMessage, EmbeddingProvider, EmbeddingRequest, LlmProvider, LlmRequest, RagasError,
    TokenUsage,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkPrompt {
    pub id: String,
    pub text: String,
}

impl BenchmarkPrompt {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
        }
    }
}

#[derive(Clone)]
pub enum BenchmarkProvider {
    Llm {
        name: String,
        provider: Arc<dyn LlmProvider>,
    },
    Embedding {
        name: String,
        provider: Arc<dyn EmbeddingProvider>,
    },
}

impl BenchmarkProvider {
    pub fn llm(name: impl Into<String>, provider: Arc<dyn LlmProvider>) -> Self {
        Self::Llm {
            name: name.into(),
            provider,
        }
    }

    pub fn embedding(name: impl Into<String>, provider: Arc<dyn EmbeddingProvider>) -> Self {
        Self::Embedding {
            name: name.into(),
            provider,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CostRates {
    pub prompt_per_1k_tokens: f64,
    pub completion_per_1k_tokens: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkMeasurement {
    pub provider_name: String,
    pub provider_kind: String,
    pub prompt_id: String,
    pub output_units: usize,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CostSummary {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub measurements: Vec<BenchmarkMeasurement>,
    pub cost: CostSummary,
}

pub async fn run_provider_benchmark(
    providers: &[BenchmarkProvider],
    prompts: &[BenchmarkPrompt],
    rates: CostRates,
) -> Result<BenchmarkReport, RagasError> {
    let mut measurements = Vec::new();
    let mut cost = CostSummary::default();

    for provider in providers {
        for prompt in prompts {
            let measurement = match provider {
                BenchmarkProvider::Llm { name, provider } => {
                    let response = provider
                        .generate(LlmRequest {
                            messages: vec![ChatMessage::user(prompt.text.clone())],
                            temperature: Some(0.0),
                        })
                        .await?;
                    BenchmarkMeasurement {
                        provider_name: name.clone(),
                        provider_kind: "llm".to_string(),
                        prompt_id: prompt.id.clone(),
                        output_units: response.content.len(),
                        usage: response.usage,
                    }
                }
                BenchmarkProvider::Embedding { name, provider } => {
                    let response = provider
                        .embed(EmbeddingRequest {
                            input: vec![prompt.text.clone()],
                        })
                        .await?;
                    BenchmarkMeasurement {
                        provider_name: name.clone(),
                        provider_kind: "embedding".to_string(),
                        prompt_id: prompt.id.clone(),
                        output_units: response.embeddings.iter().map(Vec::len).sum(),
                        usage: response.usage,
                    }
                }
            };
            cost.add_usage(measurement.usage.as_ref(), rates);
            measurements.push(measurement);
        }
    }

    Ok(BenchmarkReport { measurements, cost })
}

impl CostSummary {
    fn add_usage(&mut self, usage: Option<&TokenUsage>, rates: CostRates) {
        let Some(usage) = usage else {
            return;
        };
        let prompt_tokens = usage.prompt_tokens.unwrap_or(0) as u64;
        let completion_tokens = usage.completion_tokens.unwrap_or(0) as u64;
        let total_tokens = usage
            .total_tokens
            .map(u64::from)
            .unwrap_or(prompt_tokens + completion_tokens);

        self.prompt_tokens += prompt_tokens;
        self.completion_tokens += completion_tokens;
        self.total_tokens += total_tokens;
        self.estimated_cost_usd += prompt_tokens as f64 / 1000.0 * rates.prompt_per_1k_tokens
            + completion_tokens as f64 / 1000.0 * rates.completion_per_1k_tokens;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockEmbeddingProvider, MockLlmProvider};

    fn prompts() -> Vec<BenchmarkPrompt> {
        vec![
            BenchmarkPrompt::new("p1", "Score faithfulness"),
            BenchmarkPrompt::new("p2", "Embed relevancy"),
        ]
    }

    fn providers() -> Vec<BenchmarkProvider> {
        let llm_usage = TokenUsage {
            prompt_tokens: Some(5),
            completion_tokens: Some(2),
            total_tokens: Some(7),
        };
        let embedding_usage = TokenUsage {
            prompt_tokens: Some(3),
            completion_tokens: Some(0),
            total_tokens: Some(3),
        };

        vec![
            BenchmarkProvider::llm(
                "mock-llm",
                Arc::new(MockLlmProvider::new("benchmark response").with_usage(llm_usage)),
            ),
            BenchmarkProvider::embedding(
                "mock-embedding",
                Arc::new(
                    MockEmbeddingProvider::new(vec![vec![1.0, 0.0]]).with_usage(embedding_usage),
                ),
            ),
        ]
    }

    fn rates() -> CostRates {
        CostRates {
            prompt_per_1k_tokens: 1.0,
            completion_per_1k_tokens: 2.0,
        }
    }

    #[tokio::test]
    async fn test_15_3_1_benchmark_runner_executes_providers_over_fixed_prompts() {
        // SCEN-15.3.1 / AC1 / TEST-15.3.1
        let report = run_provider_benchmark(&providers(), &prompts(), rates())
            .await
            .expect("benchmark report");

        assert_eq!(report.measurements.len(), 4);
        assert_eq!(report.measurements[0].provider_name, "mock-llm");
        assert_eq!(report.measurements[0].provider_kind, "llm");
        assert_eq!(report.measurements[0].prompt_id, "p1");
        assert!(report.measurements[0].output_units > 0);
        assert_eq!(report.measurements[2].provider_name, "mock-embedding");
        assert_eq!(report.measurements[2].provider_kind, "embedding");
    }

    #[tokio::test]
    async fn test_15_3_2_cost_summary_aggregates_usage_and_configured_rates() {
        // SCEN-15.3.2 / AC2 / TEST-15.3.2
        let report = run_provider_benchmark(&providers(), &prompts(), rates())
            .await
            .expect("benchmark report");

        assert_eq!(report.cost.prompt_tokens, 16);
        assert_eq!(report.cost.completion_tokens, 4);
        assert_eq!(report.cost.total_tokens, 20);
        assert!((report.cost.estimated_cost_usd - 0.024).abs() < 1e-12);
    }

    #[tokio::test]
    async fn test_15_3_3_benchmark_output_is_stable_json() {
        // SCEN-15.3.3 / AC3 / TEST-15.3.3
        let first = run_provider_benchmark(&providers(), &prompts(), rates())
            .await
            .expect("first benchmark");
        let second = run_provider_benchmark(&providers(), &prompts(), rates())
            .await
            .expect("second benchmark");

        let first_json = serde_json::to_string(&first).expect("first JSON");
        let second_json = serde_json::to_string(&second).expect("second JSON");

        assert_eq!(first_json, second_json);
        assert!(first_json.contains("\"provider_name\":\"mock-llm\""));
        assert!(first_json.contains("\"estimated_cost_usd\":0.024"));
    }
}
