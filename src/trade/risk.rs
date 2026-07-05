use crate::data::models::*;

/// Risk manager for paper trading.
/// Enforces daily loss cap, per-trade limits, kill switch, etc.
#[derive(Debug, Clone)]
pub struct RiskManager {
    pub state: RiskState,
}

impl RiskManager {
    pub fn new() -> Self {
        Self {
            state: RiskState::default(),
        }
    }

    /// Run pre-trade checks before every paper buy.
    /// Returns Approved, Rejected(reason), or Warning(reason).
    pub fn check_pre_trade(
        &self,
        amount_sol: f64,
        _mode: &TradeMode,
        sizing: &SizingBounds,
        open_positions_count: u64,
        same_token_count: u64,
    ) -> PreTradeCheckResult {
        // 1. Kill switch
        if self.state.kill_switch_active {
            return PreTradeCheckResult::Rejected(
                "Kill switch active — trading disabled".to_string()
            );
        }

        // 2. Daily loss cap exceeded
        if self.state.daily_realized_pnl <= -self.state.daily_loss_cap_sol {
            return PreTradeCheckResult::Rejected(format!(
                "Daily loss cap ({} SOL) exceeded. Current PnL: {:.2} SOL",
                self.state.daily_loss_cap_sol, self.state.daily_realized_pnl
            ));
        }

        // 3. Per-trade max
        if amount_sol > self.state.per_trade_max_sol {
            return PreTradeCheckResult::Rejected(format!(
                "Amount ({:.2} SOL) exceeds per-trade max ({:.2} SOL)",
                amount_sol, self.state.per_trade_max_sol
            ));
        }

        // 4. Max open positions
        if open_positions_count >= self.state.max_open_positions as u64 {
            return PreTradeCheckResult::Rejected(format!(
                "Max open positions ({}) reached",
                self.state.max_open_positions
            ));
        }

        // 5. Max same token
        if same_token_count >= self.state.max_same_token as u64 {
            return PreTradeCheckResult::Rejected(format!(
                "Max positions for same token ({}) reached",
                self.state.max_same_token
            ));
        }

        // 6. Mode sizing bounds
        if amount_sol < sizing.min_sol || amount_sol > sizing.max_sol {
            return PreTradeCheckResult::Warning(format!(
                "Amount ({:.2} SOL) outside mode sizing bounds ({:.2} - {:.2} SOL)",
                amount_sol, sizing.min_sol, sizing.max_sol
            ));
        }

        // 7. 2 daily wins warning
        if self.state.wins_today >= 2 {
            return PreTradeCheckResult::Warning(
                "2 wins today — greed kills. Consider stopping.".to_string()
            );
        }

        PreTradeCheckResult::Approved
    }

    /// Activate kill switch (e.g., daily loss cap hit).
    pub fn activate_kill_switch(&mut self) {
        self.state.kill_switch_active = true;
    }

    /// Deactivate kill switch (scary override from Settings).
    pub fn deactivate_kill_switch(&mut self) {
        self.state.kill_switch_active = false;
    }

    /// Record a completed trade's PnL.
    pub fn record_trade(&mut self, pnl_sol: f64) {
        self.state.daily_realized_pnl += pnl_sol;
        self.state.trades_today += 1;
        if pnl_sol > 0.0 {
            self.state.wins_today += 1;
        } else if pnl_sol < 0.0 {
            self.state.losses_today += 1;
        }

        // Auto kill switch if daily loss cap exceeded
        if self.state.daily_realized_pnl <= -self.state.daily_loss_cap_sol {
            self.activate_kill_switch();
        }
    }

    /// Reset daily counters (call at midnight UTC).
    pub fn reset_daily(&mut self) {
        self.state.daily_realized_pnl = 0.0;
        self.state.trades_today = 0;
        self.state.wins_today = 0;
        self.state.losses_today = 0;
        self.state.kill_switch_active = false;
    }
}

impl Default for RiskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approved_trade() {
        let rm = RiskManager::new();
        let sizing = SizingBounds { min_sol: 0.1, max_sol: 0.5 };
        let result = rm.check_pre_trade(
            0.3, &TradeMode::Alpha, &sizing, 2, 0,
        );
        assert_eq!(result, PreTradeCheckResult::Approved);
    }

    #[test]
    fn test_rejected_amount_too_high() {
        let rm = RiskManager::new();
        let sizing = SizingBounds { min_sol: 0.1, max_sol: 0.5 };
        let result = rm.check_pre_trade(
            5.0, &TradeMode::Alpha, &sizing, 2, 0,
        );
        assert!(matches!(result, PreTradeCheckResult::Rejected(_)));
    }

    #[test]
    fn test_rejected_max_positions() {
        let rm = RiskManager::new();
        let sizing = SizingBounds { min_sol: 0.1, max_sol: 0.5 };
        let result = rm.check_pre_trade(
            0.3, &TradeMode::Alpha, &sizing, 5, 0,
        );
        assert!(matches!(result, PreTradeCheckResult::Rejected(_)));
    }

    #[test]
    fn test_warning_two_wins() {
        let mut rm = RiskManager::new();
        rm.record_trade(0.5); // win
        rm.record_trade(0.3); // win
        let sizing = SizingBounds { min_sol: 0.1, max_sol: 0.5 };
        let result = rm.check_pre_trade(
            0.3, &TradeMode::Alpha, &sizing, 2, 0,
        );
        assert!(matches!(result, PreTradeCheckResult::Warning(_)));
    }

    #[test]
    fn test_kill_switch_activated_on_loss() {
        let mut rm = RiskManager::new();
        assert!(!rm.state.kill_switch_active);
        rm.record_trade(-6.0); // exceeds 5 SOL cap
        assert!(rm.state.kill_switch_active);

        let sizing = SizingBounds { min_sol: 0.1, max_sol: 0.5 };
        let result = rm.check_pre_trade(
            0.3, &TradeMode::Alpha, &sizing, 0, 0,
        );
        assert!(matches!(result, PreTradeCheckResult::Rejected(_)));
    }

    #[test]
    fn test_daily_reset() {
        let mut rm = RiskManager::new();
        rm.record_trade(-3.0);
        assert_eq!(rm.state.trades_today, 1);

        rm.reset_daily();
        assert_eq!(rm.state.trades_today, 0);
        assert!(!rm.state.kill_switch_active);
    }
}