use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    EvaluationDataset, EvaluationSample, ParityClaim, ParityFeatureStatus, ParityFixtureMetadata,
    ParityFixtureMode, RagasError, SingleTurnSample,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BackendFamily {
    InMemory,
    LocalJsonl,
    LocalCsv,
    DiskCache,
    GoogleDrive,
}

impl BackendFamily {
    pub fn slug(self) -> &'static str {
        match self {
            Self::InMemory => "in-memory",
            Self::LocalJsonl => "local-jsonl",
            Self::LocalCsv => "local-csv",
            Self::DiskCache => "disk-cache",
            Self::GoogleDrive => "gdrive",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendMode {
    Deterministic,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendCapability {
    DatasetStorage,
    KeyValueCache,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendDescriptor {
    pub family: BackendFamily,
    pub mode: BackendMode,
    pub capability: BackendCapability,
    pub supports_key_value: bool,
    pub requires_external_service: bool,
    pub parity_status: ParityFeatureStatus,
}

impl BackendDescriptor {
    pub fn new(
        family: BackendFamily,
        mode: BackendMode,
        capability: BackendCapability,
        supports_key_value: bool,
        requires_external_service: bool,
        parity_status: ParityFeatureStatus,
    ) -> Self {
        Self {
            family,
            mode,
            capability,
            supports_key_value,
            requires_external_service,
            parity_status,
        }
    }

    pub fn parity_feature(&self) -> String {
        format!("backend::{}", self.family.slug())
    }
}

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
            credentials_env: "GDRIVE_CREDENTIALS".to_string(),
            service_account_env: "GDRIVE_SERVICE_ACCOUNT".to_string(),
            token_env: "GDRIVE_TOKEN".to_string(),
            token_default: String::new(),
            scopes: Vec::new(),
            auth_modes: Vec::new(),
            datasets_folder_name: String::new(),
            experiments_folder_name: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendRegistry {
    descriptors: Vec<BackendDescriptor>,
}

impl BackendRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: backend_descriptors(),
        }
    }

    pub fn descriptors(&self) -> &[BackendDescriptor] {
        &self.descriptors
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn backend_descriptors() -> Vec<BackendDescriptor> {
    vec![
        BackendDescriptor::new(
            BackendFamily::InMemory,
            BackendMode::Deterministic,
            BackendCapability::DatasetStorage,
            false,
            false,
            ParityFeatureStatus::Complete,
        ),
        BackendDescriptor::new(
            BackendFamily::LocalJsonl,
            BackendMode::Deterministic,
            BackendCapability::DatasetStorage,
            false,
            false,
            ParityFeatureStatus::Complete,
        ),
        BackendDescriptor::new(
            BackendFamily::LocalCsv,
            BackendMode::Deterministic,
            BackendCapability::DatasetStorage,
            false,
            false,
            ParityFeatureStatus::Complete,
        ),
        BackendDescriptor::new(
            BackendFamily::DiskCache,
            BackendMode::Deterministic,
            BackendCapability::KeyValueCache,
            true,
            false,
            ParityFeatureStatus::Complete,
        ),
        BackendDescriptor::new(
            BackendFamily::GoogleDrive,
            BackendMode::External,
            BackendCapability::DatasetStorage,
            false,
            true,
            ParityFeatureStatus::KnownGap,
        ),
    ]
}

pub fn backend_parity_claims() -> Vec<ParityClaim> {
    backend_descriptors()
        .into_iter()
        .filter_map(|descriptor| {
            let feature = descriptor.parity_feature();
            if descriptor.family == BackendFamily::DiskCache {
                return Some(ParityClaim {
                    feature,
                    status: descriptor.parity_status,
                    fixtures: vec![backend_fixture_metadata(
                        "backend::disk-cache",
                        "src/ragas/cache.py",
                        Some("tests/unit/test_cache.py"),
                        "tests/parity/fixtures/backend_disk_cache.json",
                    )],
                });
            }
            (descriptor.parity_status != ParityFeatureStatus::Complete).then_some(ParityClaim {
                feature,
                status: descriptor.parity_status,
                fixtures: Vec::new(),
            })
        })
        .collect()
}

fn backend_fixture_metadata(
    feature: &str,
    upstream_module_path: &str,
    upstream_test_path: Option<&str>,
    fixture_path: &str,
) -> ParityFixtureMetadata {
    ParityFixtureMetadata::new(
        feature,
        upstream_module_path,
        upstream_test_path.map(str::to_string),
        fixture_path,
        ParityFixtureMode::DeterministicMock,
        None,
    )
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
        if removed {
            if let Some(directory) = &self.directory {
                let path = disk_cache_record_path(directory, key);
                match std::fs::remove_file(path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(_) => {}
                }
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
        _name: &str,
        _dataset: &EvaluationDataset<EvaluationSample>,
    ) -> Result<(), RagasError> {
        Ok(())
    }

    fn load(&self, name: &str) -> Result<EvaluationDataset<EvaluationSample>, RagasError> {
        Err(not_found(name))
    }

    fn list(&self) -> Vec<String> {
        Vec::new()
    }

    fn delete(&mut self, _name: &str) -> bool {
        false
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
    use crate::release_blocking_claims;

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
    fn test_18_3_1_backend_registry_lists_upstream_families_with_status() {
        // SCEN-18.3.1 / AC1 / TEST-18.3.1
        let registry = BackendRegistry::new();
        let descriptors = registry.descriptors();
        let families: BTreeSet<_> = descriptors
            .iter()
            .map(|descriptor| descriptor.family)
            .collect();

        for expected in [
            BackendFamily::InMemory,
            BackendFamily::LocalJsonl,
            BackendFamily::LocalCsv,
            BackendFamily::DiskCache,
            BackendFamily::GoogleDrive,
        ] {
            assert!(families.contains(&expected), "missing {expected:?}");
        }

        let memory = descriptors
            .iter()
            .find(|descriptor| descriptor.family == BackendFamily::InMemory)
            .expect("in-memory descriptor");
        assert_eq!(memory.mode, BackendMode::Deterministic);
        assert_eq!(memory.parity_status, ParityFeatureStatus::Complete);

        let gdrive = descriptors
            .iter()
            .find(|descriptor| descriptor.family == BackendFamily::GoogleDrive)
            .expect("gdrive descriptor");
        assert_eq!(gdrive.mode, BackendMode::External);
        assert!(gdrive.requires_external_service);
        assert_ne!(gdrive.parity_status, ParityFeatureStatus::Complete);
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

    #[test]
    fn test_18_3_3_unsupported_external_backend_blocks_release() {
        // SCEN-18.3.3 / AC3 / TEST-18.3.3
        let claims = backend_parity_claims();
        let blockers = release_blocking_claims(&claims);
        let blocking_features: BTreeSet<_> = blockers
            .iter()
            .map(|claim| claim.feature.as_str())
            .collect();

        assert!(blocking_features.contains("backend::gdrive"));
        assert!(claims.iter().all(|claim| {
            !(claim.feature == "backend::gdrive" && claim.status == ParityFeatureStatus::Complete)
        }));
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
    fn test_24_2_3_disk_cache_complete_claim_is_fixture_backed_and_not_blocking() {
        // SCEN-24.2.3 / AC3 / TEST-24.2.3
        let disk_descriptor = backend_descriptors()
            .into_iter()
            .find(|descriptor| descriptor.family == BackendFamily::DiskCache)
            .expect("disk-cache descriptor");
        assert_eq!(disk_descriptor.parity_status, ParityFeatureStatus::Complete);

        let claims = backend_parity_claims();
        let disk_claim = claims
            .iter()
            .find(|claim| claim.feature == "backend::disk-cache")
            .expect("disk-cache parity claim");
        assert_eq!(disk_claim.status, ParityFeatureStatus::Complete);
        assert!(!disk_claim.fixtures.is_empty());

        let blockers = release_blocking_claims(&claims);
        let blocking_features: BTreeSet<_> = blockers
            .iter()
            .map(|claim| claim.feature.as_str())
            .collect();
        assert!(!blocking_features.contains("backend::disk-cache"));
        assert!(blocking_features.contains("backend::gdrive"));
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

    #[test]
    fn test_25_1_3_gdrive_complete_claim_is_fixture_backed_and_not_blocking() {
        // SCEN-25.1.3 / AC3 / TEST-25.1.3
        let descriptor = backend_descriptors()
            .into_iter()
            .find(|descriptor| descriptor.family == BackendFamily::GoogleDrive)
            .expect("gdrive descriptor");
        assert_eq!(descriptor.parity_status, ParityFeatureStatus::Complete);
        assert_eq!(descriptor.mode, BackendMode::External);
        assert!(descriptor.requires_external_service);

        let claims = backend_parity_claims();
        let claim = claims
            .iter()
            .find(|claim| claim.feature == "backend::gdrive")
            .expect("gdrive parity claim");
        assert_eq!(claim.status, ParityFeatureStatus::Complete);
        assert_eq!(claim.fixtures.len(), 1);
        assert_eq!(
            claim.fixtures[0].fixture_path,
            "tests/parity/fixtures/backend_gdrive.json"
        );

        let blockers = release_blocking_claims(&claims);
        let blocking_features: BTreeSet<_> = blockers
            .iter()
            .map(|claim| claim.feature.as_str())
            .collect();
        assert!(!blocking_features.contains("backend::gdrive"));

        let synthetic_missing = vec![ParityClaim {
            feature: "backend::external-missing".to_string(),
            status: ParityFeatureStatus::KnownGap,
            fixtures: Vec::new(),
        }];
        assert_eq!(release_blocking_claims(&synthetic_missing).len(), 1);
    }
}
