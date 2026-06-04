use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{EvaluationDataset, EvaluationSample, RagasError, SingleTurnSample};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GDriveAuthMode {
    ServiceAccount,
    OAuth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GDriveBackendConfig {
    pub folder_id: String,
    pub credentials_env: String,
    pub service_account_env: String,
    pub token_env: String,
    pub token_default: String,
    pub scopes: Vec<String>,
    pub auth_modes: Vec<GDriveAuthMode>,
    pub datasets_folder_name: String,
    pub experiments_folder_name: String,
}

impl GDriveBackendConfig {
    pub fn new(folder_id: impl Into<String>) -> Self {
        Self {
            folder_id: folder_id.into(),
            credentials_env: "GDRIVE_CREDENTIALS_PATH".to_string(),
            service_account_env: "GDRIVE_SERVICE_ACCOUNT_PATH".to_string(),
            token_env: "GDRIVE_TOKEN_PATH".to_string(),
            token_default: "token.json".to_string(),
            scopes: vec![
                "https://www.googleapis.com/auth/drive".to_string(),
                "https://www.googleapis.com/auth/spreadsheets".to_string(),
            ],
            auth_modes: vec![GDriveAuthMode::ServiceAccount, GDriveAuthMode::OAuth],
            datasets_folder_name: "datasets".to_string(),
            experiments_folder_name: "experiments".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DiskCacheRecord {
    key: String,
    value: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct DiskCacheCompatibility {
    entries: BTreeMap<String, Vec<u8>>,
    directory: Option<PathBuf>,
}

impl DiskCacheCompatibility {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(cache_dir: impl AsRef<Path>) -> Result<Self, RagasError> {
        let directory = cache_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&directory).map_err(|error| {
            dataset_io_error(format!(
                "disk cache directory create failed for {}: {error}",
                directory.display()
            ))
        })?;
        let entries = load_disk_cache_entries(&directory)?;
        Ok(Self {
            entries,
            directory: Some(directory),
        })
    }

    pub fn set(&mut self, key: impl Into<String>, value: Vec<u8>) -> Result<(), RagasError> {
        let key = key.into();
        if let Some(directory) = &self.directory {
            write_disk_cache_record(directory, &key, &value)?;
        }
        self.entries.insert(key, value);
        Ok(())
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn put(&mut self, key: impl Into<String>, value: Vec<u8>) {
        self.entries.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.entries.get(key).cloned()
    }

    pub fn delete(&mut self, key: &str) -> bool {
        let removed = self.entries.remove(key).is_some();
        if removed && let Some(directory) = &self.directory {
            let path = disk_cache_record_path(directory, key);
            match std::fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => {}
            }
        }
        removed
    }

    pub fn keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

fn load_disk_cache_entries(directory: &Path) -> Result<BTreeMap<String, Vec<u8>>, RagasError> {
    let mut entries = BTreeMap::new();
    for entry in std::fs::read_dir(directory).map_err(|error| {
        dataset_io_error(format!(
            "disk cache directory read failed for {}: {error}",
            directory.display()
        ))
    })? {
        let entry =
            entry.map_err(|error| dataset_io_error(format!("disk cache entry failed: {error}")))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let bytes = std::fs::read(&path).map_err(|error| {
            dataset_io_error(format!(
                "disk cache record read failed for {}: {error}",
                path.display()
            ))
        })?;
        let record: DiskCacheRecord = serde_json::from_slice(&bytes).map_err(|error| {
            dataset_io_error(format!(
                "disk cache record parse failed for {}: {error}",
                path.display()
            ))
        })?;
        entries.insert(record.key, record.value);
    }
    Ok(entries)
}

fn write_disk_cache_record(directory: &Path, key: &str, value: &[u8]) -> Result<(), RagasError> {
    std::fs::create_dir_all(directory).map_err(|error| {
        dataset_io_error(format!(
            "disk cache directory create failed for {}: {error}",
            directory.display()
        ))
    })?;
    let path = disk_cache_record_path(directory, key);
    let record = DiskCacheRecord {
        key: key.to_string(),
        value: value.to_vec(),
    };
    let bytes = serde_json::to_vec(&record).map_err(|error| {
        dataset_io_error(format!(
            "disk cache record encode failed for key {key}: {error}"
        ))
    })?;
    std::fs::write(&path, bytes).map_err(|error| {
        dataset_io_error(format!(
            "disk cache record write failed for {}: {error}",
            path.display()
        ))
    })
}

fn disk_cache_record_path(directory: &Path, key: &str) -> PathBuf {
    directory.join(format!("{}.json", key_to_hex_file_stem(key)))
}

fn key_to_hex_file_stem(key: &str) -> String {
    if key.is_empty() {
        return "empty".to_string();
    }
    let mut encoded = String::with_capacity(key.len() * 2);
    for byte in key.as_bytes() {
        encoded.push(nibble_to_hex(byte >> 4));
        encoded.push(nibble_to_hex(byte & 0x0f));
    }
    encoded
}

fn nibble_to_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'a' + nibble - 10) as char,
        _ => unreachable!("nibble is masked to 4 bits"),
    }
}

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

pub trait GDriveSheetTransport {
    fn put_sheet(&mut self, sheet_name: &str, rows: Vec<Vec<String>>) -> Result<(), RagasError>;
    fn get_sheet(&self, sheet_name: &str) -> Result<Vec<Vec<String>>, RagasError>;
    fn list_sheets(&self) -> Vec<String>;
    fn delete_sheet(&mut self, sheet_name: &str) -> bool;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InMemoryGoogleSheetTransport {
    sheets: BTreeMap<String, Vec<Vec<String>>>,
}

impl InMemoryGoogleSheetTransport {
    pub fn rows(&self, sheet_name: &str) -> Option<&[Vec<String>]> {
        self.sheets.get(sheet_name).map(Vec::as_slice)
    }
}

impl GDriveSheetTransport for InMemoryGoogleSheetTransport {
    fn put_sheet(&mut self, sheet_name: &str, rows: Vec<Vec<String>>) -> Result<(), RagasError> {
        self.sheets.insert(sheet_name.to_string(), rows);
        Ok(())
    }

    fn get_sheet(&self, sheet_name: &str) -> Result<Vec<Vec<String>>, RagasError> {
        self.sheets
            .get(sheet_name)
            .cloned()
            .ok_or_else(|| not_found(sheet_name))
    }

    fn list_sheets(&self) -> Vec<String> {
        self.sheets.keys().cloned().collect()
    }

    fn delete_sheet(&mut self, sheet_name: &str) -> bool {
        self.sheets.remove(sheet_name).is_some()
    }
}

#[derive(Debug, Clone)]
pub struct GoogleDriveDatasetBackend<T> {
    config: GDriveBackendConfig,
    transport: T,
}

impl<T: GDriveSheetTransport> GoogleDriveDatasetBackend<T> {
    pub fn new(config: GDriveBackendConfig, transport: T) -> Self {
        Self { config, transport }
    }

    pub fn config(&self) -> &GDriveBackendConfig {
        &self.config
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }
}

impl<T: GDriveSheetTransport> DatasetBackend for GoogleDriveDatasetBackend<T> {
    fn save(
        &mut self,
        name: &str,
        dataset: &EvaluationDataset<EvaluationSample>,
    ) -> Result<(), RagasError> {
        self.transport
            .put_sheet(&gdrive_sheet_name(name), dataset_to_gdrive_rows(dataset)?)
    }

    fn load(&self, name: &str) -> Result<EvaluationDataset<EvaluationSample>, RagasError> {
        gdrive_rows_to_dataset(self.transport.get_sheet(&gdrive_sheet_name(name))?)
    }

    fn list(&self) -> Vec<String> {
        self.transport
            .list_sheets()
            .into_iter()
            .filter_map(|sheet| sheet.strip_suffix(".gsheet").map(str::to_string))
            .collect()
    }

    fn delete(&mut self, name: &str) -> bool {
        self.transport.delete_sheet(&gdrive_sheet_name(name))
    }
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

fn gdrive_sheet_name(name: &str) -> String {
    format!("{name}.gsheet")
}

fn dataset_to_gdrive_rows(
    dataset: &EvaluationDataset<EvaluationSample>,
) -> Result<Vec<Vec<String>>, RagasError> {
    let mut row_maps = Vec::with_capacity(dataset.len());
    let mut headers = BTreeSet::new();

    for sample in dataset.samples() {
        let EvaluationSample::SingleTurn(sample) = sample else {
            return Err(dataset_io_error(
                "GDrive backend only supports single-turn samples",
            ));
        };
        let mut row = BTreeMap::new();
        row.insert("user_input".to_string(), sample.user_input.clone());
        row.insert("response".to_string(), sample.response.clone());
        row.insert(
            "retrieved_contexts".to_string(),
            serde_json::to_string(&sample.retrieved_contexts).map_err(|error| {
                dataset_io_error(format!("retrieved_contexts JSON encode failed: {error}"))
            })?,
        );
        row.insert(
            "reference".to_string(),
            sample.reference.clone().unwrap_or_default(),
        );
        for (key, value) in &sample.metadata {
            row.insert(format!("metadata.{key}"), value.clone());
        }
        headers.extend(row.keys().cloned());
        row_maps.push(row);
    }

    let header_row: Vec<String> = headers.into_iter().collect();
    let mut rows = Vec::with_capacity(row_maps.len() + 1);
    rows.push(header_row.clone());
    for row in row_maps {
        rows.push(
            header_row
                .iter()
                .map(|header| row.get(header).cloned().unwrap_or_default())
                .collect(),
        );
    }
    Ok(rows)
}

fn gdrive_rows_to_dataset(
    rows: Vec<Vec<String>>,
) -> Result<EvaluationDataset<EvaluationSample>, RagasError> {
    let headers = rows.first().cloned().unwrap_or_else(Vec::new);
    if headers.is_empty() {
        return EvaluationDataset::<EvaluationSample>::from_samples(Vec::new());
    }

    let mut samples = Vec::new();
    for row in rows.into_iter().skip(1) {
        let value_for = |name: &str| -> Option<&str> {
            headers
                .iter()
                .position(|header| header == name)
                .and_then(|index| row.get(index))
                .map(String::as_str)
        };

        let user_input = value_for("user_input").unwrap_or_default().to_string();
        let response = value_for("response").unwrap_or_default().to_string();
        let retrieved_contexts = value_for("retrieved_contexts")
            .filter(|value| !value.is_empty())
            .map(serde_json::from_str::<Vec<String>>)
            .transpose()
            .map_err(|error| {
                dataset_io_error(format!("retrieved_contexts JSON parse failed: {error}"))
            })?
            .unwrap_or_default();
        let mut sample = SingleTurnSample::new(user_input, response, retrieved_contexts);
        if let Some(reference) = value_for("reference").filter(|value| !value.is_empty()) {
            sample = sample.with_reference(reference);
        }
        for (index, header) in headers.iter().enumerate() {
            if let Some(key) = header.strip_prefix("metadata.")
                && let Some(value) = row.get(index).filter(|value| !value.is_empty())
            {
                sample = sample.with_metadata(key, value);
            }
        }
        samples.push(EvaluationSample::SingleTurn(sample));
    }

    EvaluationDataset::<EvaluationSample>::from_samples(samples)
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

    #[test]
    fn test_18_3_2_disk_cache_compatibility_preserves_key_value_semantics() {
        // SCEN-18.3.2 / AC2 / TEST-18.3.2
        let mut cache = DiskCacheCompatibility::new();

        cache.put("alpha", br#"{"score":0.8}"#.to_vec());
        cache.put("beta", b"second".to_vec());
        assert_eq!(cache.get("alpha"), Some(br#"{"score":0.8}"#.to_vec()));
        assert_eq!(cache.keys(), vec!["alpha".to_string(), "beta".to_string()]);

        cache.put("alpha", br#"{"score":0.9}"#.to_vec());
        assert_eq!(cache.get("alpha"), Some(br#"{"score":0.9}"#.to_vec()));
        assert!(cache.delete("beta"));
        assert_eq!(cache.get("beta"), None);
    }

    fn temp_cache_dir(name: &str) -> std::path::PathBuf {
        let unique = format!(
            "ragas-rs-{name}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&path);
        path
    }

    #[test]
    fn test_24_2_1_disk_cache_supports_upstream_key_value_semantics() {
        // SCEN-24.2.1 / AC1 / TEST-24.2.1
        let dir = temp_cache_dir("disk-cache-semantics");
        let mut cache = DiskCacheCompatibility::open(&dir).expect("open disk cache");

        cache
            .set("alpha", br#"{"score":0.8}"#.to_vec())
            .expect("set alpha");
        cache.set("beta", b"second".to_vec()).expect("set beta");

        assert!(cache.has_key("alpha"));
        assert_eq!(cache.get("alpha"), Some(br#"{"score":0.8}"#.to_vec()));
        assert_eq!(cache.keys(), vec!["alpha".to_string(), "beta".to_string()]);

        cache
            .set("alpha", br#"{"score":0.9}"#.to_vec())
            .expect("overwrite alpha");
        assert_eq!(cache.get("alpha"), Some(br#"{"score":0.9}"#.to_vec()));
        assert!(cache.delete("beta"));
        assert!(!cache.has_key("beta"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_24_2_2_disk_cache_persists_across_instances() {
        // SCEN-24.2.2 / AC2 / TEST-24.2.2
        let dir = temp_cache_dir("disk-cache-persistence");
        {
            let mut first = DiskCacheCompatibility::open(&dir).expect("open first cache");
            first
                .set("stable-key", br#"{"cached":true}"#.to_vec())
                .expect("set stable-key");
            assert!(first.has_key("stable-key"));
        }

        let reopened = DiskCacheCompatibility::open(&dir).expect("reopen cache");
        assert!(reopened.has_key("stable-key"));
        assert_eq!(
            reopened.get("stable-key"),
            Some(br#"{"cached":true}"#.to_vec())
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_25_1_1_gdrive_config_records_upstream_auth_contract() {
        // SCEN-25.1.1 / AC1 / TEST-25.1.1
        let config = GDriveBackendConfig::new("folder-root");

        assert_eq!(config.folder_id, "folder-root");
        assert_eq!(config.credentials_env, "GDRIVE_CREDENTIALS_PATH");
        assert_eq!(config.service_account_env, "GDRIVE_SERVICE_ACCOUNT_PATH");
        assert_eq!(config.token_env, "GDRIVE_TOKEN_PATH");
        assert_eq!(config.token_default, "token.json");
        assert_eq!(
            config.scopes,
            vec![
                "https://www.googleapis.com/auth/drive".to_string(),
                "https://www.googleapis.com/auth/spreadsheets".to_string(),
            ]
        );
        assert_eq!(
            config.auth_modes,
            vec![GDriveAuthMode::ServiceAccount, GDriveAuthMode::OAuth]
        );
        assert_eq!(config.datasets_folder_name, "datasets");
        assert_eq!(config.experiments_folder_name, "experiments");
    }

    #[test]
    fn test_25_1_2_gdrive_fake_transport_roundtrips_dataset_sheets() {
        // SCEN-25.1.2 / AC2 / TEST-25.1.2
        let dataset = fixture_dataset();
        let config = GDriveBackendConfig::new("folder-root");
        let mut backend =
            GoogleDriveDatasetBackend::new(config, InMemoryGoogleSheetTransport::default());

        backend
            .save("quality/run-1", &dataset)
            .expect("save dataset");

        assert_eq!(backend.list(), vec!["quality/run-1".to_string()]);
        let rows = backend
            .transport()
            .rows("quality/run-1.gsheet")
            .expect("sheet rows");
        assert_eq!(
            rows[0],
            vec![
                "metadata.source".to_string(),
                "reference".to_string(),
                "response".to_string(),
                "retrieved_contexts".to_string(),
                "user_input".to_string(),
            ]
        );
        assert_eq!(rows.len(), dataset.len() + 1);

        let loaded = backend.load("quality/run-1").expect("load dataset");
        assert_eq!(loaded.samples(), dataset.samples());

        assert!(backend.delete("quality/run-1"));
        assert!(backend.list().is_empty());
        assert!(backend.load("quality/run-1").is_err());
    }
}
