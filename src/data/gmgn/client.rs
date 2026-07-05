use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::net::{IpAddr, Ipv4Addr};
use tracing;

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
    pub fn new(api_key: String) -> Self {
        let http = Client::builder()
            .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(reqwest::header::ACCEPT, "application/json, text/plain, */*".parse().unwrap());
                h.insert(reqwest::header::ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
                h.insert(reqwest::header::CACHE_CONTROL, "no-cache".parse().unwrap());
                h
            })
            .build()
            .expect("building reqwest HTTP client (IPv4)");
        Self { http, api_key, rate_limiter: RateLimiter::new(20, 20), base_url: "https://gmgn.ai/defi/router/v1".to_string() }
    }

    /// Check Cloudflare block — GMGN returns HTML when Cloudflare intercepts.
    fn check_cloudflare_block(status: u16, content_type: &str) -> Result<()> {
        if content_type.contains("text/html") {
            anyhow::bail!("GMGN returned HTML (Cloudflare block) at status {}.", status);
        }
        Ok(())
    }

    /// Check for GMGN API error codes in the standard envelope.
    fn check_code(body: &Value) -> Result<()> {
        if let Some(code) = body.get("code").and_then(|c| c.as_i64()) {
            if code != 0 {
                let msg = body.get("msg").and_then(|m| m.as_str()).unwrap_or("unknown error");
                anyhow::bail!("GMGN API error (code {}): {}", code, msg);
            }
        }
        Ok(())
    }

    /// Send GET, verify response, return the full envelope body.
    async fn get_raw(&self, path: &str, weight: u32) -> Result<Value> {
        self.rate_limiter.acquire(weight).await;
        let url = format!("{}{}", self.base_url, path);
        tracing::debug!(url = %url, "GMGN GET");

        let resp = self.http.get(&url).header("X-APIKEY", &self.api_key).send().await
            .with_context(|| format!("GET {}", url))?;

        let status = resp.status();
        let ct = resp.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");
        Self::check_cloudflare_block(status.as_u16(), ct)?;
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let body: Value = resp.json().await.unwrap_or_default();
            let reset_at = body.get("reset_at").and_then(|v| v.as_u64());
            anyhow::bail!("GMGN rate limited, reset_at: {:?}", reset_at);
        }
        let body: Value = resp.error_for_status()
            .with_context(|| format!("GET {} status {}", url, status))?
            .json().await.context("parsing GMGN JSON")?;
        Self::check_code(&body)?;
        Ok(body)
    }

    /// Send POST, verify response, return the full envelope body.
    async fn post_raw(&self, path: &str, json_body: Value, weight: u32) -> Result<Value> {
        self.rate_limiter.acquire(weight).await;
        let url = format!("{}{}", self.base_url, path);
        tracing::debug!(url = %url, "GMGN POST");
        let resp = self.http.post(&url).header("X-APIKEY", &self.api_key).json(&json_body).send().await
            .with_context(|| format!("POST {}", url))?;
        let status = resp.status();
        let ct = resp.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");
        Self::check_cloudflare_block(status.as_u16(), ct)?;
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let body: Value = resp.json().await.unwrap_or_default();
            let reset_at = body.get("reset_at").and_then(|v| v.as_u64());
            anyhow::bail!("GMGN rate limited, reset_at: {:?}", reset_at);
        }
        let body: Value = resp.error_for_status()
            .with_context(|| format!("POST {} status {}", url, status))?
            .json().await.context("parsing GMGN JSON")?;
        Self::check_code(&body)?;
        Ok(body)
    }

    // ── Market Endpoints ────────────────────────────────────────

    /// Trending tokens. Response envelope: `{"code":0,"data":{"rank":[...]}}`.
    /// Returns the `data.rank` array.
    pub async fn trending(&self, interval: &str, limit: u32, order_by: &str) -> Result<Value> {
        let body = self.get_raw(
            &format!("/market/rank?chain=sol&interval={}&limit={}&order-by={}&direction=desc", interval, limit, order_by),
            1,
        ).await?;
        // Navigate: data → rank
        let data = body.get("data").context("missing data field in trending response")?;
        let rank = data.get("rank").context("missing data.rank in trending response")?.clone();
        Ok(rank)
    }

    /// Kline candles. Response envelope: `{"code":0,"data":{"list":[...]}}`.
    /// Returns the `data.list` array.
    pub async fn kline(&self, address: &str, resolution: &str, from: i64, to: i64) -> Result<Value> {
        let body = self.get_raw(
            &format!("/market/token_kline?chain=sol&address={}&resolution={}&from={}&to={}", address, resolution, from, to),
            2,
        ).await?;
        let data = body.get("data").context("missing data field in kline response")?;
        let list = data.get("list").context("missing data.list in kline response")?.clone();
        Ok(list)
    }

    /// Trenches (newly launched tokens). Response envelope: `{"code":0,"data":{...}}`.
    /// Returns the full `data` object (contains new_creation, pump, completed).
    pub async fn trenches(&self, token_type: &str) -> Result<Value> {
        let body = self.post_raw("/trenches", serde_json::json!({"chain": "sol", "type": token_type}), 3).await?;
        let data = body.get("data").cloned().unwrap_or(body);
        Ok(data)
    }

    /// Token signals. Response envelope: `{"code":0,"data":[...]}`.
    /// Returns the `data` array.
    pub async fn signal(&self) -> Result<Value> {
        let body = self.post_raw("/market/token_signal", serde_json::json!({"chain": "sol"}), 3).await?;
        let data = body.get("data").cloned().unwrap_or(body);
        Ok(data)
    }

    // ── Token Endpoints ────────────────────────────────────────

    /// Token info. Response envelope: `{"code":0,"data":{...}}`.
    /// Returns the `data` object.
    pub async fn token_info(&self, address: &str) -> Result<Value> {
        let body = self.get_raw(&format!("/token/info?chain=sol&address={}", address), 1).await?;
        let data = body.get("data").cloned().unwrap_or(body);
        Ok(data)
    }

    /// Token security info.
    pub async fn token_security(&self, address: &str) -> Result<Value> {
        let body = self.get_raw(&format!("/token/security?chain=sol&address={}", address), 1).await?;
        let data = body.get("data").cloned().unwrap_or(body);
        Ok(data)
    }

    /// Token holders by wallet tag.
    pub async fn token_holders(&self, address: &str, tag: &str, limit: u32) -> Result<Value> {
        let body = self.get_raw(&format!("/market/token_top_holders?chain=sol&address={}&tag={}&limit={}", address, tag, limit), 5).await?;
        let data = body.get("data").cloned().unwrap_or(body);
        Ok(data)
    }

    // ── Portfolio Endpoints ────────────────────────────────────

    pub async fn portfolio_info(&self, address: &str) -> Result<Value> {
        let body = self.get_raw(&format!("/portfolio/info?chain=sol&address={}", address), 1).await?;
        let data = body.get("data").cloned().unwrap_or(body);
        Ok(data)
    }

    pub async fn portfolio_holdings(&self, address: &str) -> Result<Value> {
        let body = self.get_raw(&format!("/portfolio/holdings?chain=sol&address={}", address), 5).await?;
        let data = body.get("data").cloned().unwrap_or(body);
        Ok(data)
    }

    // ── Track Endpoints ─────────────────────────────────────────

    /// Smart money trades. Response: `{"code":0,"data":[...]}` where each item is a trade.
    pub async fn smartmoney(&self, limit: u32) -> Result<Value> {
        let body = self.get_raw(&format!("/user/smartmoney?chain=sol&limit={}", limit), 1).await?;
        let data = body.get("data").cloned().unwrap_or(body);
        Ok(data)
    }

    /// KOL trades. Response: `{"code":0,"data":[...]}`.
    pub async fn kol_trades(&self, limit: u32) -> Result<Value> {
        let body = self.get_raw(&format!("/user/kol?chain=sol&limit={}", limit), 1).await?;
        let data = body.get("data").cloned().unwrap_or(body);
        Ok(data)
    }

    // ── Quote ────────────────────────────────────────────────────

    /// Quote for paper pricing.
    pub async fn quote(&self, input_token: &str, output_token: &str, amount: f64) -> Result<Value> {
        let body = self.get_raw(&format!("/trade/quote?chain=sol&input_token={}&output_token={}&input_amount={}", input_token, output_token, amount), 2).await?;
        let data = body.get("data").cloned().unwrap_or(body);
        Ok(data)
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
        assert!(client.base_url.contains("gmgn.ai"));
    }
}