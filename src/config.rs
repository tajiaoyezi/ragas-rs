//! Centralized provider configuration.
//!
//! Every provider environment-variable name and default value lives in THIS module and
//! nowhere else — the CLI, the live metric tests and the testset synthesizer all resolve
//! their provider through [`ProviderConfig`], so there is a single source of truth.
//!
//! Resolution order for every value: the real process environment first, then a `.env`
//! file in the current directory, then the built-in default. A `.env` value never
//! overrides a variable that is already set in the real environment.

use std::collections::HashMap;
use std::sync::Arc;

use crate::{LlmProvider, OpenAiCompatibleClient};

/// Environment-variable names — the single source of truth.
pub const ENV_API_KEY: &str = "OPENAI_API_KEY";
pub const ENV_BASE_URL: &str = "OPENAI_BASE_URL";
pub const ENV_MODEL: &str = "OPENAI_MODEL";
pub const ENV_EMBEDDING_API_KEY: &str = "OPENAI_EMBEDDING_API_KEY";
pub const ENV_EMBEDDING_BASE_URL: &str = "OPENAI_EMBEDDING_BASE_URL";
pub const ENV_EMBEDDING_MODEL: &str = "OPENAI_EMBEDDING_MODEL";

/// Default values — the single source of truth.
pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_MODEL: &str = "gpt-4o-mini";
pub const DEFAULT_EMBEDDING_MODEL: &str = "text-embedding-3-small";

/// Provider configuration resolved from the environment and an optional `.env` file.
///
/// Build the live clients with [`ProviderConfig::chat_client`] /
/// [`ProviderConfig::embedding_client`]; both return `None` when no API key is configured
/// so callers can cleanly stay offline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    /// Embeddings may live on a different provider than chat; falls back to the chat key.
    pub embedding_api_key: Option<String>,
    pub embedding_base_url: String,
    pub embedding_model: String,
}

impl ProviderConfig {
    /// Resolve configuration from the process environment and a `.env` file in the cwd.
    pub fn from_env() -> Self {
        Self::resolve(|key| std::env::var(key).ok(), &dotenv_map())
    }

    /// Core resolution, parameterized over the environment lookup so it is deterministically
    /// testable without mutating the global process environment.
    fn resolve(env: impl Fn(&str) -> Option<String>, dotenv: &HashMap<String, String>) -> Self {
        let get = |key: &str| -> Option<String> {
            non_empty(env(key)).or_else(|| non_empty(dotenv.get(key).cloned()))
        };
        let api_key = get(ENV_API_KEY);
        let base_url = get(ENV_BASE_URL).unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let model = get(ENV_MODEL).unwrap_or_else(|| DEFAULT_MODEL.to_string());
        // Embeddings fall back to the chat provider when not configured separately.
        let embedding_api_key = get(ENV_EMBEDDING_API_KEY).or_else(|| api_key.clone());
        let embedding_base_url = get(ENV_EMBEDDING_BASE_URL).unwrap_or_else(|| base_url.clone());
        let embedding_model =
            get(ENV_EMBEDDING_MODEL).unwrap_or_else(|| DEFAULT_EMBEDDING_MODEL.to_string());
        Self {
            api_key,
            base_url,
            model,
            embedding_api_key,
            embedding_base_url,
            embedding_model,
        }
    }

    /// Whether a chat API key is configured (live LLM features available).
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Build a chat client, or `None` when no API key is configured.
    pub fn chat_client(&self) -> Option<OpenAiCompatibleClient> {
        let api_key = self.api_key.clone()?;
        Some(
            OpenAiCompatibleClient::new(self.base_url.clone(), api_key, self.model.clone())
                .with_embedding_model(self.embedding_model.clone()),
        )
    }

    /// Build an embedding client (which may target a different provider than chat),
    /// or `None` when no embedding/API key is configured.
    pub fn embedding_client(&self) -> Option<OpenAiCompatibleClient> {
        let api_key = self.embedding_api_key.clone()?;
        Some(OpenAiCompatibleClient::new(
            self.embedding_base_url.clone(),
            api_key,
            self.embedding_model.clone(),
        ))
    }

    /// Convenience: chat client as a trait object for `evaluate` / CLI wiring.
    pub fn chat_provider(&self) -> Option<Arc<dyn LlmProvider>> {
        self.chat_client()
            .map(|client| Arc::new(client) as Arc<dyn LlmProvider>)
    }

    /// Human-readable, secret-redacted summary (e.g. for a `show_config` command).
    pub fn redacted_summary(&self) -> String {
        format!(
            "chat:\n  {}={}\n  {}={}\n  {}={}\nembeddings:\n  {}={}\n  {}={}\n  {}={}",
            ENV_BASE_URL,
            self.base_url,
            ENV_MODEL,
            self.model,
            ENV_API_KEY,
            redact(self.api_key.as_deref()),
            ENV_EMBEDDING_BASE_URL,
            self.embedding_base_url,
            ENV_EMBEDDING_MODEL,
            self.embedding_model,
            ENV_EMBEDDING_API_KEY,
            redact(self.embedding_api_key.as_deref()),
        )
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty())
}

