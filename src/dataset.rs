use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::RagasError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SingleTurnSample {
    pub user_input: String,
    pub response: String,
    pub retrieved_contexts: Vec<String>,
    pub reference: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl SingleTurnSample {
    pub fn new(
        _user_input: impl Into<String>,
        _response: impl Into<String>,
        _retrieved_contexts: Vec<String>,
    ) -> Self {
        unimplemented!("TEST-1.1.1: sample constructor is not implemented yet")
    }

    pub fn with_reference(self, _reference: impl Into<String>) -> Self {
        unimplemented!("TEST-1.1.1: sample reference setter is not implemented yet")
    }

    pub fn with_metadata(self, _key: impl Into<String>, _value: impl Into<String>) -> Self {
        unimplemented!("TEST-1.1.1: sample metadata setter is not implemented yet")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationDataset {
    samples: Vec<SingleTurnSample>,
}

impl EvaluationDataset {
    pub fn new(_samples: Vec<SingleTurnSample>) -> Result<Self, RagasError> {
        unimplemented!("TEST-1.1.2: dataset constructor is not implemented yet")
    }

    pub fn from_sample(_sample: SingleTurnSample) -> Result<Self, RagasError> {
        unimplemented!("TEST-1.1.2: single-sample dataset constructor is not implemented yet")
    }

    pub fn len(&self) -> usize {
        unimplemented!("TEST-1.1.2: dataset len is not implemented yet")
    }

    pub fn is_empty(&self) -> bool {
        unimplemented!("TEST-1.1.2: dataset empty check is not implemented yet")
    }

    pub fn iter(&self) -> impl Iterator<Item = &SingleTurnSample> {
        self.samples.iter()
    }

    pub fn samples(&self) -> &[SingleTurnSample] {
        unimplemented!("TEST-1.1.2: dataset samples accessor is not implemented yet")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_1_1_valid_sample_fields_are_preserved() {
        // SCEN-1.1.1 / AC1 / TEST-1.1.1
        let sample = SingleTurnSample::new(
            "What is Ragas?",
            "A framework for LLM evaluation.",
            vec!["Ragas evaluates LLM applications.".to_string()],
        )
        .with_reference("Ragas is an evaluation toolkit.")
        .with_metadata("source", "unit-test");

        assert_eq!(sample.user_input, "What is Ragas?");
        assert_eq!(sample.response, "A framework for LLM evaluation.");
        assert_eq!(sample.retrieved_contexts.len(), 1);
        assert_eq!(
            sample.reference.as_deref(),
            Some("Ragas is an evaluation toolkit.")
        );
        assert_eq!(sample.metadata.get("source").map(String::as_str), Some("unit-test"));
    }

    #[test]
    fn test_1_1_2_dataset_exposes_collection_helpers() {
        // SCEN-1.1.2 / AC2 / TEST-1.1.2
        let sample = SingleTurnSample::new(
            "Question",
            "Answer",
            vec!["Context".to_string()],
        );

        let dataset = EvaluationDataset::from_sample(sample).expect("valid dataset");

        assert_eq!(dataset.len(), 1);
        assert!(!dataset.is_empty());
        assert_eq!(dataset.iter().count(), 1);
        assert_eq!(dataset.samples().len(), 1);
    }

    #[test]
    fn test_1_1_3_validation_rejects_empty_and_invalid_samples() {
        // SCEN-1.1.3 / AC3 / TEST-1.1.3
        assert_eq!(
            EvaluationDataset::new(vec![]).unwrap_err(),
            RagasError::EmptyDataset
        );

        let invalid = SingleTurnSample::new("", "Answer", vec!["Context".to_string()]);
        assert_eq!(
            EvaluationDataset::new(vec![invalid]).unwrap_err(),
            RagasError::InvalidSample {
                index: 0,
                field: "user_input".to_string()
            }
        );

        let invalid = SingleTurnSample::new("Question", "Answer", vec![]);
        assert_eq!(
            EvaluationDataset::new(vec![invalid]).unwrap_err(),
            RagasError::InvalidSample {
                index: 0,
                field: "retrieved_contexts".to_string()
            }
        );
    }
}
