# Alpha Filter & Scoring Engine

> The brain of QuickScope. Full detail in the master spec §6.
> This doc is the implementer's reference.

---

## Pipeline

```
Raw GMGN + Alph AI Data
     │
     ▼
Feature Extraction ──▶ Category Scores (6) ──▶ Alpha Score (0-100)
                          │                           │
                          ▼                           ▼
                   Hard Filters (reject/OK)   Mode Select + Sizing
```

---

## Feature Vector

Every token gets a `FeatureVector` (30+ dimensions) at the point of analysis. This is **snapshotted and stored** with every paper trade for the learning engine.

### Categories & Features

#### MARKET MOMENTUM (from GMGN trending)
- `volume_1m`, `volume_5m`, `volume_1h`
- `swaps_1h`
- `price_change_1m`, `price_change_1h`
- `hot_level`

#### LIQUIDITY (from GMGN token)
- `liquidity_usd`
- `market_cap` (computed: price × circulating_supply)
- `pool_exchange`
- `is_on_curve` (bonding curve status)

#### SECURITY / RUG (from GMGN security)
- `rug_ratio` (0-1, >0.30 = high risk)
- `is_wash_trading`
- `open_source`
- `renounced_mint` (SOL-specific)
- `renounced_freeze_account` (SOL-specific)

#### HOLDER (from GMGN token/stat)
- `holder_count`
- `top_10_holder_rate`
- `dev_team_hold_rate`
- `creator_hold_rate`
- `suspected_insider_hold_rate`
- `fresh_wallet_rate`

#### WALLET SIGNALS (from GMGN stat)
- `smart_degen_count` (smart money wallets)
- `renowned_count` (KOL wallets)
- `sniper_count`
- `bundler_rate`
- `rat_trader_rate`

#### DEV PROFILE (from GMGN dev)
- `creator_status` (hold/close)
- `creator_prev_tokens`
- `creator_ath_mc`
- `cto_flag`
- `dexscr_ad`, `dexscr_boost`, `dexscr_trending_bar`

#### SOCIAL / META (GMGN link + Alph AI)
- `has_social_links`
- `launchpad_platform`
- `twitter_mentions_1h` (Alph AI)
- `twitter_sentiment` (Alph AI)
- `twitter_follower_count` (Alph AI)
- `twitter_ca_extracted` (Alph AI — was a CA found in recent tweets?)
- `signal_confidence` (Alph AI — Gold/Silver/Copper)

---

## Category Scores

Each category produces a 0-1 sub-score.

### Momentum
```
momentum = normalize(volume_1h) * 0.3
         + normalize(swaps_1h)  * 0.2
         + normalize(hot_level) * 0.2
         + clamp(price_change_1h, -100, +500) / 500 * 0.3
```

### Safety
```
safety = (1 - rug_ratio)                          * 0.35
       + (is_wash_trading ? 0 : 1)                * 0.15
       + (renounced_mint ? 1 : 0)                 * 0.15
       + (renounced_freeze_account ? 1 : 0)       * 0.15
       + (1 - top_10_holder_rate)                 * 0.20
```

### Holder Quality
```
holder = normalize(smart_degen_count)             * 0.35
       + normalize(renowned_count)                * 0.20
       + (1 - dev_team_hold_rate)                 * 0.20
       + (1 - suspected_insider_hold_rate)        * 0.15
       + (1 - fresh_wallet_rate)                  * 0.10
```

### Liquidity
```
liquidity = sigmoid(log10(liquidity_usd), midpoint=$50k)   * 0.5
          + (1 - is_on_curve)                              * 0.3
          + sigmoid(log10(market_cap), midpoint=$10k)      * 0.20
```

### Dev Trust
```
dev = (creator_status == "creator_close" ? 0 : 0.5) * 0.30
    + normalize(creator_ath_mc, max=$1M)             * 0.25
    + (cto_flag ? 0.8 : 0.3)                         * 0.20
    + (dexscr_boost ? 0.7 : 0.3)                     * 0.15
    + clamp(creator_prev_tokens, 0, 10) / 10         * 0.10
```

