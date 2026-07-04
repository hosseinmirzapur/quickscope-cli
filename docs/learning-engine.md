# Learning Engine

> Auto-tuner (statistical) + LLM post-mortem (on-demand).
> Full detail in master spec §8.

---

## Two-Pronged System

```
Learning System
├── Auto-Tuner (always-on, statistical, deterministic)
│   └── Runs every N trades (default 20), nudges weights toward winners' patterns
│
└── LLM Post-Mortem (on-demand, user-triggered)
    └── Sends trade journal to LLM, gets strategy suggestions
```

---

## Auto-Tuner

**Not ML.** Pure statistics: "what do winners have in common that losers don't?"

### Input

Every paper trade logs to the `positions` table:
- `feature_vector` (JSON) — 30+ dimensions at entry
- `alpha_score` — composite score at entry
- `mode` — EXPLODE/ALPHA/SCALP/FALLBACK
- `pnl_percent` — outcome
- `outcome` — Win | Loss
- `duration_mins`

### Algorithm (runs after every N trades)

```
1. Load last N closed trades from journal.
2. Separate into Winners (pnl > 0) and Losers (pnl < 0).
3. For each feature in the feature vector:
     winner_mean = mean(feature across winners)
     loser_mean  = mean(feature across losers)
     winner_std  = std(feature across winners)
     discrimination = |winner_mean - loser_mean| / (winner_std + epsilon)
4. For each category weight (w_momentum, w_safety, ...):
     - Compute aggregate discrimination for that category's features.
     - High discrimination → increase weight slightly (category is predictive).
     - Low discrimination → decrease weight slightly.
5. For each hard filter threshold:
     - If most losers had feature values beyond a tighter threshold → tighten.
     - If most winners were borderline (just inside) → relax slightly.
6. Compute tuning delta (old vs new weights).
7. Clamp total delta to ±5% per run.
8. Apply weight bounds [0.05, 0.40] per weight.
9. Apply threshold safety floors (hard filters can tighten but not relax below floor).
10. Save new config to alpha_config table.
11. Log full delta to tuning_history table.
```

### Guard Rails

| Guard | Rule | Why |
|---|---|---|
| Max delta per run | ±5% total weight shift | Prevent overfitting to small sample |
| Min sample size | 10+ wins AND 10+ losses before first tune | Meaningless with too few trades |
| Weight bounds | Each weight in [0.05, 0.40] | No single factor dominates |
| Threshold bounds | Hard filters tighten only, never below safety floor | Don't undo safety for performance |
| Revert | `Reset to Default` in Settings | User can always undo |
| Audit log | Every delta in `tuning_history` with sample size + discrimination | Transparent |

### Example Run

```
After 20 trades (12 wins, 8 losses):

Feature discrimination:
  rug_ratio:      winners 0.08, losers 0.28 → HIGH
  smart_degen:    winners 6.2,  losers 1.8  → HIGH
  liq_depth:      winners $180k, losers $45k → HIGH
  token_age:      winners 2.1h, losers 5.8h → MODERATE
  fresh_wallet:   winners 0.18, losers 0.22 → LOW

Tuning result (clamped to ±5%):
  w_safety:     0.15 → 0.18
  w_holder:     0.20 → 0.23
  w_liquidity:  0.18 → 0.20
  w_dev:        0.07 → 0.06
  w_momentum:   0.25 → 0.23

  hf_rug_ratio_max:     0.30 → 0.25
  hf_liquidity_min_usd: $5k  → $8k
```

---

## LLM Post-Mortem

User clicks `[Run Post-Mortem]` in Strategy tab.

### Flow

```
1. Collect trade journal for selected period (today / 7d / 30d).
2. Compute summary stats:
     win rate, avg win %, avg loss %, best/worst trades,
     feature discrimination (winners vs losers)
3. Build prompt:
     - System: trading mindset from memecoin-alpha-agent skill
     - Reference: GMGN workflow docs
     - Data: trade journal (formatted markdown table with features)
     - Current weights + hard filter thresholds
     - Discrimination analysis
4. Send to LLM provider (OpenAI / Anthropic / Ollama).
5. Display raw response in Strategy tab.
6. Parse for actionable suggestions (lines with "SUGGESTION:").
7. Each suggestion has [Apply] / [Dismiss] buttons.
8. Applied suggestions go through the same ±5% guard rail as auto-tuning.
```

### System Prompt Template

```
You are an elite memecoin trading analyst. You review paper trade
journals and provide actionable strategy improvements. Follow the
QuickScope trading mindset: assume every token rugs, every dev
exits, every influencer has already sold. Preservation > multiplication.

Rules:
- Suggest specific threshold/weight changes, not vague advice
- Reference specific trade numbers from the journal
- Consider survivorship bias — wins may be luck, not strategy
- Prioritize safety adjustments over aggression
- Keep suggestions concise (3-5 max per session)
```

### User Prompt Template

```
Session: {date} | Trades: {total} | Wins: {wins} | Losses: {losses}
Win Rate: {win_rate} | Avg Win: {avg_win_pct}% | Avg Loss: {avg_loss_pct}%
Total PnL: {total_pnl_sol} SOL

Current Alpha Filter Weights:
{weights_table}

Current Hard Filters:
{filters_table}

Trade Journal:
{formatted_trades_with_features}

Feature Discrimination (winners vs losers):
{discrimination_analysis}

Please analyze and suggest specific adjustments.
```

---

## LLM Provider Trait

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, prompt: LLMRequest) -> Result<LLMResponse>;
    fn name(&self) -> &str;
}

pub struct LLMRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

pub struct LLMResponse {
    pub content: String,
    pub provider: String,
    pub model: String,
    pub tokens_used: u32,
}
```

### Implementations

| File | Provider | Notes |
|---|---|---|
| `llm/openai.rs` | OpenAI GPT-4o / GPT-4o-mini | Default; cheap mini for post-mortems |
| `llm/anthropic.rs` | Anthropic Claude | Strong at structured analysis |
| `llm/ollama.rs` | Local Ollama (llama3, mistral) | Zero cost, offline, needs hardware |

Factory in `llm/mod.rs` selects provider based on Settings config.

---

## Storage

### `tuning_history` table

Every auto-tune run logs:
- `tuned_at` — timestamp
- `sample_size`, `wins`, `losses`
- `old_weights` (JSON), `new_weights` (JSON)
- `old_filters` (JSON), `new_filters` (JSON)
- `discrimination` (JSON — full feature analysis)

### `post_mortems` table

Every LLM post-mortem logs:
- `run_at`, `period_start`, `period_end`
- `provider`, `model`
- `prompt_summary` (truncated)
- `response` (full)
- `suggestions_applied`, `suggestions_dismissed`

---

## Auditability

Both learning paths are fully transparent:
- Auto-tuner: every delta is in `tuning_history` with the discrimination analysis that justified it.
- LLM: every post-mortem is in `post_mortems` with the full prompt and response.
- User can review the entire learning history in the Strategy tab and revert any change.
