use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RagasError {
    #[error("evaluation dataset cannot be empty")]
    EmptyDataset,

    #[error("invalid sample at index {index}: {field}")]
    InvalidSample { index: usize, field: String },
}
