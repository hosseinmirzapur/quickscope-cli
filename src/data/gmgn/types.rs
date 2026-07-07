use anyhow::{Context, Result};
use serde_json::Value;

use crate::data::models::*;

/// Helper: extract an f64 from a JSON value, handling both numeric and string representations.
fn parse_f64(v: &Value, key: &str) -> Option<f64> {
    let val = v.get(key)?;
    if let Some(n) = val.as_f64() {
        Some(n)
    } else if let Some(s) = val.as_str() {
        s.parse().ok()
    } else {
        None
    }
}

/// Helper: extract f64 from nested object, e.g. v["price"]["price"].
fn parse_nested_f64(v: &Value, outer: &str, inner: &str) -> Option<f64> {
    let price_obj = v.get(outer)?;
    parse_f64(price_obj, inner)
}

/// Parse a single trending token from GMGN's /market/rank response.
pub fn parse_trending_token(v: &Value) -> Result<TrendingToken> {
    Ok(TrendingToken {
        address: v["address"].as_str().unwrap_or("").to_string(),
        symbol: v
            .get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
        name: v
            .get("name")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
        price_usd: parse_nested_f64(v, "price", "price")
            .or_else(|| parse_f64(v, "price"))
            .unwrap_or(0.0),
        market_cap: parse_f64(v, "market_cap")
            .or_else(|| parse_f64(v, "marketCap"))
            .unwrap_or(0.0),
        liquidity_usd: parse_f64(v, "liquidity").unwrap_or(0.0),
        volume_5m: parse_nested_f64(v, "price", "volume_5m"),
        volume_1h: parse_nested_f64(v, "price", "volume_1h"),
        volume_24h: parse_nested_f64(v, "price", "volume_24h"),
        change_5m: parse_nested_f64(v, "price", "change5m").or_else(|| parse_f64(v, "change5m")),
        change_1h: parse_nested_f64(v, "price", "change1h").or_else(|| parse_f64(v, "change1h")),
        change_24h: parse_nested_f64(v, "price", "change24h").or_else(|| parse_f64(v, "change24h")),
        hot_level: v.get("hot_level").and_then(|h| h.as_u64()),
        smart_degen_count: v.get("smart_degen_count").and_then(|s| s.as_u64()),
        renowned_count: v.get("renowned_count").and_then(|r| r.as_u64()),
        holder_count: v.get("holder_count").and_then(|h| h.as_u64()),
        swaps_5m: v.get("swaps_5m").and_then(|s| s.as_u64()),
        swaps_1h: v.get("swaps_1h").and_then(|s| s.as_u64()),
        is_on_curve: v.get("is_on_curve").and_then(|i| i.as_bool()),
        launchpad_platform: v
            .get("launchpad_platform")
            .and_then(|l| l.as_str())
            .map(String::from),
        rug_ratio: parse_f64(v, "rug_ratio"),
        dexscr_boost: v.get("dexscr_boost").and_then(|d| d.as_bool()),
    })
}

