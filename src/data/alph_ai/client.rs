use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

/// Alph AI REST client with dex_cookie auth.
///
/// Auth: `Cookie: dex_cookie=<value>` header.
/// Cookie expires after 14 days — track and warn.
/// Base URL: `https://b.alph.ai/smart-web-gateway`
///
/// v1: read-only. No `order/create` or user-management endpoints.
pub struct AlphAiClient {
    http: Client,
    dex_cookie: String,
    base_url: String,
}

impl AlphAiClient {
    /// Create a new Alph AI client.
    pub fn new(dex_cookie: String) -> Self {
        let http = Client::builder()
            .build()
            .expect("building reqwest HTTP client");

        Self {
            http,
            dex_cookie,
            base_url: "https://b.alph.ai/smart-web-gateway".to_string(),
        }
    }

    /// Returns the base URL for tests/debugging.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    // ── Internal helpers ────────────────────────────────────────

    async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .header("Cookie", format!("dex_cookie={}", self.dex_cookie))
            .header("Content-Type", "application/json")
            .send()
            .await
            .with_context(|| format!("GET {}", url))?;

        let status = resp.status();
        resp.error_for_status()
            .with_context(|| format!("GET {} status {}", url, status))?
            .json()
            .await
            .context("parsing Alph AI JSON response")
    }

    async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .header("Cookie", format!("dex_cookie={}", self.dex_cookie))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {}", url))?;

        let status = resp.status();
        resp.error_for_status()
            .with_context(|| format!("POST {} status {}", url, status))?
            .json()
            .await
            .context("parsing Alph AI JSON response")
    }

    // ── Market Endpoints ────────────────────────────────────────

    /// One-shot token detail (price, MC, liquidity, security, social, AI description).
    pub async fn token_detail(&self, chain: &str, address: &str) -> Result<Value> {
        self.get(&format!(
            "/token/token-detail?chain={}&address={}",
            chain, address
        ))
        .await
    }

    /// Real-time current price.
    pub async fn current_price(&self, chain: &str, address: &str) -> Result<Value> {
        self.get(&format!(
            "/ticker/currentPrice?chain={}&address={}",
            chain, address
        ))
        .await
    }

    /// 24h stats for a token.
    pub async fn ticker_24h(&self, chain: &str, address: &str) -> Result<Value> {
        self.get(&format!(
            "/ticker/24h?chain={}&address={}",
            chain, address
        ))
        .await
    }

    /// Get k-line (candlestick) history.
    pub async fn kline(
        &self,
        chain: &str,
        token: &str,
        ktype: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "/kline/new/history?chain={}&token={}&type={}",
            chain, token, ktype
        ))
        .await
    }

    /// Popular tokens list.
    pub async fn popular_tokens(&self, chain: &str) -> Result<Value> {
        self.get(&format!(
            "/sherlock/popular_token/tokenPage?chain={}",
            chain
        ))
        .await
    }

    /// Get available snipe platforms for a chain.
    pub async fn snipe_platforms(&self, chain: &str) -> Result<Value> {
        self.get(&format!("/snipe/platform/{}", chain)).await
    }

    /// Newest token launches (snipe list).
    pub async fn snipe_new(
        &self,
        chain: &str,
        platform: &str,
        filters: &Value,
    ) -> Result<Value> {
        let body = serde_json::json!({
            "chain": chain,
            "platform": platform,
            "minAge": filters.get("minAge"),
            "maxAge": filters.get("maxAge"),
            "minMarketCap": filters.get("minMarketCap"),
            "minLiquidityUsdt": filters.get("minLiquidityUsdt"),
            "minHoldings": filters.get("minHoldings"),
            "hasTwitter": filters.get("hasTwitter"),
            "bondingCurve": filters.get("bondingCurve"),
        });
        self.post(&format!("/snipe/list/new/{}", chain), &body).await
    }

    /// AI-recommended new tokens (unique Alph AI signal).
    pub async fn snipe_aimost(
        &self,
        chain: &str,
        platform: &str,
        filters: &Value,
    ) -> Result<Value> {
        let body = serde_json::json!({
            "chain": chain,
            "platform": platform,
            "minAge": filters.get("minAge"),
            "minMarketCap": filters.get("minMarketCap"),
            "minLiquidityUsdt": filters.get("minLiquidityUsdt"),
            "hasTwitter": filters.get("hasTwitter"),
        });
        self.post(&format!("/snipe/list/aimost/{}", chain), &body).await
    }

    /// Graduated tokens (bonding curve → DEX).
    pub async fn snipe_graduated(
        &self,
        chain: &str,
        platform: &str,
        filters: &Value,
    ) -> Result<Value> {
        let body = serde_json::json!({
            "chain": chain,
            "platform": platform,
            "minMarketCap": filters.get("minMarketCap"),
            "minLiquidityUsdt": filters.get("minLiquidityUsdt"),
        });
        self.post(&format!("/snipe/list/graduated/{}", chain), &body).await
    }

    // ── Smart / Signals ────────────────────────────────────────

    /// Smart wallet list.
    pub async fn smart_wallets(&self, chain: &str) -> Result<Value> {
        self.get(&format!("/smart/smart-wallet?chain={}", chain)).await
    }

    /// Single wallet detail.
    pub async fn wallet_detail(&self, address: &str) -> Result<Value> {
        self.get(&format!("/smart/wallet?address={}", address)).await
    }

    /// Wallet's held tokens.
    pub async fn wallet_holdings(&self, address: &str) -> Result<Value> {
        self.get(&format!("/smart/holding-tokens?address={}", address)).await
    }

    /// Wallet PnL breakdown (richer than GMGN).
    pub async fn wallet_pnl(&self, address: &str) -> Result<Value> {
        self.get(&format!(
            "/smart/wallet-profit-loss?address={}",
            address
        ))
        .await
    }

    /// 1h hot tokens (smart money buys).
    pub async fn hot_tokens(&self, chain: &str) -> Result<Value> {
        self.get(&format!("/smart/hot-tokens?chain={}", chain)).await
    }

    /// 24h signal rank (Gold/Silver/Copper).
    pub async fn signal_rank_list(&self, chain: &str) -> Result<Value> {
        self.get(&format!("/signal/rank-list?chain={}", chain)).await
    }

    /// Latest signals.
    pub async fn signal_latest(&self, chain: &str) -> Result<Value> {
        self.get(&format!("/signal/list/latest?chain={}", chain)).await
    }

    /// Signals for a specific token.
    pub async fn signal_by_token(
        &self,
        chain: &str,
        address: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "/signal/list-by-token?chain={}&address={}",
            chain, address
        ))
        .await
    }

    // ── Twitter/X Endpoints ─────────────────────────────────────

    /// Search tweets by keyword.
    pub async fn twitter_search(&self, keyword: &str) -> Result<Value> {
        let body = serde_json::json!({"keyword": keyword});
        self.post("/x/search", &body).await
    }

    /// Get user's tweets.
    pub async fn twitter_tweets(&self, user_id: &str) -> Result<Value> {
        let body = serde_json::json!({"userId": user_id});
        self.post("/x/tweets", &body).await
    }

    /// Extract token CA from a tweet URL.
    pub async fn twitter_extract_ca(&self, tweet_url: &str) -> Result<Value> {
        self.get(&format!(
            "/token/twitter-search?url={}",
            tweet_url
        ))
        .await
    }

    /// Hot monitoring KOL list.
    pub async fn twitter_hot_list(&self) -> Result<Value> {
        let body = serde_json::json!({});
        self.post("/tracker/x/hotList", &body).await
    }

    // ── WebSocket helpers ──────────────────────────────────────

    /// Obtain a listenKey for WebSocket connection (expires 1h).
    pub async fn ws_listen_key(&self) -> Result<String> {
        let resp = self
            .http
            .post(format!("{}/ws/listenkey", self.base_url))
            .header("Cookie", format!("dex_cookie={}", self.dex_cookie))
            .send()
            .await
            .context("requesting WS listenKey")?;

        let json: Value = resp.json().await.context("parsing listenKey response")?;
        json["data"]["listenKey"]
            .as_str()
            .map(String::from)
            .context("listenKey not found in response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = AlphAiClient::new("test_cookie".to_string());
        assert!(client.base_url().contains("alph.ai"));
        assert_eq!(client.dex_cookie, "test_cookie");
    }
}