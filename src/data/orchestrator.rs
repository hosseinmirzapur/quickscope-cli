use anyhow::Result;
use serde_json::Value;

use crate::data::models::*;
use crate::data::alph_ai::AlphAiClient;
use crate::data::dex_screener::DexScreenerClient;
use crate::data::gmgn::GmgnClient;

/// High-level facade that merges data from all three sources.
///
/// Each method fans out to the right source(s), parses responses
/// into typed domain models, and returns them.
pub struct DataOrchestrator {
    pub gmgn: GmgnClient,
    pub alph_ai: AlphAiClient,
    pub dex: DexScreenerClient,
}

impl DataOrchestrator {
    pub fn new(
        gmgn_api_key: String,
        alph_dex_cookie: String,
    ) -> Self {
        Self {
            gmgn: GmgnClient::new(gmgn_api_key),
            alph_ai: AlphAiClient::new(alph_dex_cookie),
            dex: DexScreenerClient::new(),
        }
    }

    // ── Trending ──────────────────────────────────────────────

    /// Fetch trending tokens from GMGN (primary) and merge with
    /// DEX Screener boosts for conviction multiplier.
    pub async fn fetch_trending(&self) -> Result<Vec<TrendingToken>> {
        let gmgn_data = self.gmgn.trending("1h", 50, "volume").await?;
        let mut tokens = crate::data::gmgn::types::parse_trending_list(&gmgn_data)?;

        // Try to augment with DEX Screener boost info
        if let Ok(boosts) = self.dex.latest_boosts().await {
            if let Some(arr) = boosts["data"].as_object() {
                for token in &mut tokens {
                    let addr = &token.address;
                    if let Some(boosted) = arr.get(addr) {
                        if boosted.as_bool() == Some(true) {
                            token.dexscr_boost = Some(true);
                        }
                    }
                }
            }
        }

        Ok(tokens)
    }

    /// Fetch popular tokens from Alph AI.
    pub async fn fetch_alph_popular(&self) -> Result<Value> {
        self.alph_ai.popular_tokens("sol").await
    }

    /// Fetch trenches (new tokens) from GMGN.
    pub async fn fetch_trenches(&self, token_type: &str) -> Result<Vec<TrenchToken>> {
        let data = self.gmgn.trenches(token_type).await?;
        let empty = vec![];
        let arr = data.as_array()
            .unwrap_or(&empty);

        let tokens: Vec<TrenchToken> = arr.iter().map(|v| {
            TrenchToken {
                address: v["address"].as_str().unwrap_or("").to_string(),
                symbol: v["symbol"].as_str().unwrap_or("").to_string(),
                name: v["name"].as_str().unwrap_or("").to_string(),
                price_usd: v.get("price").and_then(|p| p["price"].as_f64())
                    .or_else(|| v["price"].as_f64()).unwrap_or(0.0),
                market_cap: v["market_cap"].as_f64().unwrap_or(0.0),
                liquidity_usd: v["liquidity"].as_f64().unwrap_or(0.0),
                age_minutes: v["age_minutes"].as_u64().unwrap_or(0),
                platform: v["platform"].as_str().unwrap_or("").to_string(),
                holder_count: v["holder_count"].as_u64().unwrap_or(0),
                dev_hold_rate: v["dev_hold_rate"].as_f64().unwrap_or(0.0),
                smart_holding: v["smart_holding"].as_u64().unwrap_or(0),
                kol_calls: v["kol_calls"].as_u64().unwrap_or(0),
                bonding_curve: v["bonding_curve"].as_bool().unwrap_or(false),
                social: None,
            }
        }).collect();

        Ok(tokens)
    }

    // ── Token Detail ──────────────────────────────────────────