### Social (Alph AI enhanced)
```
social = normalize(twitter_mentions_1h)                         * 0.30
       + twitter_sentiment_score                                * 0.20
       + normalize(twitter_follower_count)                      * 0.20
       + signal_confidence_score(Gold=1,Silver=0.6,Copper=0.3)  * 0.30
```

---

## Composite Alpha Score

```
alpha = w_momentum * momentum
      + w_safety     * safety
      + w_holder     * holder
      + w_liquidity  * liquidity
      + w_dev        * dev
      + w_social     * social
```

Scaled to 0-100. Weights `w_*` are mutable (auto-tuned).

**Default weights:**
| Weight | Default |
|---|---|
| w_momentum | 0.25 |
| w_safety | 0.15 |
| w_holder | 0.20 |
| w_liquidity | 0.18 |
| w_dev | 0.07 |
| w_social | 0.15 |

---

## Hard Filters

Applied BEFORE scoring. Any failure rejects the token.

| Filter | Threshold | Rationale |
|---|---|---|
| `rug_ratio` | > 0.30 | High rug pull likelihood |
| `dev_team_hold_rate` | > 0.15 | Dev holds too much, can dump |
| `is_wash_trading` | true | Artificial volume |
| `renounced_mint` | false (on SOL) | Can mint more tokens |
| `creator_status` + `dev_team_hold_rate` | hold AND > 0.10 | Dev still holding AND large allocation |
| `liquidity_usd` | < $5,000 | Can't exit without massive slippage |

Hard filters are configurable in Settings (user can relax for more aggressive hunting). Safety floors prevent complete removal.

---

## Mode Selection

| Mode | Trigger | Sizing (paper) | Exit |
|---|---|---|---|
| **EXPLODE** | alpha >= 75 AND momentum >= 80 AND safety >= 70 | 0.5-1.0 SOL | TP +100-300%, SL -60%, trailing after +50% |
| **ALPHA** | alpha >= 55 AND safety >= 65 | 0.2-0.5 SOL | TP +50-150%, SL -40%, trailing after +30% |
| **SCALP** | momentum >= 70 AND (alpha < 55 OR safety < 65) | 0.1-0.2 SOL | TP +10-30%, SL -15%, tight stops |
| **FALLBACK** | alpha < 55 OR any hard filter borderline | 0.05-0.1 SOL | Any profit, cut fast |

---

## Rug Detection

Separate module (`rug_detect.rs`) producing a `RugReport`:

```rust
struct RugReport {
    severity: Severity,   // LOW | MEDIUM | HIGH | CRITICAL
    flags: Vec<RugFlag>,
    verdict: String,      // human-readable
}

struct RugFlag {
    name: String,
    severity: Severity,
    detail: String,
    value: f64,
    threshold: f64,
}
```

**Severity rules:**
- CRITICAL → blocks paper trade (override only with explicit confirmation)
- HIGH → strong warning in UI
- MEDIUM → shown in Analyzer
- LOW → informational

**Example flags:**
- `high_dev_allocation` (dev_team_hold_rate > 0.05)
- `high_rug_ratio` (rug_ratio > 0.30)
- `wash_trading_detected`
- `mint_not_renounced`
- `high_insider_rate` (suspected_insider_hold_rate > 0.20)
- `high_bundler_rate` (bundler_rate > 0.30)

---

## Narrative Detection

`narrative.rs` detects the meme "meta" a token belongs to:

- **AI** — name/description contains AI, GPT, bot, agent, neural
- **Dog** — doge, shib, dog, woof, bonk
- **Cat** — cat, meow, feline
- **Frog** — pepe, frog, apu
- **Political** — political figure names, vote, election
- **Celebrity** — celebrity names
- **Food** — food, drink names

Combined with Twitter mention analysis from Alph AI to assess narrative momentum: "is this narrative heating up or cooling down?"
