use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RagasError {
    #[error("evaluation dataset cannot be empty")]
    EmptyDataset,

    #[error("invalid sample at index {index}: {field}")]
    InvalidSample { index: usize, field: String },

    #[error("dataset IO error: {message}")]
    DatasetIo { message: String },

    #[error("provider error: {message}")]
    Provider { message: String },

    #[error("parse error: {message}")]
    Parse { message: String },

    #[error("prompt error: {message}")]
    Prompt { message: String },
}