/// Mask a secret for display: first 6 + last 4 characters, or `***` if too short.
fn redact(secret: Option<&str>) -> String {
    match secret {
        None => "(not set)".to_string(),
        Some(value) => {
            let chars: Vec<char> = value.chars().collect();
            if chars.len() <= 8 {
                "***".to_string()
            } else {
                let head: String = chars[..6].iter().collect();
                let tail: String = chars[chars.len() - 4..].iter().collect();
                format!("{head}…{tail}")
            }
        }
    }
}

/// Parse a `.env` file in the current directory into a map (empty if absent/unreadable).
/// Lines are `KEY=VALUE`; blank lines and `#` comments are skipped; a leading `export `
/// and surrounding single/double quotes are stripped.
fn dotenv_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(contents) = std::fs::read_to_string(".env") else {
        return map;
    };
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let trimmed = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim();
            if key.is_empty() {
                continue;
            }
            let value = value.trim().trim_matches('"').trim_matches('\'');
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn defaults_apply_when_nothing_is_configured() {
        let config = ProviderConfig::resolve(|_| None, &HashMap::new());
        assert_eq!(config.api_key, None);
        assert!(!config.has_api_key());
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
        assert_eq!(config.model, DEFAULT_MODEL);
        assert_eq!(config.embedding_base_url, DEFAULT_BASE_URL);
        assert_eq!(config.embedding_model, DEFAULT_EMBEDDING_MODEL);
        // No key -> no clients.
        assert!(config.chat_client().is_none());
        assert!(config.embedding_client().is_none());
    }

    #[test]
    fn real_environment_wins_over_dotenv_which_wins_over_default() {
        let dotenv = map(&[
            (ENV_API_KEY, "from-dotenv"),
            (ENV_BASE_URL, "https://dotenv.example/v1"),
            (ENV_MODEL, "dotenv-model"),
        ]);
        // Real env overrides the base_url; model only in dotenv; api_key only in dotenv.
        let env = map(&[(ENV_BASE_URL, "https://real.example/v1")]);
        let config = ProviderConfig::resolve(|key| env.get(key).cloned(), &dotenv);
        assert_eq!(config.base_url, "https://real.example/v1"); // real env wins
        assert_eq!(config.model, "dotenv-model"); // dotenv used
        assert_eq!(config.api_key.as_deref(), Some("from-dotenv"));
    }

    #[test]
    fn embeddings_fall_back_to_chat_provider_then_can_be_overridden() {
        let chat_only = map(&[
            (ENV_API_KEY, "chat-key"),
            (ENV_BASE_URL, "https://chat.example/v1"),
        ]);
        let config = ProviderConfig::resolve(|key| chat_only.get(key).cloned(), &HashMap::new());
        assert_eq!(config.embedding_api_key.as_deref(), Some("chat-key"));
        assert_eq!(config.embedding_base_url, "https://chat.example/v1");
        assert_eq!(config.embedding_model, DEFAULT_EMBEDDING_MODEL);

        let split = map(&[
            (ENV_API_KEY, "chat-key"),
            (ENV_EMBEDDING_API_KEY, "embed-key"),
            (ENV_EMBEDDING_BASE_URL, "https://embed.example/v1"),
            (ENV_EMBEDDING_MODEL, "embed-model"),
        ]);
        let config = ProviderConfig::resolve(|key| split.get(key).cloned(), &HashMap::new());
        assert_eq!(config.embedding_api_key.as_deref(), Some("embed-key"));
        assert_eq!(config.embedding_base_url, "https://embed.example/v1");
        assert_eq!(config.embedding_model, "embed-model");
        // chat client uses chat-key; embedding client uses embed-key on its own base_url.
        assert!(config.chat_client().is_some());
        assert!(config.embedding_client().is_some());
    }

    #[test]
    fn blank_and_whitespace_values_are_treated_as_unset() {
        let env = map(&[(ENV_API_KEY, "   "), (ENV_MODEL, "")]);
        let config = ProviderConfig::resolve(|key| env.get(key).cloned(), &HashMap::new());
        assert_eq!(config.api_key, None);
        assert_eq!(config.model, DEFAULT_MODEL);
    }

    #[test]
    fn redacted_summary_masks_keys_and_never_leaks_the_secret() {
        let env = map(&[(ENV_API_KEY, "sk-abcdef0123456789")]);
        let config = ProviderConfig::resolve(|key| env.get(key).cloned(), &HashMap::new());
        let summary = config.redacted_summary();
        assert!(!summary.contains("sk-abcdef0123456789"));
        assert!(summary.contains("sk-abc")); // head
        assert!(summary.contains("6789")); // tail
        assert!(summary.contains(ENV_MODEL));
        assert_eq!(redact(None), "(not set)");
        assert_eq!(redact(Some("short")), "***");
    }
}
