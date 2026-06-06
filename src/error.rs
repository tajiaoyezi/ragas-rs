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

    /// The LLM stopped before completing its output (e.g. OpenAI `finish_reason == "length"`),
    /// so the response is truncated. Distinct from a malformed-but-complete [`Self::Parse`]:
    /// it is the faithful analog of Python ragas's `LLMDidNotFinishException`. Non-transient —
    /// retrying with the same `max_tokens` will not help.
    #[error(
        "the LLM did not finish generating (finish_reason: {reason}); increase max_tokens and try again"
    )]
    LlmDidNotFinish { reason: String },
}
