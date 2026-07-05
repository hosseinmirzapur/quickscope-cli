//! Paper Trade Engine — simulated buy/sell, TP/SL monitoring, risk management.

use anyhow::Result;
use crate::data::models::*;
use crate::alpha;
use crate::storage::DbManager;

mod simulator;
mod risk;
mod monitor;

pub use simulator::{simulate_buy, simulate_sell, PaperBuyResult, PaperSellResult};
pub use risk::RiskManager;
pub use monitor::TpSlMonitor;

/// The trade engine orchestrator — wires everything together.
pub struct TradeEngine {
    pub risk_manager: RiskManager,
    db: DbManager,
}

impl TradeEngine {
    pub fn new(db: DbManager) -> Self {
        Self {
            risk_manager: RiskManager::new(),
            db,
        }
    }

    /// Execute a paper buy.
    pub async fn paper_buy(
        &mut self,
        detail: &TokenDetail,
        config: &AlphaConfig,
        amount_sol: f64,
        current_price_usd: f64,
        sol_price_usd: f64,
    ) -> Result<PaperBuyResult> {
        let report = alpha::analyze_token(detail, config);

        let open_count = crate::storage::positions::count_open_positions(&self.db.pool).await?;
        let same_count = crate::storage::positions::count_open_for_token(
            &self.db.pool, &detail.token.address,
        ).await?;

        let risk_result = self.risk_manager.check_pre_trade(
            amount_sol, &report.mode, &report.sizing,
            open_count as u64, same_count as u64,
        );

        match risk_result {
            PreTradeCheckResult::Rejected(reason) => anyhow::bail!("Trade rejected: {}", reason),
            PreTradeCheckResult::Warning(_) => {
                tracing::warn!("Trade warning: continuing");
            }
            PreTradeCheckResult::Approved => {}
        }

        let slippage = 3.0;
        let buy_result = simulate_buy(
            amount_sol, current_price_usd, sol_price_usd,
            slippage, detail.token.liquidity_usd,
        );

        if buy_result.impact_warning {
            tracing::warn!(
                "Liquidity impact {:.1}% — consider smaller size",
                buy_result.liquidity_impact * 100.0
            );
        }

        let (tp_pct, sl_pct) = alpha::exit_params_for_mode(&report.mode);

        let fv_json = serde_json::to_string(&report.feature_vector)?;
        let rug_json = serde_json::to_string(&report.rug_report)?;

        crate::storage::positions::insert_position(
            &self.db.pool,
            &detail.token.address,
            &detail.token.symbol,
            "buy",
            buy_result.effective_price,
            amount_sol,
            buy_result.tokens_received,
            slippage,
            report.mode.as_str(),
            Some(tp_pct),
            Some(sl_pct),
            &fv_json,
            report.alpha_score,
            &rug_json,
        ).await?;

        Ok(buy_result)
    }

    /// Execute a paper sell (partial or full).
    pub async fn paper_sell(
        &mut self,
        position_id: &str,
        current_price_usd: f64,
        sell_percent: f64,
    ) -> Result<PaperSellResult> {
        let pos = crate::storage::positions::get_position_by_id(&self.db.pool, position_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Position {} not found", position_id))?;

        if pos.status != "open" {
            anyhow::bail!("Position {} is not open", position_id);
        }

        let result = simulate_sell(
            pos.amount_tokens,
            current_price_usd,
            pos.entry_price,
            3.0,
            pos.amount_sol,
            pos.amount_tokens,
            sell_percent,
        );

        if !result.is_partial {
            crate::storage::positions::close_position(
                &self.db.pool,
                position_id,
                result.effective_price,
                result.pnl_sol,
                result.pnl_percent,
            ).await?;

            self.risk_manager.record_trade(result.pnl_sol);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buy_result_creation() {
        let result = PaperBuyResult {
            effective_price: 0.000103,
            tokens_received: 450_000.0,
            amount_usd: 75.0,
            liquidity_impact: 0.01,
            impact_warning: false,
        };
        assert!(!result.impact_warning);
    }
}