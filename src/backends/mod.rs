use std::collections::{BTreeMap, BTreeSet};

use crate::{EvaluationDataset, EvaluationSample, RagasError, SingleTurnSample};

pub trait DatasetBackend {
    fn save(
        &mut self,
        name: &str,
        dataset: &EvaluationDataset<EvaluationSample>,
    ) -> Result<(), RagasError>;
    fn load(&self, name: &str) -> Result<EvaluationDataset<EvaluationSample>, RagasError>;
    fn list(&self) -> Vec<String>;
    fn delete(&mut self, name: &str) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryDatasetBackend {
    datasets: BTreeMap<String, EvaluationDataset<EvaluationSample>>,
}

impl InMemoryDatasetBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DatasetBackend for InMemoryDatasetBackend {
    fn save(
        &mut self,
        name: &str,
        dataset: &EvaluationDataset<EvaluationSample>,
    ) -> Result<(), RagasError> {
        self.datasets.insert(name.to_string(), dataset.clone());
        Ok(())
    }

    fn load(&self, name: &str) -> Result<EvaluationDataset<EvaluationSample>, RagasError> {
        self.datasets
            .get(name)
            .cloned()
            .ok_or_else(|| not_found(name))
    }

    fn list(&self) -> Vec<String> {
        self.datasets.keys().cloned().collect()
    }

