use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::CacheKey;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OptimizerFamily {
    Genetic,
    Dspy,
    MiproV2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OptimizerRuntime {
    RustNative,
    PythonRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DspyCacheContract {
    pub cache_key: CacheKey,
    pub deterministic_keys: bool,
    pub value_format: String,
    pub python_runtime_supported: bool,
    pub unsupported_runtime_behavior: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiproV2Trial {
    pub index: usize,
    pub seed: u64,
    pub candidate_limit: usize,
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
        generator: &dyn CandidateGenerator,
    ) -> OptimizationResult {
        self.history.clear();
        let population_size = self.config.population_size.max(1);
        let mut rng_state = self.config.seed;
        let mut population = normalize_population(
            generator.initial_candidates(),
            population_size,
            generator,
            &mut rng_state,
        );

        let mut best_candidate = population
            .first()
            .cloned()
            .unwrap_or_else(|| OptimizationCandidate::new("empty", ""));
        let mut best_score = f64::NEG_INFINITY;

        for generation in 0..=self.config.generations {
            let mut generation_scores = Vec::with_capacity(population.len());
            for candidate in &population {
                let score = objective.score(candidate);
                self.history.push(OptimizationStep {
                    generation,
                    candidate_id: candidate.id.clone(),
                    score,
                });
                generation_scores.push((candidate.clone(), score));
                if score > best_score {
                    best_candidate = candidate.clone();
                    best_score = score;
                }
            }

            if generation < self.config.generations {
                generation_scores.sort_by(|left, right| {
                    right
                        .1
                        .partial_cmp(&left.1)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| left.0.id.cmp(&right.0.id))
                });
                let parent = generation_scores
                    .first()
                    .map(|(candidate, _)| candidate.clone())
                    .unwrap_or_else(|| best_candidate.clone());
                population = (0..population_size)
                    .map(|_| generator.mutate(&parent, next_seed(&mut rng_state)))
                    .collect();
            }
        }

        OptimizationResult {
            objective_metric: objective.name().to_string(),
            best_candidate,
            best_score,
            history: self.history.clone(),
        }
    }
}

pub fn dspy_cache_contract(payload: &Value) -> DspyCacheContract {
    DspyCacheContract {
        cache_key: CacheKey::derive("optimizer.dspy", payload),
        deterministic_keys: true,
        value_format: "json".to_string(),
        python_runtime_supported: false,
        unsupported_runtime_behavior: Some(
            "Python DSPy runtime is not embedded in the Rust crate".to_string(),
        ),
    }
}

pub fn plan_mipro_v2_trials(seed: u64, trials: usize) -> Vec<MiproV2Trial> {
    let mut state = seed;
    (0..trials)
        .map(|index| {
            let trial_seed = if index == 0 {
                state
            } else {
                next_seed(&mut state)
            };
            MiproV2Trial {
                index,
                seed: trial_seed,
                candidate_limit: (index + 1) * 4,
            }
        })
        .collect()
}

fn normalize_population(
    mut candidates: Vec<OptimizationCandidate>,
    population_size: usize,
    generator: &dyn CandidateGenerator,
    rng_state: &mut u64,
) -> Vec<OptimizationCandidate> {
    if candidates.is_empty() {
        candidates.push(OptimizationCandidate::new("seed", ""));
    }
    while candidates.len() < population_size {
        let parent = candidates[candidates.len() - 1].clone();
        candidates.push(generator.mutate(&parent, next_seed(rng_state)));
    }
    candidates.truncate(population_size);
    candidates
}

fn next_seed(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
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
            .with_model(
                candidate
                    .model
                    .clone()
                    .unwrap_or_else(|| "gpt-a".to_string()),
            )
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
        assert!(first_result.history.iter().any(|step| step.generation == 3));
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

    #[test]
    fn test_21_1_2_dspy_cache_contract_records_deterministic_and_unsupported_behavior() {
        // SCEN-21.1.2 / AC2 / TEST-21.1.2
        let left = serde_json::json!({
            "api_key": "sk-secret",
            "optimizer": "MIPROv2",
            "prompt": "answer with evidence",
            "params": {"num_trials": 4, "seed": 7}
        });
        let right = serde_json::json!({
            "params": {"seed": 7, "num_trials": 4},
            "prompt": "answer with evidence",
            "optimizer": "MIPROv2",
            "api_key": "sk-secret"
        });

        let left_contract = dspy_cache_contract(&left);
        let right_contract = dspy_cache_contract(&right);

        assert_eq!(left_contract.cache_key.namespace, "optimizer.dspy");
        assert_eq!(
            left_contract.cache_key.digest,
            right_contract.cache_key.digest
        );
        assert!(left_contract.deterministic_keys);
        assert_eq!(left_contract.value_format, "json");
        assert!(!left_contract.python_runtime_supported);
        assert!(
            left_contract
                .unsupported_runtime_behavior
                .as_deref()
                .is_some_and(|message| message.contains("Python DSPy runtime"))
        );

        let redacted = left_contract.cache_key.redacted_payload.to_string();
        assert!(!redacted.contains("sk-secret"));
        assert!(redacted.contains("[redacted]"));
    }

    #[test]
    fn test_30_1_2_optimizer_planning_is_deterministic_and_redacted() {
        // SCEN-30.1.2 / AC2 / TEST-30.1.2
        let payload = serde_json::json!({
            "api_key": "sk-secret",
            "optimizer": "MIPROv2",
            "prompt": "answer with evidence",
            "params": {"num_trials": 3, "seed": 9}
        });
        let first = dspy_cache_contract(&payload);
        let second = dspy_cache_contract(&payload);
        assert_eq!(first.cache_key.digest, second.cache_key.digest);
        assert!(
            !first
                .cache_key
                .redacted_payload
                .to_string()
                .contains("sk-secret")
        );

        let trials = plan_mipro_v2_trials(9, 3);
        assert_eq!(
            trials,
            vec![
                MiproV2Trial {
                    index: 0,
                    seed: 9,
                    candidate_limit: 4,
                },
                MiproV2Trial {
                    index: 1,
                    seed: 9_u64
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407),
                    candidate_limit: 8,
                },
                MiproV2Trial {
                    index: 2,
                    seed: 9_u64
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407)
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407),
                    candidate_limit: 12,
                },
            ]
        );
    }
}
