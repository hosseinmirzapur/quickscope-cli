use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::net::{IpAddr, Ipv4Addr};

use super::rate_limiter::RateLimiter;

/// GMGN v1 read-only HTTP client.
///
/// Auth: `X-APIKEY` header with an API key from GMGN.
/// Rate limit: leaky bucket, rate=20, capacity=20.
/// IPv4 only: GMGN does not support IPv6 — we force IPv4 in the reqwest builder.
pub struct GmgnClient {
    http: Client,
    api_key: String,
    rate_limiter: RateLimiter,
    base_url: String,
}

impl GmgnClient {
    /// Create a new GMGN client.
    ///
    /// # Arguments
    /// * `api_key` — The GMGN API key (passed as `X-APIKEY` header).
    pub fn new(api_key: String) -> Self {
        let http = Client::builder()
            .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
            .build()
            .expect("building reqwest HTTP client (IPv4)");

        Self {
            http,
            api_key,
            rate_limiter: RateLimiter::new(20, 20),
            base_url: "https://gmgn.ai/defi/router/v1".to_string(),
        }
    }

    // ── Internal GET helper ────────────────────────────────────

    async fn get(&self, path: &str, weight: u32) -> Result<Value> {
        self.rate_limiter.acquire(weight).await;
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .header("X-APIKEY", &self.api_key)
            .send()
            .await
            .with_context(|| format!("GET {}", url))?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let body: Value = resp.json().await.unwrap_or_default();
            let reset_at = body.get("reset_at").and_then(|v| v.as_u64());
            anyhow::bail!("GMGN rate limited, reset_at: {:?}", reset_at);
        }

        resp.error_for_status()
            .with_context(|| format!("GET {} status", url))?
            .json()
            .await
            .context("parsing GMGN JSON response")
    }

    // ── Internal POST helper ───────────────────────────────────

    async fn post(&self, path: &str, body: Value, weight: u32) -> Result<Value> {
        self.rate_limiter.acquire(weight).await;
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .header("X-APIKEY", &self.api_key)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("POST {}", url))?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let body: Value = resp.json().await.unwrap_or_default();
            let reset_at = body.get("reset_at").and_then(|v| v.as_u64());
            anyhow::bail!("GMGN rate limited, reset_at: {:?}", reset_at);
        }

        resp.error_for_status()
            .with_context(|| format!("POST {} status", url))?
            .json()
            .await
            .context("parsing GMGN JSON response")
    }

    // ── Market Endpoints ────────────────────────────────────────

    /// Get trending tokens.
    ///
    /// Weight: 1
    pub async fn trending(
        &self,
        interval: &str,
        limit: u32,
        order_by: &str,
    ) -> Result<Value> {
        self.get(
            &format!(
                "/market/rank?chain=sol&interval={}&limit={}&order-by={}&direction=desc&filter=renounced&filter=frozen",
                interval, limit, order_by
            ),
            1,
        )
        .await
    }

    /// Get kline (candlestick) data for a token.
    ///
    /// Weight: 2
    pub async fn kline(
        &self,
        address: &str,
        resolution: &str,
        from: i64,
        to: i64,
    ) -> Result<Value> {
        self.get(
            &format!(
                "/market/token_kline?chain=sol&address={}&resolution={}&from={}&to={}",
                address, resolution, from, to
            ),
            2,
        )
        .await
    }

    /// Get trenches (newly launched tokens).
    ///
    /// Weight: 3
    pub async fn trenches(&self, token_type: &str) -> Result<Value> {
        let body = serde_json::json!({"chain": "sol", "type": token_type});
        self.post("/trenches", body, 3).await
    }

    /// Get token signals.
    ///
    /// Weight: 3
    pub async fn signal(&self) -> Result<Value> {
        let body = serde_json::json!({"chain": "sol"});
        self.post("/market/token_signal", body, 3).await
    }

    // ── Token Endpoints ────────────────────────────────────────

    /// Get token info.
    ///
    /// Weight: 1
    pub async fn token_info(&self, address: &str) -> Result<Value> {
        self.get(
            &format!("/token/info?chain=sol&address={}", address),
            1,
        )
        .await
    }

    /// Get token security info.
    ///
    /// Weight: 1
    pub async fn token_security(&self, address: &str) -> Result<Value> {
        self.get(
            &format!("/token/security?chain=sol&address={}", address),
            1,
        )
        .await
    }

    /// Get token holders info by tag.
    ///
    /// Weight: 5
    pub async fn token_holders(
        &self,
        address: &str,
        tag: &str,
        limit: u32,
    ) -> Result<Value> {
        self.get(
            &format!(
                "/market/token_top_holders?chain=sol&address={}&tag={}&limit={}",
                address, tag, limit
            ),
            5,
        )
        .await
    }

    // ── Portfolio Endpoints ────────────────────────────────────

    /// Get portfolio holdings comparison.
    ///
    /// Weight: 1
    pub async fn portfolio_info(&self, address: &str) -> Result<Value> {
        self.get(
            &format!("/portfolio/info?chain=sol&address={}", address),
            1,
        )
        .await
    }

    /// Get portfolio holdings list.
    ///
    /// Weight: 5
    pub async fn portfolio_holdings(&self, address: &str) -> Result<Value> {
        self.get(
            &format!("/portfolio/holdings?chain=sol&address={}", address),
            5,
        )
        .await
    }

    // ── Track Endpoints ─────────────────────────────────────────

    /// Get smart money wallets.
    ///
    /// Weight: 1
    pub async fn smartmoney(&self, limit: u32) -> Result<Value> {
        self.get(
            &format!("/user/smartmoney?chain=sol&limit={}", limit),
            1,
        )
        .await
    }

    /// Get KOL (Key Opinion Leader) trades.
    ///
    /// Weight: 1
    pub async fn kol_trades(&self, limit: u32) -> Result<Value> {
        self.get(
            &format!("/user/kol?chain=sol&limit={}", limit),
            1,
        )
        .await
    }

    // ── Quote (Paper Pricing) ───────────────────────────────────

    /// Get a price quote (for paper trade pricing only, no real execution).
    ///
    /// Weight: 2
    pub async fn quote(
        &self,
        input_token: &str,
        output_token: &str,
        amount: f64,
    ) -> Result<Value> {
        self.get(
            &format!(
                "/trade/quote?chain=sol&input_token={}&output_token={}&input_amount={}",
                input_token, output_token, amount
            ),
            2,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GmgnClient::new("gmgn_demo_key".to_string());
        assert_eq!(client.api_key, "gmgn_demo_key");
    }

    #[test]
    fn test_rate_limiter_basic() {
        let client = GmgnClient::new("demo".to_string());
        // Just verify the client was constructed
        assert!(client.base_url.contains("gmgn.ai"));
    }
}