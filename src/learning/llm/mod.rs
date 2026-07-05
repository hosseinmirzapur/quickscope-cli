//! LLM Provider — pluggable architecture for post-mortem analysis.
//! Uses an enum-based approach: no external trait crate needed.

use anyhow::Result;

pub mod prompts;

/// Supported LLM providers for post-mortem analysis.
#[derive(Debug, Clone)]
pub enum LlmProvider {
    OpenAi { api_key: String, model: String },
    Anthropic { api_key: String, model: String },
    Ollama { base_url: String, model: String },
    /// Dummy provider for testing — always returns the stored response.
    Stub { model: String, response: String },
}

impl LlmProvider {
    /// Send a chat completion request with system + user prompts.
    pub async fn chat(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        match self {
            Self::OpenAi { api_key, model } => openai_chat(api_key, model, system_prompt, user_prompt).await,
            Self::Anthropic { api_key, model } => anthropic_chat(api_key, model, system_prompt, user_prompt).await,
            Self::Ollama { base_url, model } => ollama_chat(base_url, model, system_prompt, user_prompt).await,
            Self::Stub { response, .. } => Ok(response.clone()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::OpenAi { .. } => "openai",
            Self::Anthropic { .. } => "anthropic",
            Self::Ollama { .. } => "ollama",
            Self::Stub { .. } => "stub",
        }
    }

    pub fn model(&self) -> &str {
        match self {
            Self::OpenAi { model, .. }
            | Self::Anthropic { model, .. }
            | Self::Ollama { model, .. }
            | Self::Stub { model, .. } => model,
        }
    }
}

// ── OpenAI ─────────────────────────────────────────────────────

async fn openai_chat(
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "temperature": 0.3,
        "max_tokens": 2000
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await?;

    let json: serde_json::Value = resp.error_for_status()?.json().await?;
    let text = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("(empty response)")
        .to_string();

    Ok(text)
}

// ── Anthropic ───────────────────────────────────────────────────

async fn anthropic_chat(
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "system": system_prompt,
        "messages": [
            {"role": "user", "content": user_prompt}
        ],
        "max_tokens": 2000,
        "temperature": 0.3
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await?;

    let json: serde_json::Value = resp.error_for_status()?.json().await?;
    let text = json["content"][0]["text"]
        .as_str()
        .unwrap_or("(empty response)")
        .to_string();

    Ok(text)
}

// ── Ollama (local) ──────────────────────────────────────────────

async fn ollama_chat(
    base_url: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "system": system_prompt,
        "prompt": user_prompt,
        "stream": false,
        "temperature": 0.3,
    });

    let url = format!("{}/api/generate", base_url.trim_end_matches('/'));
    let resp = client.post(&url).json(&body).send().await?;
    let json: serde_json::Value = resp.error_for_status()?.json().await?;
    let text = json["response"]
        .as_str()
        .unwrap_or("(empty response)")
        .to_string();

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_provider() {
        let provider = LlmProvider::Stub {
            model: "stub".to_string(),
            response: "Great trades!".to_string(),
        };
        assert_eq!(provider.name(), "stub");
    }

    #[test]
    fn test_openai_variant() {
        let provider = LlmProvider::OpenAi {
            api_key: "sk-test".to_string(),
            model: "gpt-4o".to_string(),
        };
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.model(), "gpt-4o");
    }

    #[test]
    fn test_anthropic_variant() {
        let provider = LlmProvider::Anthropic {
            api_key: "sk-ant-test".to_string(),
            model: "claude-3-opus".to_string(),
        };
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_ollama_variant() {
        let provider = LlmProvider::Ollama {
            base_url: "http://localhost:11434".to_string(),
            model: "llama3".to_string(),
        };
        assert_eq!(provider.name(), "ollama");
    }
}