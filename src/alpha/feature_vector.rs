use crate::data::models::*;

/// Extract a FeatureVector from a TokenDetail.
pub fn extract_feature_vector(detail: &TokenDetail) -> FeatureVector {
    let ps = &detail.price_stats;
    let sec = &detail.security;
    let dev = &detail.dev_info;
    let tags = &detail.wallet_tags;

    FeatureVector {
        volume_1m: ps.volume_1h.map(|v| v / 60.0), // approximate
        volume_5m: None,
        volume_1h: ps.volume_1h,
        swaps_1h: ps.swaps_1h,
        price_change_1m: ps.change_1m,
        price_change_1h: ps.change_1h,
        hot_level: ps.hot_level,

        liquidity_usd: detail.token.liquidity_usd,
        market_cap: detail.token.market_cap,
        pool_exchange: detail.pool_info.as_ref().map(|p| p.exchange.clone()).unwrap_or_default(),
        is_on_curve: detail.token.is_on_curve,

        rug_ratio: sec.rug_ratio,
        is_wash_trading: sec.is_wash_trading,
        open_source: sec.open_source,
        renounced_mint: sec.renounced_mint,
        renounced_freeze: sec.renounced_freeze,

        holder_count: detail.token.holder_count,
        top_10_holder_rate: sec.top_10_holder_rate,
        dev_team_hold_rate: sec.dev_team_hold_rate,
        creator_hold_rate: sec.creator_hold_rate,
        suspected_insider_hold_rate: sec.suspected_insider_hold_rate,
        fresh_wallet_rate: tags.fresh_wallets as f64 / (detail.token.holder_count.max(1) as f64),

        smart_degen_count: tags.smart_wallets,
        renowned_count: tags.renowned_wallets,
        sniper_count: tags.sniper_wallets,
        bundler_rate: tags.bundler_wallets as f64 / (detail.token.holder_count.max(1) as f64),
        rat_trader_rate: tags.rat_trader_wallets as f64 / (detail.token.holder_count.max(1) as f64),

        creator_status: format!("{:?}", sec.creator_status).to_lowercase(),
        creator_prev_tokens: dev.creator_prev_tokens,
        creator_ath_mc: dev.creator_ath_mc,
        cto_flag: dev.cto_flag,
        dexscr_ad: dev.dexscr_ad,
        dexscr_boost: dev.dexscr_boost,

        has_social_links: detail.social_links.as_ref().map(|s| s.has_any()).unwrap_or(false),
        dexscr_trending_bar: dev.dexscr_trending_bar,
        launchpad_platform: detail.token.launchpad_platform.clone(),

        twitter_mentions_1h: None,
        twitter_sentiment: None,
        twitter_follower_count: None,
        twitter_ca_extracted: false,
        signal_confidence: None,
        smart_wallet_pnl: None,
    }
}

/// Merge Alph AI Twitter data into an existing FeatureVector.
pub fn merge_twitter_data(
    fv: &mut FeatureVector,
    mentions_1h: Option<u64>,
    sentiment: Option<f64>,
    follower_count: Option<u64>,
    signal_confidence: Option<f64>,
) {
    fv.twitter_mentions_1h = mentions_1h;
    fv.twitter_sentiment = sentiment;
    fv.twitter_follower_count = follower_count;
    fv.twitter_ca_extracted = true;
    fv.signal_confidence = signal_confidence;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample_detail() -> TokenDetail {
        TokenDetail {
            token: Token {
                address: "So11abc".to_string(),
                symbol: "PEPE".to_string(),
                name: "Pepe Coin".to_string(),
                liquidity_usd: 100_000.0,
                market_cap: 500_000.0,
                holder_count: 2_000,
                is_on_curve: false,
                ..Default::default()
            },
            security: TokenSecurity {
                rug_ratio: 0.05,
                top_10_holder_rate: 0.15,
                dev_team_hold_rate: 0.03,
                creator_hold_rate: 0.02,
                ..Default::default()
            },
            wallet_tags: WalletTags {
                smart_wallets: 12,
                renowned_wallets: 3,
                sniper_wallets: 5,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_extract_feature_vector() {
        let detail = make_sample_detail();
        let fv = extract_feature_vector(&detail);
        assert_eq!(fv.liquidity_usd, 100_000.0);
        assert_eq!(fv.holder_count, 2_000);
        assert_eq!(fv.rug_ratio, 0.05);
        assert_eq!(fv.smart_degen_count, 12);
        assert!(!fv.is_on_curve);
    }

    #[test]
    fn test_merge_twitter_data() {
        let detail = make_sample_detail();
        let mut fv = extract_feature_vector(&detail);
        assert!(fv.twitter_mentions_1h.is_none());

        merge_twitter_data(&mut fv, Some(50), Some(0.8), Some(10_000), Some(0.6));
        assert_eq!(fv.twitter_mentions_1h, Some(50));
        assert_eq!(fv.twitter_sentiment, Some(0.8));
        assert!(fv.twitter_ca_extracted);
    }
}