    /// Fetch full token detail from multiple sources.
    /// Primary: GMGN (info + security + dev + holders).
    /// Secondary: Alph AI token-detail (AI description, social).
    pub async fn fetch_token_detail(&self, address: &str) -> Result<TokenDetail> {
        // Primary from GMGN
        let info = self.gmgn.token_info(address).await?;
        let security = self.gmgn.token_security(address).await?;
        let dev_info = self.gmgn.token_info(address).await?; // dev info embedded in token_info response

        // Merge into TokenDetail
        let mut detail = crate::data::gmgn::types::parse_token_detail(
            &info, &security, &dev_info, &info, &info,
        )?;

        // Augment with Alph AI (social, AI description)
        match self.alph_ai.token_detail("sol", address).await {
            Ok(alph_data) => {
                if let Ok(alph_detail) = crate::data::alph_ai::types::parse_alph_token_detail(&alph_data) {
                    // Merge social links if GMGN didn't have them
                    if detail.social_links.is_none() {
                        detail.social_links = alph_detail.social_links;
                    }
                    // Alph AI price used if GMGN price is 0
                    if detail.token.price_usd == 0.0 && alph_detail.token.price_usd > 0.0 {
                        detail.token.price_usd = alph_detail.token.price_usd;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(token = %address, "Alph AI token-detail failed: {}", e);
            }
        }

        Ok(detail)
    }

    // ── Kline ──────────────────────────────────────────────────

    /// Fetch kline data from GMGN.
    pub async fn fetch_kline(
        &self,
        address: &str,
        resolution: &str,
        from: i64,
        to: i64,
    ) -> Result<Vec<KlineCandle>> {
        let data = self.gmgn.kline(address, resolution, from, to).await?;
        let candles = crate::data::gmgn::types::parse_kline_list(&data)?;
        Ok(candles)
    }

    // ── Smart Money / Signals ─────────────────────────────────

    /// Fetch smart money trades from GMGN.
    pub async fn fetch_smart_money_trades(
        &self,
        limit: u32,
    ) -> Result<Vec<SmartMoneyTrade>> {
        let data = self.gmgn.smartmoney(limit).await?;
        let empty_smart = vec![];
        let arr = data.as_array().unwrap_or(&empty_smart);
        let trades: Vec<SmartMoneyTrade> = arr
            .iter()
            .filter_map(|v| crate::data::gmgn::types::parse_smart_money_trade(v).ok())
            .collect();
        Ok(trades)
    }

    /// Fetch signals from GMGN.
    pub async fn fetch_signals_gmgn(&self) -> Result<Vec<TokenSignal>> {
        let data = self.gmgn.signal().await?;
        let empty_sig = vec![];
        let arr = data.as_array().unwrap_or(&empty_sig);
        let signals: Vec<TokenSignal> = arr
            .iter()
            .filter_map(|v| crate::data::gmgn::types::parse_token_signal(v).ok())
            .collect();
        Ok(signals)
    }

    /// Fetch ALPH AI signals (Gold/Silver/Copper).
    pub async fn fetch_signals_alph(&self) -> Result<Vec<TokenSignal>> {
        let data = self.alph_ai.signal_rank_list("sol").await?;
        let empty_alph_sig = vec![];
        let arr = data.get("data")
            .and_then(|d| d.as_array())
            .unwrap_or(&empty_alph_sig);
        let signals: Vec<TokenSignal> = arr
            .iter()
            .filter_map(|v| crate::data::alph_ai::types::parse_alph_signal(v).ok())
            .collect();
        Ok(signals)
    }

    /// Fetch hot tokens (smart money) from Alph AI.
    pub async fn fetch_alph_hot_tokens(&self) -> Result<Value> {
        self.alph_ai.hot_tokens("sol").await
    }

    // ── Twitter / Social ──────────────────────────────────────

    /// Search tweets by keyword via Alph AI.
    pub async fn search_tweets(&self, keyword: &str) -> Result<Vec<Tweet>> {
        let data = self.alph_ai.twitter_search(keyword).await?;
        let empty_alph_sig = vec![];
        let arr = data.get("data")
            .and_then(|d| d.as_array())
            .unwrap_or(&empty_alph_sig);
        let tweets: Vec<Tweet> = arr
            .iter()
            .filter_map(|v| crate::data::alph_ai::types::parse_alph_tweet(v).ok())
            .collect();
        Ok(tweets)
    }

    /// Extract token addresses from a tweet URL.
    pub async fn extract_cas_from_tweet(&self, tweet_url: &str) -> Result<Vec<String>> {
        let data = self.alph_ai.twitter_extract_ca(tweet_url).await?;
        crate::data::alph_ai::types::parse_twitter_ca_list(&data)
    }

    // ── Quote (Paper Pricing) ─────────────────────────────────

    /// Get a price quote from GMGN for paper trade pricing.
    pub async fn get_quote(
        &self,
        input_token: &str,
        output_token: &str,
        amount: f64,
    ) -> Result<f64> {
        let data = self.gmgn.quote(input_token, output_token, amount).await?;
        let price = data["data"]["price"].as_f64()
            .or_else(|| data["price"].as_f64())
            .unwrap_or(0.0);
        Ok(price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let orch = DataOrchestrator::new(
            "gmgn_demo".to_string(),
            "test_cookie".to_string(),
        );
        assert!(orch.dex.base_url().contains("dexscreener"));
    }
}