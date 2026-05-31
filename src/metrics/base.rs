use async_trait::async_trait;

use crate::{
    MetricRequirements, MetricResult, MultiTurnSample, RagasError, SampleField, SingleTurnSample,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricSampleKind {
    SingleTurn,
    MultiTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricProviderRequirement {
    Llm,
    Embedding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricMetadata {
    name: String,
    sample_kind: MetricSampleKind,
    required_fields: Vec<SampleField>,
    provider_requirements: Vec<MetricProviderRequirement>,
}

impl MetricMetadata {
    pub fn new(name: impl Into<String>, sample_kind: MetricSampleKind) -> Self {
        Self {
            name: name.into(),
            sample_kind,
            required_fields: Vec::new(),
            provider_requirements: Vec::new(),
        }
    }

    pub fn with_required_fields(mut self, fields: Vec<SampleField>) -> Self {
        self.required_fields = fields;
        self
    }

    pub fn with_provider_requirements(
        mut self,
        requirements: Vec<MetricProviderRequirement>,
    ) -> Self {
        self.provider_requirements = requirements;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sample_kind(&self) -> MetricSampleKind {
        self.sample_kind
    }

    pub fn required_fields(&self) -> &[SampleField] {
        &self.required_fields
    }

    pub fn provider_requirements(&self) -> &[MetricProviderRequirement] {
        &self.provider_requirements
    }

    pub fn requires_llm(&self) -> bool {
        self.provider_requirements
            .contains(&MetricProviderRequirement::Llm)
    }

    pub fn requires_embedding(&self) -> bool {
        self.provider_requirements
            .contains(&MetricProviderRequirement::Embedding)
    }

    pub fn to_requirements(&self) -> MetricRequirements {
        MetricRequirements::new(&self.name, self.required_fields.clone())
    }
}

#[async_trait]
pub trait SingleTurnMetric: Send + Sync {
    fn metadata(&self) -> MetricMetadata;

    async fn score_single(&self, sample: &SingleTurnSample) -> Result<MetricResult, RagasError>;

    async fn score_batch(
        &self,
        samples: &[SingleTurnSample],
    ) -> Result<Vec<MetricResult>, RagasError> {
        let mut results = Vec::with_capacity(samples.len());
        for sample in samples {
            results.push(self.score_single(sample).await?);
        }
        Ok(results)
    }
}

#[async_trait]
pub trait MultiTurnMetric: Send + Sync {
    fn metadata(&self) -> MetricMetadata;

    async fn score_multi_turn(&self, sample: &MultiTurnSample) -> Result<MetricResult, RagasError>;

    async fn score_batch(
        &self,
        samples: &[MultiTurnSample],
    ) -> Result<Vec<MetricResult>, RagasError> {
        let mut results = Vec::with_capacity(samples.len());
        for sample in samples {
            results.push(self.score_multi_turn(sample).await?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use crate::MetricValue;

    struct ProviderAwareSingleTurnMetric;

    #[async_trait]
    impl SingleTurnMetric for ProviderAwareSingleTurnMetric {
        fn metadata(&self) -> MetricMetadata {
            MetricMetadata::new("judge_similarity", MetricSampleKind::SingleTurn)
                .with_provider_requirements(vec![
                    MetricProviderRequirement::Llm,
                    MetricProviderRequirement::Embedding,
                ])
        }

        async fn score_single(
            &self,
            _sample: &SingleTurnSample,
        ) -> Result<MetricResult, RagasError> {
            Ok(MetricResult::success(
                "judge_similarity",
                MetricValue::numeric(1.0),
            ))
        }
    }

    struct MultiTurnOnlyMetric;

    #[async_trait]
    impl MultiTurnMetric for MultiTurnOnlyMetric {
        fn metadata(&self) -> MetricMetadata {
            MetricMetadata::new("conversation_goal_accuracy", MetricSampleKind::MultiTurn)
                .with_provider_requirements(vec![MetricProviderRequirement::Llm])
        }

        async fn score_multi_turn(
            &self,
            _sample: &MultiTurnSample,
        ) -> Result<MetricResult, RagasError> {
            Ok(MetricResult::success(
                "conversation_goal_accuracy",
                MetricValue::numeric(1.0),
            ))
        }
    }

    #[test]
    fn test_9_1_1_metric_traits_distinguish_turn_kind_and_provider_requirements() {
        // SCEN-9.1.1 / AC1 / TEST-9.1.1
        let single = ProviderAwareSingleTurnMetric.metadata();
        assert_eq!(single.sample_kind(), MetricSampleKind::SingleTurn);
        assert!(single.requires_llm());
        assert!(single.requires_embedding());

        let multi = MultiTurnOnlyMetric.metadata();
        assert_eq!(multi.sample_kind(), MetricSampleKind::MultiTurn);
        assert!(multi.requires_llm());
        assert!(!multi.requires_embedding());
    }

    struct RecordingMetric {
        seen: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl SingleTurnMetric for RecordingMetric {
        fn metadata(&self) -> MetricMetadata {
            MetricMetadata::new("recording", MetricSampleKind::SingleTurn)
        }

        async fn score_single(
            &self,
            sample: &SingleTurnSample,
        ) -> Result<MetricResult, RagasError> {
            self.seen
                .lock()
                .expect("seen")
                .push(sample.user_input.clone());
            Ok(MetricResult::success(
                "recording",
                MetricValue::numeric(sample.user_input.len() as f64),
            ))
        }
    }

    #[tokio::test]
    async fn test_9_1_2_batch_scoring_hooks_default_to_per_sample_behavior() {
        // SCEN-9.1.2 / AC2 / TEST-9.1.2
        let seen = Arc::new(Mutex::new(Vec::new()));
        let metric = RecordingMetric { seen: seen.clone() };
        let samples = vec![
            SingleTurnSample::new("a", "response", vec![]),
            SingleTurnSample::new("bb", "response", vec![]),
            SingleTurnSample::new("ccc", "response", vec![]),
        ];

        let results = metric.score_batch(&samples).await.expect("batch results");

        assert_eq!(
            *seen.lock().expect("seen"),
            vec!["a".to_string(), "bb".to_string(), "ccc".to_string()]
        );
        assert_eq!(results.len(), 3);
        assert_eq!(
            results[2]
                .value
                .clone()
                .and_then(|value| value.as_numeric()),
            Some(3.0)
        );
    }

    #[test]
    fn test_9_1_3_metric_metadata_declares_required_sample_fields() {
        // SCEN-9.1.3 / AC3 / TEST-9.1.3
        let metadata = MetricMetadata::new("answer_correctness", MetricSampleKind::SingleTurn)
            .with_required_fields(vec![SampleField::UserInput, SampleField::Response]);

        assert_eq!(
            metadata.required_fields(),
            &[SampleField::UserInput, SampleField::Response]
        );

        let requirements = metadata.to_requirements();
        assert_eq!(requirements.metric_name(), "answer_correctness");
        assert_eq!(
            requirements.required_fields(),
            &[SampleField::UserInput, SampleField::Response]
        );
    }
}
