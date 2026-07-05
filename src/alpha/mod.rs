//! Alpha Filter Engine — the brain of QuickScope.
//!
//! Pipeline:
//!   FeatureVector → Hard Filters → Scoring → Rug Detection → Mode → Narrative → AlphaReport

use chrono::Utc;
use crate::data::models::*;

mod feature_vector;
mod scoring;
mod hard_filters;
mod rug_detect;
mod modes;
mod narrative;

pub use feature_vector::{extract_feature_vector, merge_twitter_data};
pub use scoring::{compute_scores, alpha_score, sigmoid_log, normalize_linear, clamp_divide};
pub use hard_filters::check_hard_filters;
pub use rug_detect::detect_rug;
pub use modes::{select_mode, sizing_for_mode, exit_params_for_mode};
pub use narrative::detect_narrative;

/// Run the full alpha filter pipeline on a TokenDetail.
/// Returns a complete AlphaReport.
pub fn analyze_token(detail: &TokenDetail, config: &AlphaConfig) -> AlphaReport {
    let feature_vector = extract_feature_vector(detail);
    let hard_filter_result = check_hard_filters(&feature_vector, config);
    let scores = compute_scores(&feature_vector, config);
    let alpha = alpha_score(&scores, config);
    let rug_report = detect_rug(&feature_vector);
    let mode = select_mode(alpha, &scores);
    let sizing = sizing_for_mode(&mode);
    let narrative = detect_narrative(detail);

    AlphaReport {
        token_address: detail.token.address.clone(),
        token_symbol: detail.token.symbol.clone(),
        alpha_score: alpha,
        scores,
        hard_filter_result,
        mode,
        sizing,
        feature_vector,
        rug_report,
        narrative,
        computed_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_good_detail() -> TokenDetail {
        TokenDetail {
            token: Token {
                address: "So11good".to_string(),
                symbol: "GOOD".to_string(),
                name: "Good Coin".to_string(),
                liquidity_usd: 150_000.0,
                market_cap: 800_000.0,
                holder_count: 3_000,
                is_on_curve: false,
                ..Default::default()
            },
            security: TokenSecurity {
                rug_ratio: 0.03,
                top_10_holder_rate: 0.12,
                dev_team_hold_rate: 0.02,
                renounced_mint: true,
                renounced_freeze: true,
                ..Default::default()
            },
            wallet_tags: WalletTags {
                smart_wallets: 15,
                renowned_wallets: 5,
                ..Default::default()
            },
            price_stats: PriceStats {
                volume_1h: Some(80_000.0),
                swaps_1h: Some(1_200),
                change_1h: Some(35.0),
                hot_level: Some(4),
                ..Default::default()
            },
            dev_info: DevInfo {
                cto_flag: true,
                dexscr_boost: true,
                creator_prev_tokens: 5,
                creator_ath_mc: Some(500_000.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_pipeline_good_token() {
        let detail = make_good_detail();
        let config = AlphaConfig::default();
        let report = analyze_token(&detail, &config);
        assert!(report.hard_filter_result.passed);
        assert!(report.alpha_score > 0.0);
    }

    #[test]
    fn test_pipeline_bad_token_rejected() {
        let mut detail = make_good_detail();
        detail.security.rug_ratio = 0.50;
        detail.security.is_wash_trading = true;
        detail.token.liquidity_usd = 2_000.0;
        let config = AlphaConfig::default();
        let report = analyze_token(&detail, &config);
        assert!(!report.hard_filter_result.passed);
        assert!(report.rug_report.severity == RugSeverity::Critical
            || report.rug_report.severity == RugSeverity::High);
    }
}