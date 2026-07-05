/// System prompt for the LLM post-mortem analysis.
pub const POST_MORTEM_SYSTEM: &str = r#"You are a memecoin trading analyst. Your job is to review a trader's paper trading journal and provide actionable feedback.

For each period under review:
1. Summarize the period: total trades, win rate, total PnL
2. Identify the top 3 winning trades — what made them work?
3. Identify the top 3 losing trades — what went wrong?
4. Find patterns: do winners share common characteristics (narrative, entry timing, mode)? Do losers?
5. Suggest concrete, specific adjustments:
   - Should any Alpha Filter weight be adjusted? Which direction?
   - Should any hard filter threshold be tightened?
   - Are certain modes consistently underperforming?
   - Any behavioral patterns (overtrading, FOMO, cutting winners early)?
6. Rate the session: TERRIBLE / POOR / OK / GOOD / GREAT

Be specific, data-driven, and actionable. No fluff. Use numbers. Format suggestions as bullet points."#;

/// Build a user prompt with trade journal data.
pub fn build_post_mortem_prompt(
    period_start: &str,
    period_end: &str,
    total_trades: usize,
    wins: usize,
    losses: usize,
    total_pnl: f64,
    trade_summaries: &[TradeSummary],
) -> String {
    let mut prompt = format!(
        "Review this paper trading journal from {} to {}:\n\n",
        period_start, period_end
    );

    prompt.push_str(&format!(
        "- Total trades: {}\n- Wins: {}\n- Losses: {}\n- Win rate: {:.1}%\n- Total PnL: {:.2} SOL\n\n",
        total_trades,
        wins,
        losses,
        if total_trades > 0 { wins as f64 / total_trades as f64 * 100.0 } else { 0.0 },
        total_pnl,
    ));

    prompt.push_str("Trade details:\n");
    for trade in trade_summaries {
        prompt.push_str(&format!(
            "  - {} | {} | Mode: {} | PnL: {:+.2} SOL | Alpha: {:.0} | Safety: {:.0}% | Momentum: {:.0}%\n",
            trade.symbol,
            if trade.pnl_sol >= 0.0 { "WIN" } else { "LOSS" },
            trade.mode,
            trade.pnl_sol,
            trade.alpha_score,
            trade.safety_score * 100.0,
            trade.momentum_score * 100.0,
        ));
    }

    prompt.push_str("\nWhat went right? What went wrong? Give specific, actionable suggestions.");
    prompt
}

#[derive(Debug, Clone)]
pub struct TradeSummary {
    pub symbol: String,
    pub mode: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub pnl_sol: f64,
    pub pnl_percent: f64,
    pub alpha_score: f64,
    pub momentum_score: f64,
    pub safety_score: f64,
    pub opened_at: String,
    pub closed_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt() {
        let trades = vec![
            TradeSummary {
                symbol: "PEPE".to_string(),
                mode: "EXPLODE".to_string(),
                entry_price: 0.00001,
                exit_price: 0.00003,
                pnl_sol: 1.0,
                pnl_percent: 200.0,
                alpha_score: 92.0,
                momentum_score: 0.90,
                safety_score: 0.85,
                opened_at: "2026-07-01".to_string(),
                closed_at: "2026-07-02".to_string(),
            },
            TradeSummary {
                symbol: "WIF".to_string(),
                mode: "SCALP".to_string(),
                entry_price: 1.0,
                exit_price: 0.85,
                pnl_sol: -0.15,
                pnl_percent: -15.0,
                alpha_score: 45.0,
                momentum_score: 0.75,
                safety_score: 0.30,
                opened_at: "2026-07-03".to_string(),
                closed_at: "2026-07-03".to_string(),
            },
        ];

        let prompt = build_post_mortem_prompt(
            "2026-07-01", "2026-07-05",
            2, 1, 1, 0.85,
            &trades,
        );

        assert!(prompt.contains("PEPE"));
        assert!(prompt.contains("WIF"));
        assert!(prompt.contains("EXPLODE"));
        assert!(prompt.contains("1.00 SOL"));
        assert!(prompt.contains("-0.15 SOL"));
    }
}