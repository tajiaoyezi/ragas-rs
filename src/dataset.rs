use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{MultiTurnSample, RagasError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SingleTurnSample {
    pub user_input: String,
    pub response: String,
    pub retrieved_contexts: Vec<String>,
    pub reference: Option<String>,
    /// Ground-truth reference contexts (used by the non-LLM context precision/recall metrics).
    /// Optional: omitted from JSONL when empty, so existing datasets are unaffected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_contexts: Vec<String>,
    /// Retrieved / reference context IDs for the ID-based context metrics. Optional; omitted from
    /// JSONL when empty (backward compatible).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retrieved_context_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_context_ids: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl SingleTurnSample {
    pub fn new(
        user_input: impl Into<String>,
        response: impl Into<String>,
        retrieved_contexts: Vec<String>,
    ) -> Self {
        Self {
            user_input: user_input.into(),
            response: response.into(),
            retrieved_contexts,
            reference: None,
            reference_contexts: Vec::new(),
            retrieved_context_ids: Vec::new(),
            reference_context_ids: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    pub fn with_reference_contexts(mut self, reference_contexts: Vec<String>) -> Self {
        self.reference_contexts = reference_contexts;
        self
    }

    pub fn with_context_ids(
        mut self,
        retrieved_context_ids: Vec<String>,
        reference_context_ids: Vec<String>,
    ) -> Self {
        self.retrieved_context_ids = retrieved_context_ids;
        self.reference_context_ids = reference_context_ids;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "sample_type", rename_all = "snake_case")]
pub enum EvaluationSample {
    SingleTurn(SingleTurnSample),
    MultiTurn(MultiTurnSample),
}

impl From<SingleTurnSample> for EvaluationSample {
    fn from(sample: SingleTurnSample) -> Self {
        Self::SingleTurn(sample)
    }
}

impl From<MultiTurnSample> for EvaluationSample {
    fn from(sample: MultiTurnSample) -> Self {
        Self::MultiTurn(sample)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationDataset<T = SingleTurnSample> {
    samples: Vec<T>,
    metadata: HashMap<String, String>,
}

impl<T> EvaluationDataset<T> {
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.samples.iter()
    }

    pub fn samples(&self) -> &[T] {
        &self.samples
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

impl EvaluationDataset<SingleTurnSample> {
    pub fn new(samples: Vec<SingleTurnSample>) -> Result<Self, RagasError> {
        if samples.is_empty() {
            return Err(RagasError::EmptyDataset);
        }

        for (index, sample) in samples.iter().enumerate() {
            validate_sample(index, sample)?;
        }

        Ok(Self {
            samples,
            metadata: HashMap::new(),
        })
    }

    pub fn from_sample(sample: SingleTurnSample) -> Result<Self, RagasError> {
        Self::new(vec![sample])
    }

    pub fn from_csv_str(input: &str) -> Result<Self, RagasError> {
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(input.as_bytes());
        let headers = reader
            .headers()
            .map_err(|error| dataset_io_error(format!("CSV header parse failed: {error}")))?
            .clone();

        let user_input_idx = required_header(&headers, "user_input")?;
        let response_idx = required_header(&headers, "response")?;
        let contexts_idx = required_header(&headers, "retrieved_contexts")?;
        let reference_idx = header_index(&headers, "reference");
        let metadata_columns = headers
            .iter()
            .enumerate()
            .filter_map(|(index, header)| {
                header
                    .strip_prefix("metadata.")
                    .map(|key| (index, key.to_string()))
            })
            .filter(|(_, key)| !key.is_empty())
            .collect::<Vec<_>>();

        let mut samples = Vec::new();
        for (row_index, record) in reader.records().enumerate() {
            let row_number = row_index + 2;
            let record = record.map_err(|error| {
                dataset_io_error(format!("CSV row {row_number} parse failed: {error}"))
            })?;
            let user_input = required_cell(&record, user_input_idx, "user_input", row_number)?;
            let response = required_cell(&record, response_idx, "response", row_number)?;
            let retrieved_contexts = parse_retrieved_contexts(
                required_cell(&record, contexts_idx, "retrieved_contexts", row_number)?,
                row_number,
            )?;
            let mut sample = SingleTurnSample::new(user_input, response, retrieved_contexts);

            if let Some(index) = reference_idx
                && let Some(reference) = optional_cell(&record, index)
            {
                sample = sample.with_reference(reference);
            }
            for (index, key) in &metadata_columns {
                if let Some(value) = optional_cell(&record, *index) {
                    sample = sample.with_metadata(key, value);
                }
            }

            validate_sample(row_index, &sample)?;
            samples.push(sample);
        }

        Self::new(samples)
    }
}

impl EvaluationDataset<EvaluationSample> {
    pub fn from_samples(samples: Vec<EvaluationSample>) -> Result<Self, RagasError> {
        if samples.is_empty() {
            return Err(RagasError::EmptyDataset);
        }
        for (index, sample) in samples.iter().enumerate() {
            validate_evaluation_sample(index, sample)?;
        }

        Ok(Self {
            samples,
            metadata: HashMap::new(),
        })
    }

    pub fn from_jsonl_str(input: &str) -> Result<Self, RagasError> {
        let mut samples = Vec::new();
        for (line_index, line) in input.lines().enumerate() {
            let line_number = line_index + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let sample = serde_json::from_str::<EvaluationSample>(trimmed).map_err(|error| {
                dataset_io_error(format!("JSONL line {line_number} parse failed: {error}"))
            })?;
            validate_evaluation_sample(line_index, &sample)?;
            samples.push(sample);
        }

        Self::from_samples(samples)
    }

    pub fn to_jsonl_string(&self) -> Result<String, RagasError> {
        let mut output = String::new();
        for sample in &self.samples {
            let line = serde_json::to_string(sample)
                .map_err(|error| dataset_io_error(format!("JSONL serialize failed: {error}")))?;
            output.push_str(&line);
            output.push('\n');
        }
        Ok(output)
    }
}

#[derive(Debug, Default)]
pub struct EvaluationDatasetBuilder {
    samples: Vec<EvaluationSample>,
    metadata: HashMap<String, String>,
}

impl EvaluationDatasetBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn add_sample(mut self, sample: impl Into<EvaluationSample>) -> Self {
        self.samples.push(sample.into());
        self
    }

    pub fn add_single_turn(self, sample: SingleTurnSample) -> Self {
        self.add_sample(sample)
    }

    pub fn add_multi_turn(self, sample: MultiTurnSample) -> Self {
        self.add_sample(sample)
    }

    pub fn build(self) -> Result<EvaluationDataset<EvaluationSample>, RagasError> {
        if self.samples.is_empty() {
            return Err(RagasError::EmptyDataset);
        }
        for (index, sample) in self.samples.iter().enumerate() {
            validate_evaluation_sample(index, sample)?;
        }

        Ok(EvaluationDataset {
            samples: self.samples,
            metadata: self.metadata,
        })
    }
}

fn dataset_io_error(message: impl Into<String>) -> RagasError {
    RagasError::DatasetIo {
        message: message.into(),
    }
}

fn header_index(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|header| header == name)
}

fn required_header(headers: &csv::StringRecord, name: &str) -> Result<usize, RagasError> {
    header_index(headers, name)
        .ok_or_else(|| dataset_io_error(format!("CSV missing required column: {name}")))
}

fn required_cell(
    record: &csv::StringRecord,
    index: usize,
    field: &str,
    row_number: usize,
) -> Result<String, RagasError> {
    let value = record.get(index).unwrap_or("").trim();
    if value.is_empty() {
        return Err(dataset_io_error(format!(
            "CSV row {row_number} missing required value: {field}"
        )));
    }
    Ok(value.to_string())
}

fn optional_cell(record: &csv::StringRecord, index: usize) -> Option<String> {
    record
        .get(index)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_retrieved_contexts(value: String, row_number: usize) -> Result<Vec<String>, RagasError> {
    let trimmed = value.trim();
    if trimmed.starts_with('[') {
        let contexts = serde_json::from_str::<Vec<String>>(trimmed).map_err(|error| {
            dataset_io_error(format!(
                "CSV row {row_number} retrieved_contexts JSON parse failed: {error}"
            ))
        })?;
        return Ok(contexts);
    }

    Ok(trimmed
        .split('|')
        .map(str::trim)
        .filter(|context| !context.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn validate_sample(index: usize, sample: &SingleTurnSample) -> Result<(), RagasError> {
    if sample.user_input.trim().is_empty() {
        return Err(RagasError::InvalidSample {
            index,
            field: "user_input".to_string(),
        });
    }
    if sample.response.trim().is_empty() {
        return Err(RagasError::InvalidSample {
            index,
            field: "response".to_string(),
        });
    }
    if sample
        .retrieved_contexts
        .iter()
        .all(|context| context.trim().is_empty())
    {
        return Err(RagasError::InvalidSample {
            index,
            field: "retrieved_contexts".to_string(),
        });
    }
    Ok(())
}

fn validate_evaluation_sample(index: usize, sample: &EvaluationSample) -> Result<(), RagasError> {
    match sample {
        EvaluationSample::SingleTurn(sample) => validate_sample(index, sample),
        EvaluationSample::MultiTurn(sample) => {
            if sample.messages.is_empty() {
                return Err(RagasError::InvalidSample {
                    index,
                    field: "messages".to_string(),
                });
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, MultiTurnSample};

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
        assert_eq!(
            sample.metadata.get("source").map(String::as_str),
            Some("unit-test")
        );
    }

    #[test]
    fn reference_contexts_round_trip_through_jsonl_and_default_when_absent() {
        let sample = SingleTurnSample::new("q", "a", vec!["ctx".to_string()])
            .with_reference_contexts(vec!["ref-ctx-1".to_string(), "ref-ctx-2".to_string()]);
        assert_eq!(sample.reference_contexts.len(), 2);

        let dataset = EvaluationDatasetBuilder::new()
            .add_single_turn(sample)
            .build()
            .expect("dataset");
        let jsonl = dataset.to_jsonl_string().expect("jsonl");
        assert!(jsonl.contains("reference_contexts"));
        let restored =
            EvaluationDataset::<EvaluationSample>::from_jsonl_str(&jsonl).expect("read jsonl");
        assert_eq!(restored.samples(), dataset.samples());

        // A line WITHOUT reference_contexts deserializes to an empty vec (backward compatible).
        let legacy = r#"{"sample_type":"single_turn","user_input":"q","response":"a","retrieved_contexts":["c"],"metadata":{}}"#;
        let legacy_dataset =
            EvaluationDataset::<EvaluationSample>::from_jsonl_str(legacy).expect("legacy");
        match &legacy_dataset.samples()[0] {
            EvaluationSample::SingleTurn(single) => assert!(single.reference_contexts.is_empty()),
            other => panic!("expected single turn, got {other:?}"),
        }
    }

    #[test]
    fn context_ids_round_trip_through_jsonl() {
        let sample = SingleTurnSample::new("q", "a", vec!["ctx".to_string()]).with_context_ids(
            vec!["r1".to_string(), "r2".to_string()],
            vec!["r1".to_string()],
        );
        let dataset = EvaluationDatasetBuilder::new()
            .add_single_turn(sample)
            .build()
            .expect("dataset");
        let jsonl = dataset.to_jsonl_string().expect("jsonl");
        assert!(jsonl.contains("retrieved_context_ids"));
        assert!(jsonl.contains("reference_context_ids"));
        let restored =
            EvaluationDataset::<EvaluationSample>::from_jsonl_str(&jsonl).expect("read jsonl");
        assert_eq!(restored.samples(), dataset.samples());
    }

    #[test]
    fn test_1_1_2_dataset_exposes_collection_helpers() {
        // SCEN-1.1.2 / AC2 / TEST-1.1.2
        let sample = SingleTurnSample::new("Question", "Answer", vec!["Context".to_string()]);

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

    #[test]
    fn test_5_2_1_jsonl_roundtrip_supports_single_and_multi_turn_samples() {
        // SCEN-5.2.1 / AC1 / TEST-5.2.1
        let dataset = EvaluationDatasetBuilder::new()
            .add_single_turn(
                SingleTurnSample::new("What is ragas?", "An eval framework.", vec!["ctx".into()])
                    .with_reference("Ragas evaluates LLM apps.")
                    .with_metadata("row_id", "single-1"),
            )
            .add_multi_turn(
                MultiTurnSample::new(vec![
                    Message::user("Can you inspect this?"),
                    Message::assistant("I can inspect it."),
                ])
                .with_reference("The assistant should answer the user.")
                .with_metadata("row_id", "multi-1"),
            )
            .build()
            .expect("mixed dataset");

        let jsonl = dataset.to_jsonl_string().expect("serialize jsonl");
        assert_eq!(jsonl.lines().count(), 2);

        let roundtrip =
            EvaluationDataset::<EvaluationSample>::from_jsonl_str(&jsonl).expect("read jsonl");

        assert_eq!(roundtrip.samples(), dataset.samples());
    }

    #[test]
    fn test_5_2_2_csv_import_maps_required_columns_and_reports_clear_errors() {
        // SCEN-5.2.2 / AC2 / TEST-5.2.2
        let csv = "user_input,response,retrieved_contexts,reference,metadata.source\nWhat?,Answer,ctx one|ctx two,Reference,fixture\n";
        let dataset = EvaluationDataset::from_csv_str(csv).expect("csv dataset");
        let sample = &dataset.samples()[0];

        assert_eq!(sample.user_input, "What?");
        assert_eq!(sample.response, "Answer");
        assert_eq!(sample.retrieved_contexts, vec!["ctx one", "ctx two"]);
        assert_eq!(sample.reference.as_deref(), Some("Reference"));
        assert_eq!(
            sample.metadata.get("source").map(String::as_str),
            Some("fixture")
        );

        let error = EvaluationDataset::from_csv_str("user_input,response\nWhat?,Answer\n")
            .expect_err("missing retrieved_contexts column");
        assert!(
            error
                .to_string()
                .contains("CSV missing required column: retrieved_contexts"),
            "{error}"
        );
    }

    #[test]
    fn test_5_2_3_dataset_builder_preserves_sample_order_and_metadata() {
        // SCEN-5.2.3 / AC3 / TEST-5.2.3
        let dataset = EvaluationDatasetBuilder::new()
            .with_metadata("name", "ordered-fixture")
            .add_single_turn(SingleTurnSample::new("first", "a1", vec!["c1".into()]))
            .add_multi_turn(MultiTurnSample::new(vec![Message::user("second")]))
            .add_single_turn(
                SingleTurnSample::new("third", "a3", vec!["c3".into()])
                    .with_metadata("sample", "third"),
            )
            .build()
            .expect("ordered dataset");

        assert_eq!(
            dataset.metadata().get("name").map(String::as_str),
            Some("ordered-fixture")
        );
        assert!(matches!(
            &dataset.samples()[0],
            EvaluationSample::SingleTurn(sample) if sample.user_input == "first"
        ));
        assert!(matches!(
            &dataset.samples()[1],
            EvaluationSample::MultiTurn(sample) if sample.messages[0].content == "second"
        ));
        assert!(matches!(
            &dataset.samples()[2],
            EvaluationSample::SingleTurn(sample)
                if sample.metadata.get("sample").map(String::as_str) == Some("third")
        ));
    }
}
