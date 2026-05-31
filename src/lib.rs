pub mod dataset;
pub mod eval;
pub mod error;
pub mod llm;
pub mod metric;
pub mod schema;

pub use dataset::{EvaluationDataset, EvaluationDatasetBuilder, EvaluationSample, SingleTurnSample};
pub use eval::{evaluate, EvaluationOptions, EvaluationReport, SampleEvaluation};
pub use error::RagasError;
pub use llm::{
    parse_chat_response, parse_embedding_response, ChatMessage, EmbeddingProvider,
    EmbeddingRequest, EmbeddingResponse, LlmProvider, LlmRequest, LlmResponse,
    OpenAiCompatibleClient, TokenUsage,
};
pub use metric::{
    cosine_similarity, ContextPrecisionMetric, FaithfulnessMetric, FnMetric, Metric, MetricResult,
    MetricValue, RankingItem, ResponseRelevancyMetric,
};
pub use schema::{Message, MessageRole, MultiTurnSample, Rubric, ToolCall};
