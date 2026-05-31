use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{EmbeddingProvider, LlmProvider, RagasError, TokenUsage};

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
    _providers: &[BenchmarkProvider],
    _prompts: &[BenchmarkPrompt],
    _rates: CostRates,
) -> Result<BenchmarkReport, RagasError> {
    Ok(BenchmarkReport::default())
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
                    MockEmbeddingProvider::new(vec![vec![1.0, 0.0]])
                        .with_usage(embedding_usage),
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
