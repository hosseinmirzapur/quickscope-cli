use crate::data::models::*;

/// Normalize a value to [0, 1] using a sigmoid-like log scale.
/// Useful for values that vary widely (e.g., liquidity, market cap).
pub fn sigmoid_log(value: f64, midpoint: f64) -> f64 {
    if value <= 0.0 { return 0.0; }
    let x = (value / midpoint).ln();
    1.0 / (1.0 + (-x).exp())
}

/// Linear clamp: maps x to [0, 1], clamped at min/max.
pub fn normalize_linear(value: f64, min: f64, max: f64) -> f64 {
    if max <= min { return 0.0; }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

/// Clamp a value to a range and divide by max to normalize to [0,1].
pub fn clamp_divide(value: f64, min: f64, max: f64) -> f64 {
    let clamped = value.clamp(min, max);
    if max <= 0.0 { return 0.0; }
    (clamped / max).max(0.0)
}

// ── Category Scoring ──────────────────────────────────────────

/// Compute the 6 category scores and the composite Alpha Score.
pub fn compute_scores(fv: &FeatureVector, _config: &AlphaConfig) -> CategoryScores {
    CategoryScores {
        momentum: momentum_score(fv),
        safety: safety_score(fv),
        holder_quality: holder_quality_score(fv),
        liquidity: liquidity_score(fv),
        dev_trust: dev_trust_score(fv),
        social: social_score(fv),
    }
}

pub fn alpha_score(scores: &CategoryScores, config: &AlphaConfig) -> f64 {
    (config.w_momentum * scores.momentum
        + config.w_safety * scores.safety
        + config.w_holder * scores.holder_quality
        + config.w_liquidity * scores.liquidity
        + config.w_dev * scores.dev_trust
        + config.w_social * scores.social)
    * 100_f64
    .clamp(0.0, 100.0)
}

// ── Individual Category Formulas ───────────────────────────────

pub fn momentum_score(fv: &FeatureVector) -> f64 {
    let vol_score = normalize_linear(fv.volume_1h.unwrap_or(0.0), 0.0, 1_000_000.0);
    let swap_score = normalize_linear(fv.swaps_1h.unwrap_or(0) as f64, 0.0, 10_000.0);
    let hot_score = normalize_linear(fv.hot_level.unwrap_or(0) as f64, 0.0, 5.0);
    let change_score = clamp_divide(fv.price_change_1h.unwrap_or(0.0), -100.0, 500.0);

    vol_score * 0.30 + swap_score * 0.20 + hot_score * 0.20 + change_score * 0.30
}

pub fn safety_score(fv: &FeatureVector) -> f64 {
    let rug = (1.0 - fv.rug_ratio).clamp(0.0, 1.0);
    let wash = if fv.is_wash_trading { 0.0 } else { 1.0 };
    let mint = if fv.renounced_mint { 1.0 } else { 0.0 };
    let freeze = if fv.renounced_freeze { 1.0 } else { 0.0 };
    let top10 = (1.0 - fv.top_10_holder_rate).clamp(0.0, 1.0);

    rug * 0.35 + wash * 0.15 + mint * 0.15 + freeze * 0.15 + top10 * 0.20
}

pub fn holder_quality_score(fv: &FeatureVector) -> f64 {
    let smart = normalize_linear(fv.smart_degen_count as f64, 0.0, 50.0);
    let renowned = normalize_linear(fv.renowned_count as f64, 0.0, 20.0);
    let dev_hold = (1.0 - fv.dev_team_hold_rate).clamp(0.0, 1.0);
    let insider = (1.0 - fv.suspected_insider_hold_rate).clamp(0.0, 1.0);
    let fresh = (1.0 - fv.fresh_wallet_rate).clamp(0.0, 1.0);

    smart * 0.35 + renowned * 0.20 + dev_hold * 0.20 + insider * 0.15 + fresh * 0.10
}

pub fn liquidity_score(fv: &FeatureVector) -> f64 {
    let liq = sigmoid_log(fv.liquidity_usd, 50_000.0);
    let curve = if fv.is_on_curve { 0.0 } else { 1.0 };
    let mc = sigmoid_log(fv.market_cap, 10_000.0);

    liq * 0.50 + curve * 0.30 + mc * 0.20
}

pub fn dev_trust_score(fv: &FeatureVector) -> f64 {
    let status = match fv.creator_status.as_str() {
        "creatorclose" | "creator_close" => 0.0,
        _ => 0.5,
    };
    let ath = normalize_linear(
        fv.creator_ath_mc.unwrap_or(0.0),
        0.0,
        1_000_000.0,
    );
    let cto = if fv.cto_flag { 0.8 } else { 0.3 };
    let boost = if fv.dexscr_boost { 0.7 } else { 0.3 };
    let prev = normalize_linear(fv.creator_prev_tokens as f64, 0.0, 10.0);

    status * 0.30 + ath * 0.25 + cto * 0.20 + boost * 0.15 + prev * 0.10
}

pub fn social_score(fv: &FeatureVector) -> f64 {
    let mentions = normalize_linear(fv.twitter_mentions_1h.unwrap_or(0) as f64, 0.0, 100.0);
    let sentiment = fv.twitter_sentiment.unwrap_or(0.5).clamp(0.0, 1.0);
    let followers = normalize_linear(fv.twitter_follower_count.unwrap_or(0) as f64, 0.0, 1_000_000.0);
    let signal = fv.signal_confidence.unwrap_or(0.3);

    mentions * 0.30 + sentiment * 0.20 + followers * 0.20 + signal * 0.30
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_fv() -> FeatureVector {
        FeatureVector {
            liquidity_usd: 100_000.0,
            market_cap: 500_000.0,
            is_on_curve: false,
            volume_1h: Some(50_000.0),
            swaps_1h: Some(500),
            price_change_1h: Some(25.0),
            hot_level: Some(3),
            rug_ratio: 0.05,
            is_wash_trading: false,
            renounced_mint: true,
            renounced_freeze: true,
            top_10_holder_rate: 0.15,
            dev_team_hold_rate: 0.03,
            holder_count: 2_000,
            smart_degen_count: 12,
            renowned_count: 3,
            creator_status: "creatorclose".to_string(),
            creator_ath_mc: Some(200_000.0),
            creator_prev_tokens: 3,
            cto_flag: true,
            dexscr_boost: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_momentum_score() {
        let fv = sample_fv();
        let score = momentum_score(&fv);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_safety_score_perfect() {
        let fv = sample_fv();
        let score = safety_score(&fv);
        // Should be high since rug=0.05, not wash trading, renounced mint+freeze, low top10
        assert!(score > 0.8);
    }

    #[test]
    fn test_safety_score_risky() {
        let fv = FeatureVector {
            rug_ratio: 0.45,
            is_wash_trading: true,
            renounced_mint: false,
            renounced_freeze: false,
            top_10_holder_rate: 0.80,
            ..sample_fv()
        };
        let score = safety_score(&fv);
        assert!(score < 0.4);
    }

    #[test]
    fn test_liquidity_score() {
        let fv = sample_fv();
        let score = liquidity_score(&fv);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_alpha_score_computation() {
        let fv = sample_fv();
        let config = AlphaConfig::default();
        let scores = compute_scores(&fv, &config);
        let alpha = alpha_score(&scores, &config);
        assert!(alpha >= 0.0 && alpha <= 100.0, "alpha={}", alpha);
        // With good safety and holder scores, should be reasonably high
        assert!(alpha > 40.0);
    }

    #[test]
    fn test_sigmoid_log() {
        assert!((sigmoid_log(50_000.0, 50_000.0) - 0.5).abs() < 0.01);
        assert!(sigmoid_log(500_000.0, 50_000.0) > 0.9);
        assert!(sigmoid_log(5_000.0, 50_000.0) < 0.5);
    }

    #[test]
    fn test_normalize_linear() {
        assert_eq!(normalize_linear(0.0, 0.0, 100.0), 0.0);
        assert_eq!(normalize_linear(50.0, 0.0, 100.0), 0.5);
        assert_eq!(normalize_linear(100.0, 0.0, 100.0), 1.0);
    }

    #[test]
    fn test_clamp_divide() {
        assert_eq!(clamp_divide(-50.0, -100.0, 500.0), 0.0);
        assert_eq!(clamp_divide(250.0, -100.0, 500.0), 0.5);
        assert_eq!(clamp_divide(600.0, -100.0, 500.0), 1.0);
    }
}