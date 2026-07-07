use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

/// Simple DEX Screener REST client (no auth needed).
///
/// Used for cross-referencing GMGN trending data and
/// discovering boosted tokens.
pub struct DexScreenerClient {
    http: Client,
    base_url: String,
}

impl DexScreenerClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .build()
            .expect("building reqwest HTTP client");

        Self {
            http,
            base_url: "https://api.dexscreener.com/latest/dex".to_string(),
        }
    }

    /// Returns the base URL for tests/debugging.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {}", url))?;

        let status = resp.status();
        resp.error_for_status()
            .with_context(|| format!("GET {} status {}", url, status))?
            .json()
            .await
            .context("parsing DEX Screener JSON response")
    }

    /// Search for a token by name/symbol.
    pub async fn search(&self, query: &str) -> Result<Value> {
        self.get(&format!("/search?q={}", query)).await
    }

    /// Get pairs for a specific token on Solana.
    pub async fn token_pairs(&self, token_address: &str) -> Result<Value> {
        self.get(&format!("/tokens/{}", token_address)).await
    }

    /// Get Solana trending pairs.
    pub async fn trending(&self) -> Result<Value> {
        self.get("/search?q=sol%20trending").await
    }

    /// Get the latest boosted tokens.
    pub async fn latest_boosts(&self) -> Result<Value> {
        // Use the token-boosts endpoint
        let url = "https://api.dexscreener.com/token-boosts/latest/v1";
        let resp = self
            .http
            .get(url)
            .send()
            .await
            .context("GET token-boosts")?;
        let status = resp.status();
        resp.error_for_status()
            .with_context(|| format!("GET token-boosts status {}", status))?
            .json()
            .await
            .context("parsing token-boosts response")
    }
}

impl Default for DexScreenerClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = DexScreenerClient::new();
        assert!(client.base_url().contains("dexscreener"));
    }
}
