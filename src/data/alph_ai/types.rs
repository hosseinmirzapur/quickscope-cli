use anyhow::Result;
use serde_json::Value;

use crate::data::models::*;

// ── Token Detail Parser ──────────────────────────────────────

/// Parse the token-detail response into a TokenDetail struct.
pub fn parse_alph_token_detail(v: &Value) -> Result<TokenDetail> {
    let data = v.get("data").unwrap_or(v);

    let token = Token {
        address: data["tokenAddress"].as_str().unwrap_or("").to_string(),
        symbol: data["symbol"].as_str().unwrap_or("").to_string(),
        name: data["tokenName"].as_str().unwrap_or("").to_string(),
        decimals: data["decimals"].as_u64().unwrap_or(0) as u8,
        price_usd: data["tokenPriceUsdt"]
            .as_f64()
            .or_else(|| data["price"].as_f64())
            .unwrap_or(0.0),
        market_cap: data["marketCap"].as_f64().unwrap_or(0.0),
        liquidity_usd: data["liquidityUsdt"]
            .as_f64()
            .or_else(|| data["liquidity"].as_f64())
            .unwrap_or(0.0),
        circulating_supply: data["circulatingSupply"].as_f64().unwrap_or(0.0),
        holder_count: data["holderCount"]
            .as_u64()
            .or_else(|| data["holders"].as_u64())
            .unwrap_or(0),
        created_at: chrono::Utc::now(),
        open_timestamp: data["createdTimestamp"].as_i64().unwrap_or(0),
        logo_url: data["logo"].as_str().map(String::from),
        launchpad_platform: data["platform"].as_str().map(String::from),
        is_on_curve: data["bondingCurve"].as_bool().unwrap_or(false),
    };

    Ok(TokenDetail {
        token,
        security: parse_alph_security(data),
        dev_info: DevInfo::default(),
        social_links: parse_alph_social(data),
        wallet_tags: WalletTags::default(),
        pool_info: None,
        price_stats: parse_alph_price_stats(data),
    })
}

fn parse_alph_security(v: &Value) -> TokenSecurity {
    TokenSecurity {
        rug_ratio: v["rugRatio"].as_f64().unwrap_or(0.0),
        is_wash_trading: v["washTrading"].as_bool().unwrap_or(false),
        open_source: v["openSource"].as_bool().unwrap_or(false),
        renounced_mint: v["renouncedMint"].as_bool().unwrap_or(false),
        renounced_freeze: v["renouncedFreezeAccount"].as_bool().unwrap_or(false),
        is_honeypot: v["honeypot"].as_bool().unwrap_or(false),
        buy_tax: v["buyTax"].as_f64().unwrap_or(0.0),
        sell_tax: v["sellTax"].as_f64().unwrap_or(0.0),
        top_10_holder_rate: v["top10HolderRate"].as_f64().unwrap_or(0.0),
        dev_team_hold_rate: v["devHoldRate"].as_f64().unwrap_or(0.0),
        creator_hold_rate: v["creatorHoldRate"].as_f64().unwrap_or(0.0),
        creator_status: CreatorStatus::Unknown,
        suspected_insider_hold_rate: 0.0,
        burn_status: String::new(),
        sniper_count: 0,
    }
}

fn parse_alph_social(v: &Value) -> Option<SocialLinks> {
    let twitter = v["twitter"].as_str().or_else(|| v["xUsername"].as_str());
    let website = v["website"]
        .as_str()
        .or_else(|| v["officialWebsite"].as_str());
    let telegram = v["telegram"].as_str();
    let discord = v["discord"].as_str();
    let description = v["description"]
        .as_str()
        .or_else(|| v["aiDescription"].as_str());

    if twitter.is_none() && website.is_none() && telegram.is_none() && discord.is_none() {
        return None;
    }

    Some(SocialLinks {
        twitter_username: twitter.map(String::from),
        website: website.map(String::from),
        telegram: telegram.map(String::from),
        discord: discord.map(String::from),
        description: description.map(String::from),
    })
}

