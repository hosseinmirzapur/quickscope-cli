use anyhow::Result;
use sqlx::SqlitePool;

use super::llm::prompts::{build_post_mortem_prompt, TradeSummary, POST_MORTEM_SYSTEM};
use super::llm::LlmProvider;
use crate::storage::{journal, positions};

/// Run an LLM post-mortem analysis on closed trades for a given period.
pub async fn run_post_mortem(
    pool: &SqlitePool,
    provider: &LlmProvider,
    period_start: &str,
    period_end: &str,
) -> Result<String> {
    // 1. Gather closed positions in the period
    let closed =
        positions::get_closed_positions(pool, Some(period_start), Some(period_end)).await?;

    if closed.is_empty() {
        return Ok("No closed trades in the selected period.".to_string());
    }

    // 2. Build trade summaries
    let wins = closed
        .iter()
        .filter(|p| p.pnl_sol.unwrap_or(0.0) > 0.0)
        .count();
    let losses = closed
        .iter()
        .filter(|p| p.pnl_sol.unwrap_or(0.0) <= 0.0)
        .count();
    let total_pnl: f64 = closed.iter().map(|p| p.pnl_sol.unwrap_or(0.0)).sum();

    let trade_summaries: Vec<TradeSummary> = closed
        .iter()
        .map(|p| {
            // Try to extract scores from feature_vector JSON
            let (momentum, safety) = p
                .feature_vector
                .as_ref()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok())
                .map(|fv| {
                    (
                        fv.get("momentum").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        fv.get("safety").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    )
                })
                .unwrap_or((0.0, 0.0));

            TradeSummary {
                symbol: p.token_symbol.clone(),
                mode: p.mode.clone(),
                entry_price: p.entry_price,
                exit_price: p.exit_price.unwrap_or(0.0),
                pnl_sol: p.pnl_sol.unwrap_or(0.0),
                pnl_percent: p.pnl_percent.unwrap_or(0.0),
                alpha_score: p.alpha_score,
                momentum_score: momentum,
                safety_score: safety,
                opened_at: p.opened_at.clone(),
                closed_at: p.closed_at.clone().unwrap_or_default(),
            }
        })
        .collect();

    // 3. Build prompt
    let user_prompt = build_post_mortem_prompt(
        period_start,
        period_end,
        closed.len(),
        wins,
        losses,
        total_pnl,
        &trade_summaries,
    );

    // 4. Call LLM
    let response = provider.chat(POST_MORTEM_SYSTEM, &user_prompt).await?;

    // 5. Save to post_mortems table
    let summary = format!(
        "{} trades ({}W/{}L) {:.2} SOL",
        closed.len(),
        wins,
        losses,
        total_pnl
    );
    journal::log_post_mortem(
        pool,
        period_start,
        period_end,
        provider.name(),
        provider.model(),
        &summary,
        &response,
    )
    .await?;

    Ok(response)
}
