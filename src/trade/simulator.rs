use crate::data::models::*;

/// Simulate a paper buy — calculates how many tokens you'd receive.
pub fn simulate_buy(
    amount_sol: f64,
    current_price_usd: f64,
    sol_price_usd: f64,
    slippage_percent: f64,
    liquidity_usd: f64,
) -> PaperBuyResult {
    let effective_price = current_price_usd * (1.0 + slippage_percent / 100.0);
    let amount_usd = amount_sol * sol_price_usd;
    let tokens_received = amount_usd / effective_price;

    // Liquidity impact warning
    let impact = if liquidity_usd > 0.0 {
        amount_usd / liquidity_usd
    } else {
        0.0
    };
    let impact_warning = impact > 0.05;

    PaperBuyResult {
        effective_price,
        tokens_received,
        amount_usd,
        liquidity_impact: impact,
        impact_warning,
    }
}

/// Simulate a paper sell — calculates PnL.
pub fn simulate_sell(
    amount_tokens: f64,
    current_price_usd: f64,
    entry_price_usd: f64,
    slippage_percent: f64,
    amount_sol_invested: f64,
    tokens_owned: f64,
    sell_percent: f64,
) -> PaperSellResult {
    let tokens_to_sell = tokens_owned * (sell_percent / 100.0);
    let effective_price = current_price_usd * (1.0 - slippage_percent / 100.0);
    let proceeds_usd = tokens_to_sell * effective_price;
    let cost_basis_usd = tokens_to_sell * entry_price_usd;
    let pnl_usd = proceeds_usd - cost_basis_usd;
    let pnl_percent = if cost_basis_usd > 0.0 {
        (pnl_usd / cost_basis_usd) * 100.0
    } else {
        0.0
    };
    let pnl_sol = if amount_sol_invested > 0.0 && tokens_owned > 0.0 {
        pnl_usd * (amount_sol_invested * (sell_percent / 100.0)) / (tokens_owned * entry_price_usd)
    } else {
        0.0
    };

    PaperSellResult {
        tokens_sold: tokens_to_sell,
        effective_price,
        proceeds_usd,
        pnl_usd,
        pnl_percent,
        pnl_sol,
        is_partial: sell_percent < 100.0,
    }
}

#[derive(Debug, Clone)]
pub struct PaperBuyResult {
    pub effective_price: f64,
    pub tokens_received: f64,
    pub amount_usd: f64,
    pub liquidity_impact: f64,
    pub impact_warning: bool,
}

#[derive(Debug, Clone)]
pub struct PaperSellResult {
    pub tokens_sold: f64,
    pub effective_price: f64,
    pub proceeds_usd: f64,
    pub pnl_usd: f64,
    pub pnl_percent: f64,
    pub pnl_sol: f64,
    pub is_partial: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_buy() {
        let result = simulate_buy(
            0.5,     // 0.5 SOL
            0.0001,  // price $0.0001
            150.0,   // SOL price $150
            3.0,     // 3% slippage
            500_000.0, // liquidity
        );
        assert!(result.tokens_received > 0.0);
        assert!(result.effective_price > 0.0001);
        assert!(!result.impact_warning);
    }

    #[test]
    fn test_simulate_buy_impact_warning() {
        let result = simulate_buy(
            100.0,   // 100 SOL (large)
            0.01,
            150.0,
            3.0,
            200_000.0, // low liquidity
        );
        assert!(result.impact_warning);
    }

    #[test]
    fn test_simulate_sell_profit() {
        let result = simulate_sell(
            100_000.0,   // tokens to sell
            0.0002,      // current price
            0.0001,      // entry price
            3.0,         // slippage
            0.5,         // SOL invested
            100_000.0,   // tokens owned
            100.0,       // sell 100%
        );
        assert!(result.pnl_usd > 0.0);
        assert!(result.pnl_percent > 0.0);
        assert!(!result.is_partial);
    }

    #[test]
    fn test_simulate_sell_partial() {
        let result = simulate_sell(
            100_000.0,
            0.0002,
            0.0001,
            3.0,
            0.5,
            100_000.0,
            50.0, // sell 50%
        );
        assert!(result.is_partial);
        assert!(result.tokens_sold == 50_000.0);
    }
}