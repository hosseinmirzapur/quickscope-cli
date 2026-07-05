use super::models::*;

#[test]
fn test_trending_token_creation() {
    let token = TrendingToken {
        address: "So11...abc".to_string(),
        symbol: "PEPE".to_string(),
        name: "Pepe".to_string(),
        price_usd: 0.000018,
        market_cap: 12_400_000.0,
        liquidity_usd: 890_000.0,
        volume_1h: Some(42_000.0),
        change_1h: Some(18.5),
        smart_degen_count: Some(5),
        is_on_curve: Some(false),
        ..Default::default()
    };
    assert_eq!(token.symbol, "PEPE");
    assert_eq!(token.market_cap, 12_400_000.0);
}

#[test]
fn test_smart_money_trade_side() {
    let trade = SmartMoneyTrade {
        side: TradeSide::Buy,
        amount_usd: 1500.0,
        ..Default::default()
    };
    assert_eq!(trade.side, TradeSide::Buy);
    assert_eq!(trade.amount_usd, 1500.0);
}

#[test]
fn test_signal_confidence_score() {
    assert_eq!(SignalConfidence::Gold.score(), 1.0);
    assert_eq!(SignalConfidence::Silver.score(), 0.6);
    assert_eq!(SignalConfidence::Copper.score(), 0.3);
}

#[test]
fn test_trade_mode_as_str() {
    assert_eq!(TradeMode::Explode.as_str(), "EXPLODE");
    assert_eq!(TradeMode::Alpha.as_str(), "ALPHA");
    assert_eq!(TradeMode::Scalp.as_str(), "SCALP");
    assert_eq!(TradeMode::Fallback.as_str(), "FALLBACK");
}

#[test]
fn test_social_links_has_any() {
    let empty = SocialLinks::default();
    assert!(!empty.has_any());

    let with_twitter = SocialLinks {
        twitter_username: Some("solguy".to_string()),
        ..Default::default()
    };
    assert!(with_twitter.has_any());
}

#[test]
fn test_alpha_config_defaults() {
    let config = AlphaConfig::default();
    assert_eq!(config.w_momentum, 0.25);
    assert_eq!(config.hf_rug_ratio_max, 0.30);
    assert!(config.hf_wash_trading);
    assert!(config.hf_renounced_mint);
}

#[test]
fn test_risk_state_defaults() {
    let risk = RiskState::default();
    assert_eq!(risk.daily_loss_cap_sol, 5.0);
    assert_eq!(risk.per_trade_max_sol, 2.5);
    assert_eq!(risk.max_open_positions, 5);
    assert!(!risk.kill_switch_active);
}

#[test]
fn test_tab_index_navigation() {
    assert_eq!(TabIndex::Dashboard.next(), TabIndex::Scanner);
    assert_eq!(TabIndex::Scanner.prev(), TabIndex::Dashboard);
    assert_eq!(TabIndex::Settings.next(), TabIndex::Dashboard);
    assert_eq!(TabIndex::Dashboard.prev(), TabIndex::Settings);
    assert_eq!(TabIndex::Scanner.label(), "Scanner");
    assert_eq!(TabIndex::COUNT, 7);
}

#[test]
fn test_portfolio_defaults() {
    let portfolio = Portfolio::default();
    assert_eq!(portfolio.balance_sol, 50.0);
}

#[test]
fn test_paper_position_default() {
    let pos = PaperPosition::default();
    assert_eq!(pos.status, PositionStatus::Open);
    assert_eq!(pos.mode, TradeMode::Fallback);
    assert!(pos.tp_percent.is_none());
}

#[test]
fn test_rug_severity_ordering() {
    assert_ne!(RugSeverity::Low, RugSeverity::Critical);
    assert_ne!(RugSeverity::Medium, RugSeverity::High);
}

#[test]
fn test_filter_failure_directions() {
    let exceeded = FilterFailure {
        name: "rug_ratio".to_string(),
        value: 0.45,
        threshold: 0.30,
        direction: FilterDirection::Exceeded,
    };
    assert!(matches!(exceeded.direction, FilterDirection::Exceeded));
}

#[test]
fn test_kline_candle_default() {
    let candle = KlineCandle::default();
    assert_eq!(candle.time, 0);
    assert_eq!(candle.buys, 0);
    assert_eq!(candle.sells, 0);
}

#[test]
fn test_creator_status_serde() {
    let status = CreatorStatus::CreatorHold;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"creator_hold\"");

    let parsed: CreatorStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, CreatorStatus::CreatorHold);
}