fn parse_alph_price_stats(v: &Value) -> PriceStats {
    PriceStats {
        price_1m: v["price1m"].as_f64(),
        price_5m: v["price5m"].as_f64(),
        price_1h: v["price1h"].as_f64(),
        price_6h: v["price6h"].as_f64(),
        price_24h: v["price24h"].as_f64(),
        volume_1h: v["volume1h"].as_f64(),
        volume_24h: v["volume24h"].as_f64(),
        buys_1h: v["buys1h"].as_u64(),
        sells_1h: v["sells1h"].as_u64(),
        swaps_1h: v["swaps1h"].as_u64(),
        hot_level: None,
        change_1m: v["change1m"].as_f64(),
        change_5m: v["change5m"].as_f64(),
        change_1h: v["change1h"]
            .as_f64()
            .or_else(|| v["priceChangePercent1h"].as_f64()),
    }
}

// ── Tweet Parser ──────────────────────────────────────────────

/// Parse a tweet from Alph AI's /x/search or /x/tweets response.
pub fn parse_alph_tweet(v: &Value) -> Result<Tweet> {
    let created_at = v["created_at"]
        .as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);

    Ok(Tweet {
        id: v["id"].as_str().unwrap_or("").to_string(),
        username: v["username"]
            .as_str()
            .or_else(|| v["screen_name"].as_str())
            .unwrap_or("")
            .to_string(),
        display_name: v["display_name"]
            .as_str()
            .or_else(|| v["name"].as_str())
            .unwrap_or("")
            .to_string(),
        text: v["text"]
            .as_str()
            .or_else(|| v["full_text"].as_str())
            .unwrap_or("")
            .to_string(),
        tweet_type: parse_tweet_type(v),
        created_at,
        likes: v
            .get("public_metrics")
            .and_then(|p| p["like_count"].as_u64())
            .or_else(|| v["like_count"].as_u64())
            .unwrap_or(0),
        retweets: v
            .get("public_metrics")
            .and_then(|p| p["retweet_count"].as_u64())
            .or_else(|| v["retweet_count"].as_u64())
            .unwrap_or(0),
        replies: v
            .get("public_metrics")
            .and_then(|p| p["reply_count"].as_u64())
            .or_else(|| v["reply_count"].as_u64())
            .unwrap_or(0),
        extracted_token_addresses: Vec::new(), // filled later by CA extraction
    })
}

fn parse_tweet_type(v: &Value) -> TweetType {
    match v.get("type").and_then(|t| t.as_str()).unwrap_or("send") {
        "send" => TweetType::Send,
        "retweeted" => TweetType::Retweeted,
        "replied_to" => TweetType::RepliedTo,
        "quoted" => TweetType::Quoted,
        _ => TweetType::Send,
    }
}

// ── Signal Parser ─────────────────────────────────────────────

/// Parse a signal from Alph AI's signal rank/list endpoints.
pub fn parse_alph_signal(v: &Value) -> Result<TokenSignal> {
    let token_info = v.get("tokenInfo").unwrap_or(v);

    Ok(TokenSignal {
        token_address: token_info["tokenAddress"]
            .as_str()
            .or_else(|| v["tokenAddress"].as_str())
            .unwrap_or("")
            .to_string(),
        token_symbol: token_info["symbol"]
            .as_str()
            .or_else(|| v["symbol"].as_str())
            .unwrap_or("")
            .to_string(),
        signal_type: parse_alph_signal_type(v),
        confidence: parse_alph_confidence(v),
        trigger_at: v["triggerAt"]
            .as_i64()
            .or_else(|| v["createTime"].as_i64())
            .unwrap_or(0),
        amount_usd: None,
        description: v["reason"]
            .as_str()
            .or_else(|| v["description"].as_str())
            .map(String::from),
    })
}

fn parse_alph_signal_type(v: &Value) -> SignalType {
    let push_types = v.get("pushType");
    if let Some(arr) = push_types.and_then(|p| p.as_array()) {
        for item in arr {
            match item.as_str().unwrap_or("") {
                "SMART" => return SignalType::SmartMoneyBuy,
                "KOL_CALL" => return SignalType::KolMention,
                "CTO" => return SignalType::Cto,
                "DEX_AD" => return SignalType::DexAd,
                _ => {}
            }
        }
    }
    // Fallback: check individual fields
    if v["kolInfo"].is_object() {
        SignalType::KolMention
    } else if v["smartInfo"].is_object() {
        SignalType::SmartMoneyBuy
    } else {
        SignalType::PriceSpike
    }
}

