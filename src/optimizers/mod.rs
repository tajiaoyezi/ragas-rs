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
                let primary = generation_scores
                    .first()
                    .map(|(candidate, _)| candidate.clone())
                    .unwrap_or_else(|| best_candidate.clone());
                // Second elite parent so that crossover combines text from two
                // distinct population members; falls back to the primary when the
                // population has only one member.
                let secondary = generation_scores
                    .get(1)
                    .map(|(candidate, _)| candidate.clone())
                    .unwrap_or_else(|| primary.clone());
                population = (0..population_size)
                    .map(|_| {
                        // Cross the two elites' prompt TEXT, then mutate the
                        // recombined child via the caller's generator.
                        let crossed =
                            crossover_prompt_text(&primary, &secondary, next_seed(&mut rng_state));
                        generator.mutate(&crossed, next_seed(&mut rng_state))
                    })
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

/// Single-point crossover over prompt TEXT: split both parents into
/// whitespace-delimited tokens and recombine a prefix of `left` with a suffix
/// of `right` at a seeded crossover point. The child inherits `left`'s id/model
/// metadata; the caller's `mutate` is applied afterwards. The cut point is a
/// pure function of `seed`, so the same seed always yields the same child.
fn crossover_prompt_text(
    left: &OptimizationCandidate,
    right: &OptimizationCandidate,
    seed: u64,
) -> OptimizationCandidate {
    let left_tokens: Vec<&str> = left.prompt.split_whitespace().collect();
    let right_tokens: Vec<&str> = right.prompt.split_whitespace().collect();

    // Choose a cut index over the longer parent so both prefixes and suffixes
    // are reachable; empty parents degrade gracefully to an empty prompt.
    let span = left_tokens.len().max(right_tokens.len());
    let prompt = if span == 0 {
        String::new()
    } else {
        let cut = (seed % (span as u64 + 1)) as usize;
        let prefix = &left_tokens[..cut.min(left_tokens.len())];
        let suffix = &right_tokens[cut.min(right_tokens.len())..];
        prefix
            .iter()
            .chain(suffix.iter())
            .copied()
            .collect::<Vec<&str>>()
            .join(" ")
    };

    let mut child = OptimizationCandidate::new(format!("{}x{}", left.id, right.id), prompt);
    child.model = left.model.clone();
    child.parameters = left.parameters.clone();
    child
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

    /// Generator that evolves prompt TEXT toward a target token. Each mutation
    /// appends one token drawn from a tiny vocabulary (which includes the
    /// target token), so the population can genuinely discover the token over
    /// generations. The chosen token is a pure function of the seed, so the
    /// generator is deterministic.
    struct TokenRewardGenerator {
        vocab: Vec<&'static str>,
    }

    impl CandidateGenerator for TokenRewardGenerator {
        fn initial_candidates(&self) -> Vec<OptimizationCandidate> {
            // Initial prompts deliberately lack the target token so the
            // optimizer has to improve to find it.
            vec![
                OptimizationCandidate::new("p0", "respond clearly"),
                OptimizationCandidate::new("p1", "be concise"),
            ]
        }

        fn mutate(&self, candidate: &OptimizationCandidate, seed: u64) -> OptimizationCandidate {
            let token = self.vocab[(seed as usize) % self.vocab.len()];
            OptimizationCandidate::new(
                format!("{}-{}", candidate.id, seed % 1000),
                format!("{} {}", candidate.prompt, token),
            )
        }
    }

    /// Scores a candidate by counting occurrences of the target token in its
    /// prompt TEXT. The score is a real function of the candidate, not a
    /// constant, so improvements only come from better prompt text.
    struct TokenRewardObjective {
        target: &'static str,
    }

    impl ObjectiveMetric for TokenRewardObjective {
        fn name(&self) -> &str {
            "token_reward"
        }

        fn score(&self, candidate: &OptimizationCandidate) -> f64 {
            candidate
                .prompt
                .split_whitespace()
                .filter(|word| *word == self.target)
                .count() as f64
        }
    }

    fn initial_best_fitness(
        objective: &dyn ObjectiveMetric,
        generator: &dyn CandidateGenerator,
    ) -> f64 {
        generator
            .initial_candidates()
            .iter()
            .map(|candidate| objective.score(candidate))
            .fold(f64::NEG_INFINITY, f64::max)
    }

    #[test]
    fn test_genetic_optimizer_improves_token_reward_fitness() {
        // Real-behavior: with a scorer that rewards prompts containing a target
        // token, the evolved best fitness must improve over (or hold) the best
        // of the initial population, and the winning prompt must actually carry
        // the token -- proving fitness drives selection, not a constant.
        let generator = TokenRewardGenerator {
            vocab: vec!["grounded", "filler", "noise", "extra"],
        };
        let objective = TokenRewardObjective { target: "grounded" };

        let baseline = initial_best_fitness(&objective, &generator);
        assert_eq!(baseline, 0.0, "initial prompts must lack the target token");

        let mut optimizer = GeneticOptimizer::new(GeneticOptimizerConfig {
            seed: 2024,
            generations: 8,
            population_size: 6,
        });
        let result = optimizer.optimize(&objective, &generator);

        // (a) final best fitness >= initial best fitness.
        assert!(
            result.best_score >= baseline,
            "evolved best {} should be >= initial best {}",
            result.best_score,
            baseline
        );
        // Fitness is genuinely earned by prompt text, not asserted constant.
        assert!(
            result.best_score > baseline,
            "optimizer should have discovered the rewarded token"
        );
        assert!(
            result.best_candidate.prompt.contains("grounded"),
            "winning prompt {:?} must contain the rewarded token",
            result.best_candidate.prompt
        );
    }

    #[test]
    fn test_genetic_optimizer_is_reproducible_for_same_seed() {
        // Reproducibility: identical seed/config => byte-identical result and
        // trajectory across two independent runs.
        let generator = TokenRewardGenerator {
            vocab: vec!["grounded", "filler", "noise", "extra"],
        };
        let objective = TokenRewardObjective { target: "grounded" };
        let config = GeneticOptimizerConfig {
            seed: 99,
            generations: 6,
            population_size: 5,
        };

        let first = GeneticOptimizer::new(config).optimize(&objective, &generator);
        let second = GeneticOptimizer::new(config).optimize(&objective, &generator);

        assert_eq!(first.best_candidate, second.best_candidate);
        assert_eq!(first.best_score, second.best_score);
        assert_eq!(first.history, second.history);
    }

    #[test]
    fn test_genetic_optimizer_uses_different_seeds_for_distinct_trajectories() {
        // Different seeds should be able to explore differently: the histories
        // are not forced to match, proving the seed actually drives evolution.
        let generator = TokenRewardGenerator {
            vocab: vec!["grounded", "filler", "noise", "extra"],
        };
        let objective = TokenRewardObjective { target: "grounded" };

        let run = |seed: u64| {
            GeneticOptimizer::new(GeneticOptimizerConfig {
                seed,
                generations: 5,
                population_size: 5,
            })
            .optimize(&objective, &generator)
        };

        let a = run(1);
        let b = run(7);
        assert_ne!(
            a.history, b.history,
            "different seeds should yield different trajectories"
        );
    }

    #[test]
    fn test_crossover_prompt_text_recombines_words() {
        // The crossover helper must genuinely splice two parents' tokens, not
        // echo one parent. With a seed that lands on an interior cut point the
        // child carries a prefix from `left` and a suffix from `right`.
        let left = OptimizationCandidate::new("L", "alpha beta gamma");
        let right = OptimizationCandidate::new("R", "one two three");

        // span = 3, seed % 4 picks the cut. Seed 2 -> cut 2 -> "alpha beta three".
        let child = crossover_prompt_text(&left, &right, 2);
        assert_eq!(child.prompt, "alpha beta three");
        assert_eq!(child.id, "LxR");

        // Cut at 0 yields the full right parent; cut at span yields full left.
        assert_eq!(crossover_prompt_text(&left, &right, 0).prompt, "one two three");
        assert_eq!(
            crossover_prompt_text(&left, &right, 3).prompt,
            "alpha beta gamma"
        );
    }

    #[test]
    fn test_crossover_prompt_text_handles_empty_prompts() {
        // Adversarial: empty parents must not panic and produce an empty prompt.
        let empty_left = OptimizationCandidate::new("L", "");
        let empty_right = OptimizationCandidate::new("R", "");
        assert_eq!(crossover_prompt_text(&empty_left, &empty_right, 0).prompt, "");
        assert_eq!(crossover_prompt_text(&empty_left, &empty_right, 5).prompt, "");

        // One empty parent still recombines without losing the non-empty side.
        let filled = OptimizationCandidate::new("F", "keep this");
        let crossed = crossover_prompt_text(&empty_left, &filled, 0);
        assert_eq!(crossed.prompt, "keep this");
    }
}
