use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizationCandidate {
    pub id: String,
    pub prompt: String,
    pub model: Option<String>,
    pub parameters: BTreeMap<String, String>,
}

impl OptimizationCandidate {
    pub fn new(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            prompt: prompt.into(),
            model: None,
            parameters: BTreeMap::new(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

pub trait ObjectiveMetric {
    fn name(&self) -> &str;
    fn score(&self, candidate: &OptimizationCandidate) -> f64;
}

pub trait CandidateGenerator {
    fn initial_candidates(&self) -> Vec<OptimizationCandidate>;
    fn mutate(&self, candidate: &OptimizationCandidate, seed: u64) -> OptimizationCandidate;
}

pub trait Optimizer {
    fn optimize(
        &mut self,
        objective: &dyn ObjectiveMetric,
        generator: &dyn CandidateGenerator,
    ) -> OptimizationResult;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneticOptimizerConfig {
    pub seed: u64,
    pub generations: usize,
    pub population_size: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizationStep {
    pub generation: usize,
    pub candidate_id: String,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub objective_metric: String,
    pub best_candidate: OptimizationCandidate,
    pub best_score: f64,
    pub history: Vec<OptimizationStep>,
}

#[derive(Debug, Clone)]
pub struct GeneticOptimizer {
    config: GeneticOptimizerConfig,
    history: Vec<OptimizationStep>,
}

impl GeneticOptimizer {
    pub fn new(config: GeneticOptimizerConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
        }
    }

    pub fn history(&self) -> &[OptimizationStep] {
        &self.history
    }
}

impl Optimizer for GeneticOptimizer {
    fn optimize(
        &mut self,
        objective: &dyn ObjectiveMetric,
        _generator: &dyn CandidateGenerator,
    ) -> OptimizationResult {
        let _config = self.config;
        self.history.clear();
        OptimizationResult {
            objective_metric: objective.name().to_string(),
            best_candidate: OptimizationCandidate::new("", ""),
            best_score: 0.0,
            history: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct KeywordObjective {
        keyword: String,
    }

    impl ObjectiveMetric for KeywordObjective {
        fn name(&self) -> &str {
            "keyword_match"
        }

        fn score(&self, candidate: &OptimizationCandidate) -> f64 {
            if candidate.prompt.contains(&self.keyword) {
                1.0
            } else {
                0.0
            }
        }
    }

    struct StaticGenerator;

    impl CandidateGenerator for StaticGenerator {
        fn initial_candidates(&self) -> Vec<OptimizationCandidate> {
            vec![
                OptimizationCandidate::new("baseline", "Answer the question").with_model("gpt-a"),
                OptimizationCandidate::new("grounded", "Answer with grounded evidence")
                    .with_model("gpt-b"),
            ]
        }

        fn mutate(&self, candidate: &OptimizationCandidate, seed: u64) -> OptimizationCandidate {
            OptimizationCandidate::new(
                format!("{}-m{}", candidate.id, seed % 7),
                format!("{} variant-{}", candidate.prompt, seed % 7),
            )
            .with_model(candidate.model.clone().unwrap_or_else(|| "gpt-a".to_string()))
            .with_parameter("seed_mod", (seed % 7).to_string())
        }
    }

    struct VariantObjective;

    impl ObjectiveMetric for VariantObjective {
        fn name(&self) -> &str {
            "variant_score"
        }

        fn score(&self, candidate: &OptimizationCandidate) -> f64 {
            candidate
                .parameters
                .get("seed_mod")
                .and_then(|value| value.parse::<f64>().ok())
                .unwrap_or(0.0)
        }
    }

    #[test]
    fn test_15_2_1_optimizer_trait_accepts_objective_metric_and_candidate_generator() {
        // SCEN-15.2.1 / AC1 / TEST-15.2.1
        let mut optimizer = GeneticOptimizer::new(GeneticOptimizerConfig {
            seed: 11,
            generations: 1,
            population_size: 2,
        });
        let objective = KeywordObjective {
            keyword: "grounded".to_string(),
        };

        let result = optimizer.optimize(&objective, &StaticGenerator);

        assert_eq!(result.objective_metric, "keyword_match");
        assert_eq!(result.best_candidate.id, "grounded");
        assert_eq!(result.best_candidate.model.as_deref(), Some("gpt-b"));
        assert_eq!(result.best_score, 1.0);
    }

    #[test]
    fn test_15_2_2_genetic_optimizer_evolves_deterministically_with_seeded_rng() {
        // SCEN-15.2.2 / AC2 / TEST-15.2.2
        let config = GeneticOptimizerConfig {
            seed: 42,
            generations: 3,
            population_size: 3,
        };
        let mut first = GeneticOptimizer::new(config);
        let mut second = GeneticOptimizer::new(config);

        let first_result = first.optimize(&VariantObjective, &StaticGenerator);
        let second_result = second.optimize(&VariantObjective, &StaticGenerator);

        assert_eq!(first_result.history, second_result.history);
        assert_eq!(first_result.best_candidate, second_result.best_candidate);
        assert!(first_result.best_score > 0.0);
        assert!(
            first_result
                .history
                .iter()
                .any(|step| step.generation == 3)
        );
    }

    #[test]
    fn test_15_2_3_optimizer_history_is_inspectable() {
        // SCEN-15.2.3 / AC3 / TEST-15.2.3
        let mut optimizer = GeneticOptimizer::new(GeneticOptimizerConfig {
            seed: 7,
            generations: 2,
            population_size: 2,
        });

        let result = optimizer.optimize(&VariantObjective, &StaticGenerator);

        assert_eq!(optimizer.history(), result.history.as_slice());
        assert_eq!(result.history.len(), 6);
        assert_eq!(result.history[0].generation, 0);
        assert!(
            result
                .history
                .iter()
                .all(|step| !step.candidate_id.is_empty())
        );
    }
}