    fn delete(&mut self, name: &str) -> bool {
        self.datasets.remove(name).is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct JsonlDatasetBackend {
    documents: BTreeMap<String, String>,
}

impl JsonlDatasetBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DatasetBackend for JsonlDatasetBackend {
    fn save(
        &mut self,
        name: &str,
        dataset: &EvaluationDataset<EvaluationSample>,
    ) -> Result<(), RagasError> {
        self.documents
            .insert(name.to_string(), dataset.to_jsonl_string()?);
        Ok(())
    }

    fn load(&self, name: &str) -> Result<EvaluationDataset<EvaluationSample>, RagasError> {
        let document = self.documents.get(name).ok_or_else(|| not_found(name))?;
        EvaluationDataset::<EvaluationSample>::from_jsonl_str(document)
    }

    fn list(&self) -> Vec<String> {
        self.documents.keys().cloned().collect()
    }

    fn delete(&mut self, name: &str) -> bool {
        self.documents.remove(name).is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CsvDatasetBackend {
    documents: BTreeMap<String, String>,
}

impl CsvDatasetBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DatasetBackend for CsvDatasetBackend {
    fn save(
        &mut self,
        name: &str,
        dataset: &EvaluationDataset<EvaluationSample>,
    ) -> Result<(), RagasError> {
        self.documents
            .insert(name.to_string(), dataset_to_csv_string(dataset)?);
        Ok(())
    }

    fn load(&self, name: &str) -> Result<EvaluationDataset<EvaluationSample>, RagasError> {
        let document = self.documents.get(name).ok_or_else(|| not_found(name))?;
        let single_turn_dataset = EvaluationDataset::<SingleTurnSample>::from_csv_str(document)?;
        EvaluationDataset::<EvaluationSample>::from_samples(
            single_turn_dataset
                .samples()
                .iter()
                .cloned()
                .map(EvaluationSample::SingleTurn)
                .collect(),
        )
    }

    fn list(&self) -> Vec<String> {
        self.documents.keys().cloned().collect()
    }

    fn delete(&mut self, name: &str) -> bool {
        self.documents.remove(name).is_some()
    }
}

fn not_found(name: &str) -> RagasError {
    RagasError::DatasetIo {
        message: format!("dataset backend entry not found: {name}"),
    }
}

fn dataset_to_csv_string(
    dataset: &EvaluationDataset<EvaluationSample>,
) -> Result<String, RagasError> {
    let mut metadata_keys = BTreeSet::new();
    let mut single_turn_samples = Vec::with_capacity(dataset.len());
    for sample in dataset.samples() {
        let EvaluationSample::SingleTurn(sample) = sample else {
            return Err(dataset_io_error(
                "CSV backend only supports single-turn samples",
            ));
        };
        metadata_keys.extend(sample.metadata.keys().cloned());
        single_turn_samples.push(sample);
    }

    let mut headers = vec![
        "user_input".to_string(),
        "response".to_string(),
        "retrieved_contexts".to_string(),
        "reference".to_string(),
    ];
    headers.extend(metadata_keys.iter().map(|key| format!("metadata.{key}")));

    let mut writer = csv::Writer::from_writer(Vec::new());
    writer
        .write_record(&headers)
        .map_err(|error| dataset_io_error(format!("CSV header write failed: {error}")))?;

    for sample in single_turn_samples {
        let mut record = vec![
            sample.user_input.clone(),
            sample.response.clone(),
            serde_json::to_string(&sample.retrieved_contexts).map_err(|error| {
                dataset_io_error(format!("retrieved_contexts JSON encode failed: {error}"))
            })?,
            sample.reference.clone().unwrap_or_default(),
        ];
        for key in &metadata_keys {
            record.push(sample.metadata.get(key).cloned().unwrap_or_default());
        }
        writer
            .write_record(&record)
            .map_err(|error| dataset_io_error(format!("CSV row write failed: {error}")))?;
    }

    let bytes = writer
        .into_inner()
        .map_err(|error| dataset_io_error(format!("CSV writer finalize failed: {error}")))?;
    String::from_utf8(bytes)
        .map_err(|error| dataset_io_error(format!("CSV writer emitted invalid UTF-8: {error}")))
}

fn dataset_io_error(message: impl Into<String>) -> RagasError {
    RagasError::DatasetIo {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(question: &str, response: &str, context: &str) -> EvaluationSample {
        EvaluationSample::SingleTurn(
            SingleTurnSample::new(question, response, vec![context.to_string()])
                .with_reference(response)
                .with_metadata("source", "unit-test"),
        )
    }

    fn fixture_dataset() -> EvaluationDataset<EvaluationSample> {
        EvaluationDataset::from_samples(vec![
            sample(
                "What is RAG?",
                "Retrieval augmented generation.",
                "RAG uses retrieval.",
            ),
            sample(
                "What is eval?",
                "Evaluation scores answers.",
                "Metrics score responses.",
            ),
        ])
        .expect("fixture dataset")
    }

    #[test]
    fn test_14_1_1_backend_trait_supports_save_load_list_and_delete() {
        // SCEN-14.1.1 / AC1 / TEST-14.1.1
        let mut backend: Box<dyn DatasetBackend> = Box::new(InMemoryDatasetBackend::new());
        let dataset = fixture_dataset();

        backend.save("runs/run-1", &dataset).expect("save dataset");
        assert_eq!(backend.list(), vec!["runs/run-1".to_string()]);

        let loaded = backend.load("runs/run-1").expect("load dataset");
        assert_eq!(loaded.samples(), dataset.samples());

        assert!(backend.delete("runs/run-1"));
        assert!(backend.load("runs/run-1").is_err());
        assert!(backend.list().is_empty());
    }

    #[test]
    fn test_14_1_2_in_memory_backend_is_deterministic_for_tests() {
        // SCEN-14.1.2 / AC2 / TEST-14.1.2
        let mut backend = InMemoryDatasetBackend::new();
        let dataset = fixture_dataset();

        backend.save("zeta", &dataset).expect("save zeta");
        backend.save("alpha", &dataset).expect("save alpha");
        backend.save("alpha", &dataset).expect("overwrite alpha");

        assert_eq!(
            backend.list(),
            vec!["alpha".to_string(), "zeta".to_string()]
        );
        assert_eq!(backend.load("alpha").expect("alpha").samples().len(), 2);
    }

    #[test]
    fn test_14_1_3_jsonl_and_csv_local_backends_preserve_dataset_schema() {
        // SCEN-14.1.3 / AC3 / TEST-14.1.3
        let dataset = fixture_dataset();

        let mut jsonl = JsonlDatasetBackend::new();
        jsonl.save("fixture", &dataset).expect("save jsonl");
        let jsonl_roundtrip = jsonl.load("fixture").expect("load jsonl");
        assert_eq!(jsonl_roundtrip.samples(), dataset.samples());

        let mut csv = CsvDatasetBackend::new();
        csv.save("fixture", &dataset).expect("save csv");
        let csv_roundtrip = csv.load("fixture").expect("load csv");
        assert_eq!(csv_roundtrip.samples(), dataset.samples());
    }
}