/// Parse a single kline candle from GMGN's /market/token_kline response.
pub fn parse_kline_candle(v: &Value) -> Result<KlineCandle> {
    Ok(KlineCandle {
        time: v.get("time").and_then(|t| t.as_i64()).unwrap_or(0),
        open: v
            .get("open")
            .and_then(|o| o.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        close: v
            .get("close")
            .and_then(|c| c.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        high: v
            .get("high")
            .and_then(|h| h.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        low: v
            .get("low")
            .and_then(|l| l.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        volume_usd: v
            .get("volume")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        amount: v
            .get("amount")
            .and_then(|a| a.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        buys: v.get("buys").and_then(|b| b.as_u64()).unwrap_or(0),
        sells: v.get("sells").and_then(|s| s.as_u64()).unwrap_or(0),
    })
}

/// Parse a smart money trade from GMGN.
pub fn parse_smart_money_trade(v: &Value) -> Result<SmartMoneyTrade> {
    let side_str = v.get("side").and_then(|s| s.as_str()).unwrap_or("sell");
    let side = match side_str.to_lowercase().as_str() {
        "buy" => TradeSide::Buy,
        _ => TradeSide::Sell,
    };

    Ok(SmartMoneyTrade {
        tx_hash: v
            .get("tx_hash")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string(),
        maker: v
            .get("maker")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_string(),
        side,
        token_address: v
            .get("token")
            .and_then(|t| t["address"].as_str())
            .unwrap_or("")
            .to_string(),
        token_symbol: v
            .get("token")
            .and_then(|t| t["symbol"].as_str())
            .unwrap_or("")
            .to_string(),
        amount_usd: v
            .get("amount_usd")
            .and_then(|a| a.as_f64())
            .or_else(|| {
                v.get("amount_usd")
                    .and_then(|a| a.as_str())
                    .and_then(|s| s.parse().ok())
            })
            .unwrap_or(0.0),
        token_amount: v
            .get("amount")
            .and_then(|a| a.as_f64())
            .or_else(|| {
                v.get("amount")
                    .and_then(|a| a.as_str())
                    .and_then(|s| s.parse().ok())
            })
            .unwrap_or(0.0),
        price_usd: parse_nested_f64(v, "price", "price")
            .or_else(|| parse_f64(v, "price"))
            .unwrap_or(0.0),
        price_change: parse_f64(v, "price_change").unwrap_or(0.0),
        is_open_or_close: v
            .get("is_open_or_close")
            .and_then(|i| i.as_bool())
            .unwrap_or(false),
        timestamp: v.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0),
        maker_tags: {
            let raw = v.get("tags");
            raw.and_then(|t| t.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|item| item.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default()
        },
        maker_twitter: v
            .get("maker_twitter")
            .and_then(|m| m.as_str())
            .map(String::from),
        launchpad: v
            .get("launchpad")
            .and_then(|l| l.as_str())
            .map(String::from),
    })
}

/// Parse a token signal from GMGN.
pub fn parse_token_signal(v: &Value) -> Result<TokenSignal> {
    Ok(TokenSignal {
        token_address: v
            .get("address")
            .and_then(|a| a.as_str())
            .unwrap_or("")
            .to_string(),
        token_symbol: v
            .get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
        signal_type: parse_signal_type(v),
        confidence: parse_signal_confidence(v),
        trigger_at: v.get("trigger_at").and_then(|t| t.as_i64()).unwrap_or(0),
        amount_usd: v.get("amount_usd").and_then(|a| a.as_f64()),
        description: v
            .get("description")
            .and_then(|d| d.as_str())
            .map(String::from),
    })
}

fn parse_signal_type(v: &Value) -> SignalType {
    match v.get("signal_type").and_then(|s| s.as_str()).unwrap_or("") {
        "price_spike" => SignalType::PriceSpike,
        "smart_money_buy" => SignalType::SmartMoneyBuy,
        "large_buy" => SignalType::LargeBuy,
        "dex_ad" => SignalType::DexAd,
        "kol_mention" => SignalType::KolMention,
        "cto" => SignalType::Cto,
        _ => SignalType::PriceSpike,
    }
}

fn parse_signal_confidence(v: &Value) -> SignalConfidence {
    match v
        .get("confidence")
        .and_then(|c| c.as_str())
        .unwrap_or("copper")
    {
        "gold" => SignalConfidence::Gold,
        "silver" => SignalConfidence::Silver,
        _ => SignalConfidence::Copper,
    }
}

/// Parse token security info from GMGN.
pub fn parse_token_security(v: &Value) -> Result<TokenSecurity> {
    Ok(TokenSecurity {
        rug_ratio: v.get("rug_ratio").and_then(|r| r.as_f64()).unwrap_or(0.0),
        is_wash_trading: v
            .get("is_wash_trading")
            .and_then(|w| w.as_bool())
            .unwrap_or(false),
        open_source: v
            .get("open_source")
            .and_then(|o| o.as_bool())
            .unwrap_or(false),
        renounced_mint: v
            .get("renounced_mint")
            .and_then(|r| r.as_bool())
            .unwrap_or(false),
        renounced_freeze: v
            .get("renounced_freeze")
            .and_then(|r| r.as_bool())
            .unwrap_or(false),
        is_honeypot: v
            .get("is_honeypot")
            .and_then(|h| h.as_bool())
            .unwrap_or(false),
        buy_tax: v.get("buy_tax").and_then(|b| b.as_f64()).unwrap_or(0.0),
        sell_tax: v.get("sell_tax").and_then(|s| s.as_f64()).unwrap_or(0.0),
        top_10_holder_rate: v
            .get("top_10_holder_rate")
            .and_then(|t| t.as_f64())
            .unwrap_or(0.0),
        dev_team_hold_rate: v
            .get("dev_team_hold_rate")
            .and_then(|d| d.as_f64())
            .unwrap_or(0.0),
        creator_hold_rate: v
            .get("creator_hold_rate")
            .and_then(|c| c.as_f64())
            .unwrap_or(0.0),
        creator_status: parse_creator_status(v),
        suspected_insider_hold_rate: v
            .get("suspected_insider_hold_rate")
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0),
        burn_status: v
            .get("burn_status")
            .and_then(|b| b.as_str())
            .unwrap_or("")
            .to_string(),
        sniper_count: v.get("sniper_count").and_then(|s| s.as_u64()).unwrap_or(0),
    })
}

fn parse_creator_status(v: &Value) -> CreatorStatus {
    match v
        .get("creator_status")
        .and_then(|c| c.as_str())
        .unwrap_or("unknown")
    {
        "creator_hold" => CreatorStatus::CreatorHold,
        "creator_close" => CreatorStatus::CreatorClose,
        _ => CreatorStatus::Unknown,
    }
}

/// Parse dev info from GMGN.
pub fn parse_dev_info(v: &Value) -> Result<DevInfo> {
    Ok(DevInfo {
        creator_address: v
            .get("creator_address")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string(),
        creator_token_balance: v
            .get("creator_token_balance")
            .and_then(|c| c.as_f64())
            .unwrap_or(0.0),
        creator_status: parse_creator_status(v),
        creator_prev_tokens: v
            .get("creator_prev_tokens")
            .and_then(|c| c.as_u64())
            .unwrap_or(0),
        creator_ath_mc: v.get("creator_ath_mc").and_then(|c| c.as_f64()),
        creator_ath_token: v
            .get("creator_ath_token")
            .and_then(|c| c.as_str())
            .map(String::from),
        cto_flag: v.get("cto_flag").and_then(|c| c.as_bool()).unwrap_or(false),
        dexscr_ad: v
            .get("dexscr_ad")
            .and_then(|d| d.as_bool())
            .unwrap_or(false),
        dexscr_boost: v
            .get("dexscr_boost")
            .and_then(|d| d.as_bool())
            .unwrap_or(false),
        dexscr_trending_bar: v
            .get("dexscr_trending_bar")
            .and_then(|d| d.as_bool())
            .unwrap_or(false),
    })
}

/// Parse wallet tags from GMGN.
pub fn parse_wallet_tags(v: &Value) -> WalletTags {
    WalletTags {
        smart_wallets: v
            .get("smart_degen_count")
            .or_else(|| v.get("smart_wallets"))
            .and_then(|s| s.as_u64())
            .unwrap_or(0),
        renowned_wallets: v
            .get("renowned_count")
            .or_else(|| v.get("renowned_wallets"))
            .and_then(|r| r.as_u64())
            .unwrap_or(0),
        sniper_wallets: v
            .get("sniper_count")
            .or_else(|| v.get("sniper_wallets"))
            .and_then(|s| s.as_u64())
            .unwrap_or(0),
        rat_trader_wallets: v
            .get("rat_trader_wallets")
            .and_then(|r| r.as_u64())
            .unwrap_or(0),
        bundler_wallets: v
            .get("bundler_wallets")
            .and_then(|b| b.as_u64())
            .unwrap_or(0),
        whale_wallets: v.get("whale_wallets").and_then(|w| w.as_u64()).unwrap_or(0),
        fresh_wallets: v.get("fresh_wallets").and_then(|f| f.as_u64()).unwrap_or(0),
    }
}

/// Parse pool info from GMGN.
pub fn parse_pool_info(v: &Value) -> Option<PoolInfo> {
    let pool = v.get("pool")?;
    Some(PoolInfo {
        pool_address: pool
            .get("pool_address")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string(),
        exchange: pool
            .get("exchange")
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .to_string(),
        liquidity_usd: pool
            .get("liquidity_usd")
            .and_then(|l| l.as_f64())
            .unwrap_or(0.0),
        base_reserve: pool
            .get("base_reserve")
            .and_then(|b| b.as_f64())
            .unwrap_or(0.0),
        quote_reserve: pool
            .get("quote_reserve")
            .and_then(|q| q.as_f64())
            .unwrap_or(0.0),
        fee_ratio: pool
            .get("fee_ratio")
            .and_then(|f| f.as_f64())
            .unwrap_or(0.0),
    })
}

/// Parse price stats from GMGN.
pub fn parse_price_stats(v: &Value) -> PriceStats {
    let price = v.get("price");
    PriceStats {
        price_1m: price.and_then(|p| p["price_1m"].as_f64()),
        price_5m: price.and_then(|p| p["price_5m"].as_f64()),
        price_1h: price.and_then(|p| p["price_1h"].as_f64()),
        price_6h: price.and_then(|p| p["price_6h"].as_f64()),
        price_24h: price.and_then(|p| p["price_24h"].as_f64()),
        volume_1h: price.and_then(|p| p["volume_1h"].as_f64()),
        volume_24h: price.and_then(|p| p["volume_24h"].as_f64()),
        buys_1h: price.and_then(|p| p["buys_1h"].as_u64()),
        sells_1h: price.and_then(|p| p["sells_1h"].as_u64()),
        swaps_1h: price.and_then(|p| p["swaps_1h"].as_u64()),
        hot_level: v
            .get("hot_level")
            .and_then(|h| h.as_u64())
            .or_else(|| price.and_then(|p| p["hot_level"].as_u64())),
        change_1m: price.and_then(|p| p["change_1m"].as_f64()),
        change_5m: price.and_then(|p| p["change_5m"].as_f64()),
        change_1h: price.and_then(|p| p["change_1h"].as_f64()),
    }
}

/// Parse a full token detail by merging multiple GMGN response fields.
pub fn parse_token_detail(
    token_info: &Value,
    security: &Value,
    dev_info: &Value,
    wallet_tags: &Value,
    pool: &Value,
) -> Result<TokenDetail> {
    let token = Token {
        address: token_info["address"].as_str().unwrap_or("").to_string(),
        symbol: token_info
            .get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
        name: token_info
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string(),
        decimals: token_info
            .get("decimals")
            .and_then(|d| d.as_u64())
            .unwrap_or(0) as u8,
        price_usd: parse_nested_f64(token_info, "price", "price")
            .or_else(|| parse_f64(token_info, "price"))
            .unwrap_or(0.0),
        market_cap: parse_f64(token_info, "market_cap")
            .or_else(|| parse_f64(token_info, "marketCap"))
            .unwrap_or(0.0),
        liquidity_usd: parse_f64(token_info, "liquidity").unwrap_or(0.0),
        circulating_supply: parse_f64(token_info, "circulating_supply").unwrap_or(0.0),
        holder_count: token_info
            .get("holder_count")
            .and_then(|h| h.as_u64())
            .unwrap_or(0),
        created_at: chrono::Utc::now(), // GMGN doesn't always provide this
        open_timestamp: token_info
            .get("open_timestamp")
            .and_then(|t| t.as_i64())
            .unwrap_or(0),
        logo_url: token_info
            .get("logo")
            .and_then(|l| l.as_str())
            .map(String::from),
        launchpad_platform: token_info
            .get("launchpad_platform")
            .and_then(|l| l.as_str())
            .map(String::from),
        is_on_curve: token_info
            .get("is_on_curve")
            .and_then(|i| i.as_bool())
            .unwrap_or(false),
    };

    Ok(TokenDetail {
        token,
        security: parse_token_security(security)?,
        dev_info: parse_dev_info(dev_info)?,
        social_links: None, // TODO: parse from Alph AI twitter module
        wallet_tags: parse_wallet_tags(wallet_tags),
        pool_info: parse_pool_info(pool),
        price_stats: parse_price_stats(token_info),
    })
}

/// Parse an array of trending tokens.
pub fn parse_trending_list(v: &Value) -> Result<Vec<TrendingToken>> {
    let arr = v.as_array().context("expected array of trending tokens")?;
    arr.iter().map(parse_trending_token).collect()
}

/// Parse an array of kline candles.
pub fn parse_kline_list(v: &Value) -> Result<Vec<KlineCandle>> {
    let arr = v.as_array().context("expected array of kline candles")?;
    arr.iter().map(parse_kline_candle).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_trending_token_minimal() {
        let v = json!({
            "address": "So11111111111111111111111111111111111111112",
            "symbol": "PEPE",
            "name": "Pepe Coin",
            "price": {"price": "0.000018"},
            "marketCap": 12_400_000.0,
            "liquidity": 890_000.0,
            "smart_degen_count": 5,
            "renowned_count": 2,
            "holder_count": 15_000,
            "hot_level": 3,
            "is_on_curve": false
        });
        let token = parse_trending_token(&v).unwrap();
        assert_eq!(token.symbol, "PEPE");
        assert_eq!(token.price_usd, 0.000018);
        assert_eq!(token.smart_degen_count, Some(5));
        assert_eq!(token.is_on_curve, Some(false));
    }

    #[test]
    fn test_parse_trending_token_empty() {
        let v = json!({});
        let token = parse_trending_token(&v).unwrap();
        assert_eq!(token.symbol, "");
        assert_eq!(token.price_usd, 0.0);
    }

    #[test]
    fn test_parse_kline_candle() {
        let v = json!({
            "time": 1720000000000_i64,
            "open": "0.000010",
            "close": "0.000012",
            "high": "0.000015",
            "low": "0.000009",
            "volume": "50000.0",
            "amount": "5000000000.0",
            "buys": 120_u64,
            "sells": 80_u64
        });
        let candle = parse_kline_candle(&v).unwrap();
        assert_eq!(candle.time, 1720000000000);
        assert_eq!(candle.open, 0.000010);
        assert_eq!(candle.close, 0.000012);
        assert_eq!(candle.buys, 120);
        assert_eq!(candle.sells, 80);
    }

    #[test]
    fn test_parse_smart_money_trade_buy() {
        let v = json!({
            "tx_hash": "tx123",
            "maker": "walletA",
            "side": "buy",
            "token": {"address": "So11abc", "symbol": "BONK"},
            "amount_usd": "5000.0",
            "amount": "1000000.0",
            "price": {"price": 0.000005},
            "price_change": 2.5,
            "is_open_or_close": true,
            "timestamp": 1720000000_i64,
            "tags": ["smart_degen", "whale"],
            "maker_twitter": "solguy"
        });
        let trade = parse_smart_money_trade(&v).unwrap();
        assert_eq!(trade.side, TradeSide::Buy);
        assert_eq!(trade.token_symbol, "BONK");
        assert_eq!(trade.amount_usd, 5000.0);
        assert_eq!(trade.maker_tags.len(), 2);
        assert_eq!(trade.maker_twitter.unwrap(), "solguy");
    }

    #[test]
    fn test_parse_smart_money_trade_sell() {
        let v = json!({
            "side": "sell",
            "token": {"address": "So11xyz", "symbol": "WIF"},
            "amount_usd": "1000.0"
        });
        let trade = parse_smart_money_trade(&v).unwrap();
        assert_eq!(trade.side, TradeSide::Sell);
    }

    #[test]
    fn test_parse_token_signal() {
        let v = json!({
            "address": "So11sig",
            "symbol": "MOON",
            "signal_type": "smart_money_buy",
            "confidence": "gold",
            "trigger_at": 1720000000_i64,
            "amount_usd": 10000.0,
            "description": "Whale bought $10k"
        });
        let signal = parse_token_signal(&v).unwrap();
        assert!(matches!(signal.signal_type, SignalType::SmartMoneyBuy));
        assert!(matches!(signal.confidence, SignalConfidence::Gold));
        assert_eq!(signal.amount_usd, Some(10000.0));
    }

    #[test]
    fn test_parse_token_security_clean() {
        let v = json!({
            "rug_ratio": 0.05,
            "is_wash_trading": false,
            "open_source": true,
            "renounced_mint": true,
            "renounced_freeze": true,
            "is_honeypot": false,
            "buy_tax": 0.0,
            "sell_tax": 0.0,
            "top_10_holder_rate": 0.15,
            "dev_team_hold_rate": 0.03,
            "creator_hold_rate": 0.02,
            "creator_status": "creator_close",
            "suspected_insider_hold_rate": 0.01,
            "burn_status": "none",
            "sniper_count": 0
        });
        let security = parse_token_security(&v).unwrap();
        assert_eq!(security.rug_ratio, 0.05);
        assert!(!security.is_wash_trading);
        assert!(security.renounced_mint);
        assert_eq!(security.creator_status, CreatorStatus::CreatorClose);
    }

    #[test]
    fn test_parse_token_security_risky() {
        let v = json!({
            "rug_ratio": 0.45,
            "is_wash_trading": true,
            "dev_team_hold_rate": 0.25,
            "top_10_holder_rate": 0.80
        });
        let security = parse_token_security(&v).unwrap();
        assert_eq!(security.rug_ratio, 0.45);
        assert!(security.is_wash_trading);
        assert_eq!(security.dev_team_hold_rate, 0.25);
    }

    #[test]
    fn test_parse_trending_list() {
        let v = json!([
            {"address": "A", "symbol": "A", "name": "A", "price": {"price": 1.0}},
            {"address": "B", "symbol": "B", "name": "B", "price": {"price": 2.0}}
        ]);
        let list = parse_trending_list(&v).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_parse_trending_list_not_array() {
        let v = json!({"not": "an array"});
        let result = parse_trending_list(&v);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_kline_list() {
        let v = json!([
            {"time": 1, "open": "1.0", "close": "2.0", "high": "3.0", "low": "0.5", "volume": "100", "amount": "200", "buys": 10, "sells": 5},
            {"time": 2, "open": "2.0", "close": "3.0", "high": "4.0", "low": "1.0", "volume": "200", "amount": "400", "buys": 20, "sells": 10}
        ]);
        let list = parse_kline_list(&v).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[1].time, 2);
    }
}