fn parse_alph_confidence(v: &Value) -> SignalConfidence {
    match v.get("level").and_then(|l| l.as_str()).unwrap_or("copper") {
        "gold" | "Gold" => SignalConfidence::Gold,
        "silver" | "Silver" => SignalConfidence::Silver,
        _ => SignalConfidence::Copper,
    }
}

// ── Twitter CA Extraction Parser ──────────────────────────────

/// Parse the CA extraction result from twitter-search endpoint.
/// Returns a list of token addresses found in/associated with a tweet.
pub fn parse_twitter_ca_list(v: &Value) -> Result<Vec<String>> {
    let data = v.get("data");
    let arr = match data.and_then(|d| d.as_array()) {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    let addresses: Vec<String> = arr
        .iter()
        .filter_map(|item| item["tokenAddress"].as_str().map(String::from))
        .collect();

    Ok(addresses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_alph_tweet() {
        let v = json!({
            "id": "12345",
            "username": "solguy",
            "display_name": "Sol Guy",
            "text": "Check out $PEPE on solana!",
            "type": "send",
            "created_at": "2026-07-05T10:00:00Z",
            "public_metrics": {"like_count": 42, "retweet_count": 5, "reply_count": 3}
        });
        let tweet = parse_alph_tweet(&v).unwrap();
        assert_eq!(tweet.username, "solguy");
        assert_eq!(tweet.likes, 42);
        assert_eq!(tweet.retweets, 5);
        assert!(matches!(tweet.tweet_type, TweetType::Send));
    }

    #[test]
    fn test_parse_alph_tweet_retweeted() {
        let v = json!({
            "id": "67890",
            "username": "trader123",
            "display_name": "Trader",
            "text": "RT @solguy buy $WIF",
            "type": "retweeted",
            "created_at": "2026-07-05T11:00:00Z",
            "like_count": 10,
            "retweet_count": 0,
            "reply_count": 1
        });
        let tweet = parse_alph_tweet(&v).unwrap();
        assert!(matches!(tweet.tweet_type, TweetType::Retweeted));
    }

    #[test]
    fn test_parse_alph_signal_gold() {
        let v = json!({
            "tokenInfo": {
                "tokenAddress": "So11sig",
                "symbol": "MOON"
            },
            "pushType": ["SMART", "KOL_CALL"],
            "level": "gold",
            "createTime": 1720000000_i64,
            "reason": "Whale accumulation + KOL shill"
        });
        let signal = parse_alph_signal(&v).unwrap();
        assert_eq!(signal.token_symbol, "MOON");
        assert!(matches!(signal.signal_type, SignalType::SmartMoneyBuy));
        assert!(matches!(signal.confidence, SignalConfidence::Gold));
    }

    #[test]
    fn test_parse_alph_signal_silver() {
        let v = json!({
            "tokenInfo": {"tokenAddress": "So11abc", "symbol": "PEPE"},
            "pushType": ["KOL_CALL"],
            "level": "silver",
            "createTime": 1720000000_i64
        });
        let signal = parse_alph_signal(&v).unwrap();
        assert!(matches!(signal.confidence, SignalConfidence::Silver));
        assert!(matches!(signal.signal_type, SignalType::KolMention));
    }

    #[test]
    fn test_parse_twitter_ca_list() {
        let v = json!({
            "data": [
                {"chain": "sol", "tokenAddress": "So11111111111111111111111111111111111111112", "poolLiquidityUsdt": 16377.57},
                {"chain": "sol", "tokenAddress": "So22222222222222222222222222222222222222222", "poolLiquidityUsdt": 5000.0}
            ]
        });
        let addresses = parse_twitter_ca_list(&v).unwrap();
        assert_eq!(addresses.len(), 2);
        assert!(addresses[0].starts_with("So1"));
    }
}
