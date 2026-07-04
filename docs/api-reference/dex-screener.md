# DEX Screener API Reference (QuickScope Usage)

> Tertiary data source — cross-references GMGN trending/boosts.
> Free, no auth. Be polite with request rates.

Base URL: `https://api.dexscreener.com`

---

## Endpoints Used

| Endpoint | Method | QuickScope Use |
|---|---|---|
| `/latest/dex/search?q=<query>` | GET | Token/pair search by name or address |
| `/token-boosts/latest` | GET | Recently boosted tokens |
| `/token-boosts/top` | GET | Top boosted tokens (24h) |
| `/latest/dex/tokens/<address>` | GET | Pairs for a specific token |
| `/latest/dex/pairs/<chain>/<pair>` | GET | Specific pair data |

---

## Response Highlights

### Search / token pairs

Each pair includes:
- `baseToken` / `quoteToken` (address, symbol, name)
- `priceUsd`, `priceNative`
- `liquidity.usd` — pool liquidity
- `fdv` — fully diluted valuation
- `marketCap`
- `volume` (h24, h6, h1, m30, m15)
- `priceChange` (m5, h1, h6, h24)
- `txns` — buy/sell counts per window
- `pairCreatedAt`
- `info` — social links, image, header image
- `boosts` / `alerts` — community boost count

### Boosts

Tokens that users have paid to boost (visibility). Useful conviction signal:
- `/token-boosts/top` — highest boosters in 24h.
- A token boosted on DEX Screener AND trending on GMGN = stronger signal.

---

## QuickScope Integration

```
DataOrchestrator.fetch_token_conviction(token_address):
  1. GMGN: token info + security + holders  → base analysis
  2. Alph AI: token-detail + signals        → social + signal confidence
  3. DEX Screener: /latest/dex/tokens/<addr> → boosts, cross-DEX pairs
  
  Merge → conviction_multiplier:
    boosted_on_dexscr ? +0.05 : 0
    multi_dex_pairs   ? +0.05 : 0
```

The conviction multiplier adjusts the Alpha Score slightly (±5%) based on cross-source agreement.

---

## Cache TTL

| Data | TTL |
|---|---|
| Search results | 60s |
| Token pairs | 60s |
| Boost lists | 120s |

---

## Rate Limits

No documented hard limit. DEX Screener is free but be polite:
- Cache everything (TTL 60-120s).
- Batch lookups where possible.
- Avoid tight polling loops — use Alph AI WebSocket for real-time.
